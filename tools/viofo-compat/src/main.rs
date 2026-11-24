#![allow(dead_code)]
use std::path::Path;
mod focus;

mod error {
    #[derive(Debug, thiserror::Error)]
    pub enum AppError {
        #[error("io error: {0}")] Io(#[from] std::io::Error),
        #[error("invalid filename: {0}")] InvalidFilename(String),
        #[error("parse error: {0}")] Parse(String),
        #[error("not a video: {0}")] NotVideo(String),
        #[error("internal error: {0}")] Internal(String),
    }
}
mod model {
    pub const LABEL_FRONT: &str = "Front";
    pub const LABEL_INTERIOR: &str = "Interior";
    pub const LABEL_REAR: &str = "Rear";
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GpsPoint {
        pub t_offset_s: f64, pub lat: f64, pub lon: f64,
        pub speed_mps: f64, pub heading_deg: f64,
        pub altitude_m: f64, pub fix_ok: bool,
    }
    pub struct ChannelMeta {
        pub duration_s: f64, pub width: u32, pub height: u32,
        pub fps_num: u32, pub fps_den: u32,
        pub codec: String, pub has_gpmd_track: bool,
    }
}
#[path = "../../../src-tauri/src/scan/naming.rs"]
mod original_naming;
#[path = "../../../src-tauri/src/gps/novatek.rs"]
mod original_decoder;
#[path = "../vendor/ours/filename.rs"]
mod our_naming;
mod scan { pub(crate) use crate::original_naming as naming; pub(crate) use crate::our_naming as viofo; }
#[path = "../vendor/mp4_probe.rs"]
mod original_probe;
mod metadata { pub(crate) use crate::original_probe as mp4_probe; }
mod gps {
    pub(crate) use crate::focus::validation;
    #[path = "../../vendor/ours/viofo.rs"] pub mod viofo;
    #[path = "../../vendor/ours/viofo_timing.rs"] pub mod viofo_timing;
}

fn name_json(parsed: Option<original_naming::ParsedName>) -> serde_json::Value {
    match parsed {
        Some(p) => serde_json::json!({"kind": p.camera_kind, "channel": p.channel_label,
            "group": p.group_key, "start": p.start_time.to_string()}),
        None => serde_json::Value::Null,
    }
}
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        eprintln!("Use run-compare.py; worker usage: viofo-compare <names|original|ours|ours-raw> <file>");
        std::process::exit(2);
    }
    let path = Path::new(&args[2]);
    let result = match args[1].to_str().unwrap_or("") {
        "catalog" => focus::catalog(path),
        "focus" => focus::run(path),
        "names" => {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            Ok(serde_json::json!({"original":name_json(original_naming::parse(&name).ok()),
                "ours":name_json(our_naming::parse(&name))}))
        },
        "original" => original_decoder::extract(path).map(|p| serde_json::json!({"points":p})),
        "ours" => gps::viofo::extract_video(path).map(|p| serde_json::json!({"points":p})),
        "ours-raw" => gps::viofo::extract(path).map(|p| serde_json::json!({"points":p})),
        _ => { eprintln!("Unknown mode"); std::process::exit(2); },
    };
    match result {
        Ok(value) => println!("{}", value),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); },
    }
}
