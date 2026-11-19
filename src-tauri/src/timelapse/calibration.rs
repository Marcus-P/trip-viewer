//! Local, bounded empirical tuning. No GPU model allowlist and no telemetry.
use super::{
    ffmpeg::{self, Encoder, GpuScale, ProcessingProfile},
    types::FfmpegCapabilities,
    CancelFlag,
};
use crate::{app_settings::AppSettingsHandle, error::AppError};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::Path,
    sync::atomic::Ordering,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const REVISION: u32 = 1;
const TTL: u64 = 7 * 24 * 3600;
const GIB: u64 = 1024 * 1024 * 1024;
pub const BATCH_BUDGET: Duration = Duration::from_secs(45);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub encoder: Encoder,
    pub profile: ProcessingProfile,
    pub device: Option<String>,
    pub decoder_device: Option<String>,
    pub filter_threads: usize,
    pub concurrency: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    key: String,
    created: u64,
    pub choice: Choice,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub state: &'static str,
    pub message: String,
}
impl Choice {
    pub fn baseline(caps: &FfmpegCapabilities) -> Self {
        Self {
            encoder: Encoder::pick(caps),
            profile: ProcessingProfile {
                decoder: caps.decoder,
                software_scale: caps.software_processing,
                gpu_scale: None,
            },
            device: caps.vaapi_device.clone(),
            decoder_device: caps.decoder_device.clone(),
            filter_threads: 2,
            concurrency: 1,
        }
    }
    pub fn describe(&self) -> String {
        format!(
            "{} · decode {} · scale {} · {} filter threads · {} job(s)",
            self.encoder.as_str(),
            self.profile.decoder.as_str(),
            self.encoder.scaler_label(self.profile),
            self.filter_threads,
            self.concurrency
        )
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn better(candidate: f64, baseline: f64) -> bool {
    candidate.is_finite() && candidate > 0.0 && baseline.is_finite() && candidate < baseline * 0.85
}
fn cached(records: &[Record], key: &str, time: u64) -> Option<Choice> {
    records
        .iter()
        .find(|r| {
            r.key == key
                && time >= r.created
                && time - r.created < TTL
                && (1..=8).contains(&r.choice.filter_threads)
                && (1..=4).contains(&r.choice.concurrency)
        })
        .map(|r| r.choice.clone())
}
fn query(
    path: &str,
    args: &[&str],
    cancel: &CancelFlag,
    deadline: Instant,
) -> Result<String, AppError> {
    let remaining = deadline
        .saturating_duration_since(Instant::now())
        .min(Duration::from_secs(4));
    if remaining.is_zero() {
        return Err(AppError::Internal("calibration budget exhausted".into()));
    }
    let mut cmd = ffmpeg::ffmpeg_command(path);
    cmd.args(args);
    ffmpeg::tuning_query(cmd, cancel, remaining)
}

/// Hash technical runtime details only. Driver queries never collect host names,
/// users, serials or GPU UUIDs. If unavailable, disable persistent reuse this run.
fn runtime_fingerprint(cancel: &CancelFlag, deadline: Instant) -> String {
    #[cfg(windows)]
    let result = query("powershell.exe", &["-NoProfile", "-NonInteractive", "-Command",
        "Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion | ConvertTo-Json -Compress"], cancel, deadline).ok();
    #[cfg(target_os = "linux")]
    let result = {
        let mut text = std::fs::read_to_string("/proc/version").unwrap_or_default();
        for node in ffmpeg::vaapi_render_nodes() {
            if let Some(name) = Path::new(&node).file_name() {
                let base = Path::new("/sys/class/drm").join(name).join("device");
                for file in ["vendor", "device", "uevent"] {
                    if let Ok(s) = std::fs::read_to_string(base.join(file)) {
                        text.push_str(&s);
                    }
                }
            }
        }
        // Userspace VA driver upgrades need invalidation even with the same kernel.
        for dir in [
            "/usr/lib/x86_64-linux-gnu/dri",
            "/usr/lib/aarch64-linux-gnu/dri",
        ] {
            if let Ok(entries) = std::fs::read_dir(dir) {
                let mut entries: Vec<_> = entries.flatten().collect();
                entries.sort_by_key(|e| e.file_name());
                for entry in entries {
                    if let Ok(m) = entry.metadata() {
                        text.push_str(&format!(
                            "{:?}:{}:{:?}",
                            entry.file_name(),
                            m.len(),
                            m.modified().ok()
                        ));
                    }
                }
            }
        }
        if let Ok(s) = std::fs::read_to_string("/proc/driver/nvidia/version") {
            text.push_str(&s);
        }
        let _ = (cancel, deadline);
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    };
    #[cfg(target_os = "macos")]
    let result = query(
        "/usr/sbin/system_profiler",
        &["SPDisplaysDataType", "-detailLevel", "mini"],
        cancel,
        deadline,
    )
    .ok()
    .and_then(|s| {
        query("/usr/bin/sw_vers", &["-productVersion"], cancel, deadline)
            .ok()
            .map(|version| {
                format!(
                    "{version}\n{}",
                    s.lines()
                        .filter(|l| !l.contains("Serial") && !l.contains("UUID"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
    });
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    let result: Option<String> = None;
    result.unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

/// Live host-memory ceiling; a 12-GiB job allowance deliberately retains the
/// existing pessimistic 4K protection. Unknown availability means sequential.
pub fn safe_parallelism(requested: usize) -> usize {
    memory_cap(requested, available_memory())
}
fn memory_cap(requested: usize, available: Option<u64>) -> usize {
    available
        .map(|b| {
            requested
                .clamp(1, 4)
                .min((b.saturating_sub(2 * GIB) / (12 * GIB)).max(1) as usize)
        })
        .unwrap_or(1)
}
#[cfg(target_os = "linux")]
fn available_memory() -> Option<u64> {
    std::fs::read_to_string("/proc/meminfo")
        .ok()?
        .lines()
        .find_map(|l| {
            l.strip_prefix("MemAvailable:")?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()
        })
        .map(|v| v * 1024)
}
#[cfg(windows)]
fn available_memory() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut status) } != 0 {
        Some(status.ullAvailPhys)
    } else {
        None
    }
}
// Conservative until a reliable live memory measure exists on this platform.
#[cfg(not(any(windows, target_os = "linux")))]
fn available_memory() -> Option<u64> {
    None
}

fn measure(
    path: &str,
    source: &Path,
    choice: &Choice,
    cancel: &CancelFlag,
    deadline: Instant,
) -> Option<f64> {
    let remaining = deadline
        .saturating_duration_since(Instant::now())
        .min(Duration::from_secs(6));
    if remaining < Duration::from_millis(400)
        || cancel.load(Ordering::Relaxed)
        || safe_parallelism(choice.concurrency) < choice.concurrency
    {
        return None;
    }
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    let start = Instant::now();
    let ok = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..choice.concurrency)
            .map(|_| {
                scope.spawn(|| {
                    ffmpeg::benchmark(
                        path,
                        source,
                        choice.encoder,
                        choice.profile,
                        choice.device.as_deref(),
                        choice.decoder_device.as_deref(),
                        choice.filter_threads,
                        (cores / choice.concurrency).clamp(1, 8),
                        cancel,
                        remaining,
                    )
                    .is_ok()
                })
            })
            .collect();
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().unwrap_or(false))
            .collect();
        results.into_iter().all(|ok| ok)
    });
    if ok && !cancel.load(Ordering::Relaxed) {
        Some(start.elapsed().as_secs_f64() / choice.concurrency as f64)
    } else {
        None
    }
}

/// Calibrate before production work starts, under the batch's shared deadline.
/// Measurements include startup/transfer/drain costs. Short samples are a
/// practical estimate, not a guarantee of the global optimum on every input.
pub fn select(
    path: &str,
    source: &Path,
    caps: &FfmpegCapabilities,
    settings: &AppSettingsHandle,
    cancel: &CancelFlag,
    deadline: Instant,
    jobs: usize,
    emit: impl Fn(Status),
) -> Choice {
    let baseline = Choice::baseline(caps);
    if cancel.load(Ordering::Relaxed) || Instant::now() >= deadline {
        return baseline;
    }
    emit(Status {
        state: "measuring",
        message: "Calibrating a short local video sample (up to 45 seconds per batch)…".into(),
    });
    let mut cache_hit = false;
    let result = (|| -> Result<Choice, AppError> {
        let source_str = source
            .to_str()
            .ok_or_else(|| AppError::Internal("invalid source path".into()))?;
        let metadata = query(
            path,
            &[
                "-hide_banner",
                "-i",
                source_str,
                "-map",
                "0:v:0",
                "-t",
                "0",
                "-c:v",
                "copy",
                "-an",
                "-f",
                "null",
                "-",
            ],
            cancel,
            deadline,
        )?;
        let video = metadata
            .lines()
            .find_map(|l| l.split_once("Video: ").map(|(_, s)| s))
            .ok_or_else(|| AppError::Internal("no video metadata".into()))?;
        let video = video
            .split(',')
            .filter(|part| !part.contains("kb/s"))
            .collect::<Vec<_>>()
            .join(",");
        let mut hash = DefaultHasher::new();
        (
            REVISION,
            std::env::consts::OS,
            &caps.version,
            path,
            video,
            jobs.clamp(1, 4),
            runtime_fingerprint(cancel, deadline),
        )
            .hash(&mut hash);
        let executable = if Path::new(path).is_file() {
            Some(std::path::PathBuf::from(path))
        } else {
            std::env::var_os("PATH").and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|dir| dir.join(path))
                    .find(|p| p.is_file())
            })
        };
        if let Some(executable) = executable {
            if let Ok(m) = std::fs::metadata(&executable) {
                (executable, m.len(), m.modified().ok()).hash(&mut hash);
            }
        }
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .hash(&mut hash);
        let key = format!("{:016x}", hash.finish());
        if let Some(mut found) = cached(&settings.read().timelapse_calibration, &key, now()) {
            found.concurrency = safe_parallelism(found.concurrency.min(jobs.max(1)));
            cache_hit = true;
            return Ok(found);
        }
        let inventory = query(path, &["-hide_banner", "-encoders"], cancel, deadline)?;
        let hw = query(path, &["-hide_banner", "-hwaccels"], cancel, deadline)?;
        let filters = query(path, &["-hide_banner", "-filters"], cancel, deadline)?;
        let mut best = baseline.clone();
        // Warm up and repeat the baseline. Use the faster result so a cold
        // startup cannot make a speculative candidate appear faster.
        let first = measure(path, source, &best, cancel, deadline)
            .ok_or_else(|| AppError::Internal("baseline not measurable within budget".into()))?;
        let second = measure(path, source, &best, cancel, deadline)
            .ok_or_else(|| AppError::Internal("baseline repeat failed".into()))?;
        let mut score = first.min(second);
        let nodes = ffmpeg::vaapi_render_nodes();
        let mut candidates = Vec::new();
        // Prefer cheap modifications to the known-working chain before probing
        // other APIs. Unsupported filters fail locally and never become jobs.
        if cfg!(windows) && baseline.profile.decoder == super::types::Decoder::D3d11 {
            for (filter, scale) in [
                ("scale_d3d11", GpuScale::D3d11),
                ("vpp_amf", GpuScale::Amf),
                ("scale_qsv", GpuScale::Qsv),
            ] {
                if filters.split_whitespace().any(|t| t == filter)
                    && (scale != GpuScale::Qsv || baseline.encoder == Encoder::HevcQsv)
                    && (scale != GpuScale::Amf || baseline.encoder == Encoder::HevcAmf)
                {
                    let mut c = baseline.clone();
                    c.profile.software_scale = false;
                    c.profile.gpu_scale = Some(scale);
                    candidates.push(c);
                }
            }
        }
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        for threads in [1, 4, 8] {
            if threads <= cores && (threads < 8 || available_memory().is_some_and(|m| m >= 4 * GIB))
            {
                let mut c = baseline.clone();
                c.filter_threads = threads;
                candidates.push(c);
            }
        }
        for encoder in [
            Encoder::HevcNvenc,
            Encoder::HevcAmf,
            Encoder::HevcQsv,
            Encoder::HevcVaapi,
            Encoder::LibX265,
        ] {
            if encoder == baseline.encoder
                || !inventory.split_whitespace().any(|t| t == encoder.as_str())
            {
                continue;
            }
            if encoder == Encoder::HevcVaapi && nodes.is_empty() {
                continue;
            }
            if encoder == Encoder::HevcQsv && !cfg!(windows) {
                continue;
            }
            let device = if encoder == Encoder::HevcVaapi {
                nodes.first().cloned()
            } else {
                None
            };
            for profile in ffmpeg::processing_profiles(
                encoder,
                &hw,
                cfg!(windows),
                cfg!(target_os = "linux"),
                cfg!(target_os = "macos"),
            ) {
                candidates.push(Choice {
                    encoder,
                    profile,
                    device: device.clone(),
                    decoder_device: device.clone().or_else(|| nodes.first().cloned()),
                    filter_threads: 2,
                    concurrency: 1,
                });
            }
        }
        for candidate in candidates {
            if Instant::now() >= deadline || cancel.load(Ordering::Relaxed) {
                break;
            }
            if let Some(t) = measure(path, source, &candidate, cancel, deadline) {
                if better(t, score) {
                    // Require confirmation, not one lucky run.
                    if let Some(repeat) = measure(path, source, &candidate, cancel, deadline) {
                        if better(t.max(repeat), score) {
                            score = t.max(repeat);
                            best = candidate;
                        }
                    }
                }
            }
        }
        for n in 2..=safe_parallelism(jobs.min(4)) {
            let mut candidate = best.clone();
            candidate.concurrency = n;
            if let Some(t) = measure(path, source, &candidate, cancel, deadline) {
                if better(t, score) {
                    if let Some(repeat) = measure(path, source, &candidate, cancel, deadline) {
                        if better(t.max(repeat), score) {
                            score = t.max(repeat);
                            best = candidate;
                        }
                    }
                }
            }
        }
        if !cancel.load(Ordering::Relaxed) {
            let record = Record {
                key,
                created: now(),
                choice: best.clone(),
            };
            let _ = settings.update(|s| {
                s.timelapse_calibration.retain(|r| {
                    r.created <= now() && now() - r.created < TTL && r.key != record.key
                });
                s.timelapse_calibration.push(record);
                if s.timelapse_calibration.len() > 8 {
                    s.timelapse_calibration.remove(0);
                }
            });
        }
        Ok(best)
    })();
    let (state, choice) = match result {
        Ok(c) => (if cache_hit { "cached" } else { "selected" }, c),
        Err(_) => ("safe", baseline),
    };
    let message = format!(
        "{}: {}",
        if state == "safe" {
            "Using safe settings (calibration unavailable)"
        } else if cache_hit {
            "Saved calibration"
        } else {
            "Selected"
        },
        choice.describe()
    );
    eprintln!("[timelapse] calibration: {message}");
    emit(Status { state, message });
    choice
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_rejects_noise_and_invalid_times() {
        assert!(!better(0.90, 1.0));
        assert!(better(0.8, 1.0));
        assert!(!better(f64::NAN, 1.0));
        assert!(!better(0.0, 1.0));
    }
    #[test]
    fn unknown_or_low_live_ram_is_sequential() {
        assert_eq!(memory_cap(4, None), 1);
        assert_eq!(memory_cap(4, Some(8 * GIB)), 1);
        assert_eq!(memory_cap(4, Some(28 * GIB)), 2);
        assert_eq!(memory_cap(99, Some(128 * GIB)), 4);
    }
    #[test]
    fn cache_expires_and_rejects_changed_runtime_or_future_clock() {
        let choice = Choice {
            encoder: Encoder::LibX265,
            profile: ProcessingProfile::SOFTWARE,
            device: None,
            decoder_device: None,
            filter_threads: 2,
            concurrency: 1,
        };
        let records = vec![Record {
            key: "runtime-a".into(),
            created: 100,
            choice: choice.clone(),
        }];
        assert_eq!(cached(&records, "runtime-a", 101), Some(choice));
        assert!(cached(&records, "runtime-b", 101).is_none());
        assert!(cached(&records, "runtime-a", 100 + TTL).is_none());
        assert!(cached(&records, "runtime-a", 99).is_none());
    }
}

