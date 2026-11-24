//! Diagnostic-only fresh decode of the 0.13.0 selection/boundary/stitching rules.
use crate::{error::AppError, model::GpsPoint};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[path = "../vendor/app013/validation.rs"] pub(crate) mod validation;
#[path = "../vendor/app013/viofo.rs"] mod viofo;
#[path = "../vendor/app013/viofo_timing.rs"] mod viofo_timing;
fn gps_timing(_: &Path, _: &str, _: Duration, _: usize) {}

#[derive(Debug, Clone, Serialize)]
struct GpsBatchItem { file_path: String, points: Vec<GpsPoint> }
#[derive(Debug, Clone, Serialize)]
enum BatchGps {
    Plain(GpsBatchItem),
    Viofo { file_path: String, timed: Vec<viofo::TimedGpsPoint> },
}
struct TripGpsSegmentSource { duration_s: f64 }
struct TripGpsSegmentUpdate { index: usize, points: Vec<GpsPoint> }
include!("../vendor/app013/pipeline_functions.rs");

#[derive(Deserialize)]
struct Manifest { groups: Vec<Vec<String>> }
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    serde_json::from_slice(&std::fs::read(path)?).map_err(|e| AppError::Parse(e.to_string()))
}
pub fn catalog(path: &Path) -> Result<Value, AppError> {
    let paths: Vec<String> = read_json(path)?;
    let items: Vec<_> = paths.iter().filter_map(|path| {
        let file = Path::new(path).file_name()?.to_str()?;
        let name = crate::our_naming::parse(file)?;
        Some(json!({"path":path,"file":file,"group":name.group_key,
            "start":name.start_time.to_string(),"channel":name.channel_label}))
    }).collect();
    Ok(json!({"items":items}))
}
fn batch_json(item: &BatchGps) -> Value {
    match item {
        BatchGps::Viofo {file_path,timed} => json!({"file":file_path,"timed":timed}),
        BatchGps::Plain(item) => json!({"file":item.file_path,"timed":[]}),
    }
}
fn replay(selected: &[BatchGps], durations: &[f64]) -> (Vec<BatchGps>, Vec<GpsPoint>) {
    let segments: Vec<_> = durations.iter().map(|d| TripGpsSegmentSource {duration_s:*d}).collect();
    let stitched = extract_trip_gps_segments::<std::convert::Infallible>(
        &segments, |i,_| Ok(selected[i].clone()), |_,_| {}).unwrap();
    // Expose per-file corrections too, using the same incremental boundary calls.
    let mut corrected = Vec::new();
    for item in selected {
        corrected.push(item.clone());
        let start=corrected.len().saturating_sub(2);
        validate_viofo_batch_boundaries(&mut corrected[start..]);
    }
    (corrected,stitched)
}
pub fn run(path: &Path) -> Result<Value, AppError> {
    let manifest: Manifest = read_json(path)?;
    if manifest.groups.len()!=3 || manifest.groups.iter().any(|g|g.is_empty() || g.len()>3) {
        return Err(AppError::Parse("Need three adjacent recording groups, each with 1–3 channels".into()));
    }
    let mut segments=Vec::new();
    let mut selected=Vec::new();
    let mut durations=Vec::new();
    for paths in manifest.groups {
        let mut candidates=Vec::new();
        let mut batches=Vec::new();
        let duration_s=crate::metadata::mp4_probe::probe(Path::new(&paths[0]))?.duration_s;
        for path in paths {
            let path_obj=PathBuf::from(&path);
            let file=path_obj.file_name().unwrap_or_default().to_string_lossy().to_string();
            let duration=crate::metadata::mp4_probe::probe(&path_obj).ok().map(|m|m.duration_s);
            let (timed,error)=match extract_viofo_timed(&path_obj) {
                Ok(timed)=>(timed,None), Err(e)=>(vec![],Some(e.to_string())),
            };
            // Do not emit full paths or raw error strings in the IPC report.
            let batch=BatchGps::Viofo {file_path:file.clone(),timed};
            candidates.push(json!({"file":file,"duration_s":duration,
                "decode_ok":error.is_none(),"quality":batch_quality(&batch,duration_s),
                "timed":match &batch {BatchGps::Viofo {timed,..}=>timed,_=>unreachable!()}}));
            batches.push(batch);
        }
        let best=best_segment_gps(batches,duration_s);
        segments.push(json!({"duration_s":duration_s,"candidates":candidates,
            "selected_before_boundary":batch_json(&best)}));
        durations.push(duration_s);
        selected.push(best);
    }
    let (corrected,stitched)=replay(&selected,&durations);
    for (segment,after) in segments.iter_mut().zip(corrected) {
        segment["selected_after_boundary"]=batch_json(&after);
    }
    Ok(json!({"segments":segments,"stitched":stitched,
        "scope":"Fresh decode of available candidates; exact 0.13.0 selection, incremental boundary and stitch functions. No DB cache, media-health exclusions or playback."}))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn timed(ms:i64,lon:f64)->viofo::TimedGpsPoint {
        viofo::TimedGpsPoint {utc:chrono::DateTime::from_timestamp_millis(ms).unwrap().naive_utc(),
            point:GpsPoint {t_offset_s:ms as f64/1000.,lat:0.,lon,speed_mps:11.12,
                heading_deg:90.,altitude_m:0.,fix_ok:true}}
    }
    fn batch(file:&str,points:Vec<viofo::TimedGpsPoint>)->BatchGps {
        BatchGps::Viofo {file_path:file.into(),timed:points}
    }
    #[test]
    fn selects_by_usable_count_then_span_with_stable_ties() {
        let a=batch("front",vec![timed(0,0.)]);
        let b=batch("rear",vec![timed(0,0.),timed(1000,0.0001)]);
        assert_eq!(batch_json(&best_segment_gps([a,b],2.))["file"],"rear");
        let a=batch("front",vec![timed(0,0.),timed(1000,0.0001)]);
        let b=batch("rear",vec![timed(0,0.),timed(1000,0.0001)]);
        assert_eq!(batch_json(&best_segment_gps([a,b],2.))["file"],"front");
    }
    #[test]
    fn replay_removes_initial_outlier_and_stitches_corrected_points() {
        let left=batch("left",vec![timed(0,0.),timed(1000,0.0001)]);
        let mut middle_points=vec![timed(2000,20.),timed(3000,0.0003),timed(4000,0.0004)];
        for p in &mut middle_points {p.point.t_offset_s-=2.;}
        let middle=batch("middle",middle_points);
        let (corrected,stitched)=replay(&[left,middle],&[2.,3.]);
        assert_eq!(batch_json(&corrected[1])["timed"].as_array().unwrap().len(),2);
        assert_eq!(stitched.len(),4);
        assert!(stitched.iter().all(|p|p.lon<1.));
    }
    #[test]
    fn replay_preserves_real_utc_pause_without_using_it_as_outlier_proof() {
        let left=batch("left",vec![timed(0,0.),timed(1000,0.0001)]);
        let mut points=vec![timed(60000,20.),timed(61000,0.0003)];
        for p in &mut points {p.point.t_offset_s-=60.;}
        let right=batch("right",points);
        let (corrected,_)=replay(&[left,right],&[2.,2.]);
        assert_eq!(batch_json(&corrected[1])["timed"].as_array().unwrap().len(),2);
    }
}
