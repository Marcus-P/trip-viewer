pub(crate) mod cache;
pub mod evidence;
pub mod mp4_probe;

use crate::error::AppError;
use crate::model::ChannelMeta;
use std::path::Path;

#[tauri::command]
pub async fn probe_file(path: String) -> Result<ChannelMeta, AppError> {
    mp4_probe::probe(Path::new(&path))
}

/// Explicit diagnostic access to existing header evidence. Source paths are
/// input only; no GPS positions, recording dates or file paths are returned.
#[tauri::command]
pub async fn inspect_media_evidence(
    path: String,
) -> Result<Vec<evidence::TrackEvidence>, AppError> {
    let reader = mp4::read_mp4(std::fs::File::open(path)?)
        .map_err(|error| AppError::Parse(error.to_string()))?;
    let fragmented = !reader.moofs.is_empty();
    let mut tracks: Vec<_> = reader
        .tracks()
        .values()
        .map(|track| evidence::inspect(track, fragmented))
        .collect();
    tracks.sort_by_key(|track| track.track_id);
    Ok(tracks)
}