#[cfg(all(test, unix))]
mod process_tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        sync::{atomic::AtomicBool, Arc},
    };
    fn fake(script: &str) -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ffmpeg");
        fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        (dir, path.to_string_lossy().into_owned())
    }
    fn caps() -> FfmpegCapabilities {
        FfmpegCapabilities {
            version: "ffmpeg fake".into(),
            nvenc_hevc: false,
            amf_hevc: false,
            qsv_hevc: false,
            vaapi_device: None,
            software_processing: true,
            decoder: super::super::types::Decoder::Software,
            decoder_device: None,
        }
    }
    #[test]
    fn cancelled_or_exhausted_calibration_never_invokes_missing_binary() {
        let dir = tempfile::tempdir().unwrap();
        let settings = AppSettingsHandle::load(dir.path());
        let cancel = Arc::new(AtomicBool::new(true));
        let caps = caps();
        let expected = Choice::baseline(&caps);
        assert_eq!(
            select(
                "missing",
                Path::new("missing"),
                &caps,
                &settings,
                &cancel,
                Instant::now() + BATCH_BUDGET,
                1,
                |_| panic!("no work after cancel")
            ),
            expected
        );
        cancel.store(false, Ordering::Relaxed);
        assert_eq!(
            select(
                "missing",
                Path::new("missing"),
                &caps,
                &settings,
                &cancel,
                Instant::now(),
                1,
                |_| panic!("no work after deadline")
            ),
            expected
        );
        assert!(settings.read().timelapse_calibration.is_empty());
    }
    #[test]
    fn benchmark_requires_output_and_real_hardware_confirmation() {
        let cancel = Arc::new(AtomicBool::new(false));
        let (_dir, binary) = fake("echo 'out_time_us=500000'; echo 'progress=end'");
        assert!(ffmpeg::benchmark(
            &binary,
            Path::new("source"),
            Encoder::LibX265,
            ProcessingProfile::SOFTWARE,
            None,
            None,
            2,
            1,
            &cancel,
            Duration::from_secs(1)
        )
        .is_ok());
        let hw = ProcessingProfile {
            decoder: super::super::types::Decoder::D3d11,
            software_scale: true,
            gpu_scale: None,
        };
        assert!(ffmpeg::benchmark(
            &binary,
            Path::new("source"),
            Encoder::HevcAmf,
            hw,
            None,
            None,
            2,
            1,
            &cancel,
            Duration::from_secs(1)
        )
        .is_err());
        let (_dir, binary) = fake("exit 0");
        assert!(ffmpeg::benchmark(
            &binary,
            Path::new("source"),
            Encoder::LibX265,
            ProcessingProfile::SOFTWARE,
            None,
            None,
            2,
            1,
            &cancel,
            Duration::from_secs(1)
        )
        .is_err());
    }
    #[test]
    fn deadline_and_cancel_reap_benchmark_child() {
        // exec avoids a shell grandchild holding the diagnostic pipes open.
        let (_dir, binary) = fake("exec sleep 20");
        let cancel = Arc::new(AtomicBool::new(false));
        let start = Instant::now();
        assert!(ffmpeg::benchmark(
            &binary,
            Path::new("source"),
            Encoder::LibX265,
            ProcessingProfile::SOFTWARE,
            None,
            None,
            2,
            1,
            &cancel,
            Duration::from_millis(80)
        )
        .is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
        let flag = cancel.clone();
        let stop = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            flag.store(true, Ordering::Relaxed);
        });
        let start = Instant::now();
        assert!(ffmpeg::benchmark(
            &binary,
            Path::new("source"),
            Encoder::LibX265,
            ProcessingProfile::SOFTWARE,
            None,
            None,
            2,
            1,
            &cancel,
            Duration::from_secs(10)
        )
        .is_err());
        stop.join().unwrap();
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    #[test]
    fn failed_baseline_does_not_cache_or_modify_media() {
        let (_dir, binary) = fake("echo 'Video: hevc, yuv420p10le, 3840x2160, 30 fps' >&2; exit 0");
        let dir = tempfile::tempdir().unwrap();
        let settings = AppSettingsHandle::load(dir.path());
        let source = dir.path().join("original.mp4");
        fs::write(&source, b"original").unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let caps = caps();
        let choice = select(
            &binary,
            &source,
            &caps,
            &settings,
            &cancel,
            Instant::now() + Duration::from_secs(2),
            1,
            |_| {},
        );
        assert_eq!(choice, Choice::baseline(&caps));
        assert_eq!(fs::read(&source).unwrap(), b"original");
        assert!(settings.read().timelapse_calibration.is_empty());
    }
    #[test]
    fn aggregate_throughput_is_normalized_per_completed_sample() {
        let (_dir, binary) = fake("sleep 0.1; echo 'out_time_us=500000'; echo 'progress=end'");
        if safe_parallelism(2) < 2 {
            return;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let mut choice = Choice::baseline(&caps());
        let single = measure(
            &binary,
            Path::new("source"),
            &choice,
            &cancel,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
        choice.concurrency = 2;
        let parallel = measure(
            &binary,
            Path::new("source"),
            &choice,
            &cancel,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
        assert!(
            parallel < single * 0.85,
            "parallel={parallel}, single={single}"
        );
    }
    #[test]
    fn successful_calibration_reuses_cache_without_more_benchmarks() {
        let (_dir, binary) = fake(
            r#"case " $* " in
          *" -progress "*) echo measured >> "${0}.calls"; case " $* " in *" -filter_complex_threads 1 "*) sleep 0.02 ;; *) sleep 0.2 ;; esac; echo 'out_time_us=500000'; echo 'progress=end' ;;
          *" -encoders "*) echo libx265 ;;
          *" -i "*) echo 'Video: hevc, yuv420p, 1920x1080, 30 fps' >&2 ;;
        esac"#,
        );
        let dir = tempfile::tempdir().unwrap();
        let settings = AppSettingsHandle::load(dir.path());
        let cancel = Arc::new(AtomicBool::new(false));
        let caps = caps();
        let choice = select(
            &binary,
            Path::new("source"),
            &caps,
            &settings,
            &cancel,
            Instant::now() + Duration::from_secs(4),
            1,
            |_| {},
        );
        assert_eq!(choice.encoder, Encoder::LibX265);
        assert_eq!(choice.filter_threads, 1, "only a repeatable measured improvement wins");
        assert_eq!(settings.read().timelapse_calibration.len(), 1);
        let count = fs::read_to_string(format!("{binary}.calls")).unwrap();
        let again = select(
            &binary,
            Path::new("source"),
            &caps,
            &settings,
            &cancel,
            Instant::now() + Duration::from_secs(4),
            1,
            |_| {},
        );
        assert_eq!(choice, again);
        assert_eq!(
            count,
            fs::read_to_string(format!("{binary}.calls")).unwrap()
        );
    }
    #[test]
    #[ignore = "requires a real FFmpeg with libx265"]
    fn real_ffmpeg_calibration_sample_produces_output_without_changing_input() {
        let binary = std::env::var("TRIPVIEWER_TEST_FFMPEG").unwrap_or_else(|_| "ffmpeg".into());
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.mp4");
        let status = ffmpeg::ffmpeg_command(&binary)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=size=320x180:rate=30:duration=1",
                "-c:v",
                "libx265",
                "-preset",
                "ultrafast",
                "-x265-params",
                "pools=1:log-level=error",
                "-an",
            ])
            .arg(&source)
            .status()
            .unwrap();
        assert!(status.success());
        let before = fs::read(&source).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(ffmpeg::benchmark(
            &binary,
            &source,
            Encoder::LibX265,
            ProcessingProfile::SOFTWARE,
            None,
            None,
            2,
            1,
            &cancel,
            Duration::from_secs(10)
        )
        .is_ok());
        assert_eq!(before, fs::read(&source).unwrap());
    }
}
