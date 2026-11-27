//! Optional archive-local reuse of successful, unchanged MP4 metadata.
//! No media bytes, codec failures or main-schema additions are stored here.
use crate::{error::AppError, metadata::mp4_probe, model::ChannelMeta};
use rusqlite::{params, Connection};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
    time::UNIX_EPOCH,
};

const VERSION: i64 = 1;
type Fingerprint = (i64, String);
type Entry = (String, Fingerprint, ChannelMeta);

pub(crate) struct MetadataCache {
    root: PathBuf,
    saved: HashMap<String, (Fingerprint, ChannelMeta)>,
    pending: Mutex<Vec<Entry>>,
    pub hits: AtomicUsize,
}

fn fingerprint(path: &Path) -> Option<Fingerprint> {
    let meta = path.metadata().ok()?;
    Some((
        i64::try_from(meta.len()).ok()?,
        meta.modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos()
            .to_string(),
    ))
}

impl MetadataCache {
    pub fn load(conn: &Connection, root: &Path) -> Result<Self, AppError> {
        let mut stmt = conn.prepare("SELECT file_path, file_size, modified_ns, payload_json FROM extensions.metadata_cache_v1 WHERE probe_version=?1")?;
        let rows = stmt.query_map([VERSION], |r| {
            Ok((
                r.get::<_, String>(0)?,
                (r.get::<_, i64>(1)?, r.get::<_, String>(2)?),
                r.get::<_, String>(3)?,
            ))
        })?;
        let mut saved = HashMap::new();
        for row in rows {
            let (path, stamp, json) = row?;
            if let Ok(meta) = serde_json::from_str::<ChannelMeta>(&json) {
                if meta.duration_s.is_finite() && meta.duration_s >= 0.0 && meta.fps_den > 0 {
                    saved.insert(path, (stamp, meta));
                }
            }
        }
        Ok(Self {
            root: root.to_path_buf(),
            saved,
            pending: Mutex::new(Vec::new()),
            hits: AtomicUsize::new(0),
        })
    }

    pub fn probe(&self, path: &Path) -> Result<ChannelMeta, AppError> {
        self.probe_with(path, || mp4_probe::probe(path))
    }

    fn probe_with(
        &self,
        path: &Path,
        probe: impl FnOnce() -> Result<ChannelMeta, AppError>,
    ) -> Result<ChannelMeta, AppError> {
        let key = crate::paths::to_archive_relative(path, &self.root).ok();
        let before = fingerprint(path);
        if let (Some(key), Some(stamp)) = (&key, &before) {
            if let Some((saved, meta)) = self.saved.get(key) {
                if saved == stamp {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    return Ok(meta.clone());
                }
            }
        }
        let meta = probe()?;
        // A file modified while parsing cannot establish reusable metadata.
        if let (Some(key), Some(stamp)) = (key, before) {
            if fingerprint(path).as_ref() == Some(&stamp) {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.push((key, stamp, meta.clone()));
                }
            }
        }
        Ok(meta)
    }

    pub fn flush(&self, conn: &Connection) -> Result<(), AppError> {
        let pending = self
            .pending
            .lock()
            .map_err(|_| AppError::Internal("metadata cache poisoned".into()))?;
        if pending.is_empty() {
            return Ok(());
        }
        let tx = conn.unchecked_transaction()?;
        for (path, (size, modified), meta) in pending.iter() {
            let json =
                serde_json::to_string(meta).map_err(|e| AppError::Internal(e.to_string()))?;
            tx.execute("INSERT INTO extensions.metadata_cache_v1(file_path,file_size,modified_ns,probe_version,payload_json) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(file_path) DO UPDATE SET file_size=excluded.file_size,modified_ns=excluded.modified_ns,probe_version=excluded.probe_version,payload_json=excluded.payload_json",params![path,size,modified,VERSION,json])?;
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn meta() -> ChannelMeta {
        ChannelMeta {
            duration_s: 3.0,
            width: 64,
            height: 64,
            fps_num: 30,
            fps_den: 1,
            codec: "h264".into(),
            has_gpmd_track: false,
        }
    }
    #[test]
    fn unchanged_metadata_survives_reopen_but_replaced_or_old_policy_data_does_not() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("sample.mp4");
        std::fs::write(&path, b"synthetic").unwrap();
        let db = crate::db::open(root.path()).unwrap();
        {
            let conn = db.lock().unwrap();
            let cache = MetadataCache::load(&conn, root.path()).unwrap();
            cache.probe_with(&path, || Ok(meta())).unwrap();
            cache.flush(&conn).unwrap();
        }
        drop(db);
        let db = crate::db::open(root.path()).unwrap();
        let conn = db.lock().unwrap();
        let cache = MetadataCache::load(&conn, root.path()).unwrap();
        cache
            .probe_with(&path, || panic!("unchanged media must not be parsed again"))
            .unwrap();
        assert_eq!(cache.hits.load(Ordering::Relaxed), 1);
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(
                std::fs::FileTimes::new()
                    .set_modified(UNIX_EPOCH + std::time::Duration::from_secs(123)),
            )
            .unwrap();
        assert!(cache
            .probe_with(&path, || Err(AppError::Parse("changed mtime".into())))
            .is_err());
        std::fs::write(&path, b"replacement content").unwrap();
        assert!(cache
            .probe_with(&path, || Err(AppError::Parse("fresh probe".into())))
            .is_err());
        conn.execute(
            "UPDATE extensions.metadata_cache_v1 SET probe_version=-1",
            [],
        )
        .unwrap();
        let cache = MetadataCache::load(&conn, root.path()).unwrap();
        assert!(cache.saved.is_empty());
        assert_eq!(
            conn.query_row("PRAGMA main.user_version", [], |r| r.get::<_, i32>(0))
                .unwrap(),
            13
        );
    }
    #[test]
    fn failed_or_concurrently_changed_inputs_are_not_cached() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("sample.mp4");
        std::fs::write(&path, b"initial").unwrap();
        let db = crate::db::open(root.path()).unwrap();
        let conn = db.lock().unwrap();
        let cache = MetadataCache::load(&conn, root.path()).unwrap();
        assert!(cache
            .probe_with(&path, || Err(AppError::Parse("bad".into())))
            .is_err());
        cache
            .probe_with(&path, || {
                std::fs::write(&path, b"changed while probing").unwrap();
                Ok(meta())
            })
            .unwrap();
        cache.flush(&conn).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM extensions.metadata_cache_v1",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
    }
}
