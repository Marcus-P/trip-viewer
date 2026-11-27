//! Fork-only archive data lives beside the upstream-compatible database.
//! Neither bookmarks nor reusable analysis may raise main.user_version.
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::Path;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS extensions.media_health_ranges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL, file_size INTEGER NOT NULL, modified_ms INTEGER NOT NULL,
    start_ms INTEGER NOT NULL, end_ms INTEGER, error_kind TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS extensions.idx_media_health_lookup
    ON media_health_ranges(file_path, file_size, modified_ms, start_ms);
CREATE TABLE IF NOT EXISTS extensions.gps_segment_checkpoints (
    trip_id TEXT NOT NULL, segment_id TEXT NOT NULL, source_signature TEXT NOT NULL,
    parser_version INTEGER NOT NULL, payload_json TEXT NOT NULL,
    PRIMARY KEY (trip_id, segment_id)
);
CREATE TABLE IF NOT EXISTS extensions.trip_bookmarks (
    trip_id TEXT NOT NULL, time_ms INTEGER NOT NULL CHECK(time_ms >= 0),
    PRIMARY KEY (trip_id, time_ms)
);
CREATE TABLE IF NOT EXISTS extensions.trip_gps_provenance (
    trip_id TEXT PRIMARY KEY, source_signature TEXT NOT NULL,
    payload_hash TEXT NOT NULL, parser_version INTEGER NOT NULL
);
PRAGMA extensions.user_version = 1;
"#;

pub(super) fn prepare(conn: &mut Connection, path: Option<&Path>) -> Result<(), AppError> {
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    let version: i32 = conn.query_row("PRAGMA main.user_version", [], |r| r.get(0))?;
    if version > super::migrations::HEAD_VERSION as i32 {
        // Keep other writers out between the durable extra copy and the main
        // conversion. Contention is bounded by busy_timeout above.
        conn.pragma_update(None, "locking_mode", "EXCLUSIVE")?;
        conn.execute_batch("BEGIN EXCLUSIVE; COMMIT;")?;
        verify_legacy_schema(conn, version)?;
        // VACUUM INTO includes committed WAL contents. Never copy a live .db
        // file alone, overwrite a prior backup, or delete the recovery copy.
        if let Some(path) = path {
            let stem = format!("tripviewer-schema-{version}-before-compat");
            let backup = (0..10_000)
                .map(|index| {
                    path.with_file_name(if index == 0 {
                        format!("{stem}.sqlite")
                    } else {
                        format!("{stem}-{index}.sqlite")
                    })
                })
                .find(|candidate| !candidate.exists())
                .ok_or_else(|| AppError::Internal("no unused archive backup filename".into()))?;
            conn.execute("VACUUM main INTO ?1", [backup.to_string_lossy().as_ref()])?;
            let saved =
                Connection::open_with_flags(&backup, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            let check: String = saved.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
            if check != "ok" {
                return Err(AppError::Internal(
                    "archive backup failed integrity validation".into(),
                ));
            }
        }
    }
    let extras = path.map(|p| p.with_file_name("archive-extras-v1.sqlite"));
    let file = extras.as_ref().map(|p| p.to_string_lossy());
    conn.execute(
        "ATTACH DATABASE ?1 AS extensions",
        [file.as_deref().unwrap_or(":memory:")],
    )?;
    let extra_version: i32 = conn.query_row("PRAGMA extensions.user_version", [], |r| r.get(0))?;
    if extra_version > 1 {
        return Err(AppError::ArchiveSchemaTooNew {
            found: extra_version,
            expected: 1,
        });
    }
    conn.execute_batch(SCHEMA)?;
    if version > super::migrations::HEAD_VERSION as i32 {
        // Persist the complete extra data first. If conversion of main fails,
        // main remains at its original version and this copy is safe to retry.
        let tx = conn.transaction()?;
        if version >= 14 {
            tx.execute_batch("INSERT INTO extensions.media_health_ranges(file_path,file_size,modified_ms,start_ms,end_ms,error_kind,updated_at_ms)
SELECT file_path,file_size,modified_ms,start_ms,end_ms,error_kind,updated_at_ms FROM main.media_health_ranges AS old
WHERE NOT EXISTS (SELECT 1 FROM extensions.media_health_ranges AS saved WHERE saved.file_path=old.file_path
AND saved.file_size=old.file_size AND saved.modified_ms=old.modified_ms AND saved.start_ms=old.start_ms
AND saved.end_ms IS old.end_ms AND saved.error_kind=old.error_kind AND saved.updated_at_ms=old.updated_at_ms);")?;
        }
        if version >= 15 {
            let records = {
                let mut stmt = tx.prepare("SELECT trip_id, source_signature FROM main.trip_gps WHERE source_signature IS NOT NULL")?;
                let rows =
                    stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
                rows.collect::<Result<Vec<_>, _>>()?
            };
            for (trip, signature) in records {
                set_source_signature(&tx, &trip, &signature)?;
            }
        }
        if version >= 16 {
            tx.execute_batch("INSERT OR REPLACE INTO extensions.gps_segment_checkpoints SELECT * FROM main.gps_segment_checkpoints;")?;
        }
        if version >= 17 {
            tx.execute_batch("INSERT OR IGNORE INTO extensions.trip_bookmarks SELECT * FROM main.trip_bookmarks;")?;
        }
        tx.commit()?;
        let tx = conn.transaction()?;
        if version >= 17 {
            tx.execute_batch("DROP TABLE main.trip_bookmarks;")?;
        }
        if version >= 16 {
            tx.execute_batch("DROP TABLE main.gps_segment_checkpoints;")?;
        }
        if version >= 15 {
            tx.execute_batch("ALTER TABLE main.trip_gps DROP COLUMN source_signature;")?;
        }
        if version >= 14 {
            tx.execute_batch("DROP TABLE main.media_health_ranges;")?;
        }
        tx.pragma_update(None, "user_version", 13)?;
        let violation = tx
            .query_row("PRAGMA main.foreign_key_check", [], |_| Ok(()))
            .optional()?;
        if violation.is_some() {
            return Err(AppError::Internal(
                "archive conversion failed foreign-key validation".into(),
            ));
        }
        tx.commit()?;
    } else {
        super::migrations::apply(conn)?;
    }
    // TEMP triggers do not alter main.sqlite_master. Old applications ignore
    // the extra file; prune their deleted trips when this application reopens.
    conn.execute_batch(
        r#"
DELETE FROM extensions.trip_bookmarks WHERE trip_id NOT IN (SELECT id FROM main.trips);
DELETE FROM extensions.gps_segment_checkpoints WHERE trip_id NOT IN (SELECT id FROM main.trips);
DELETE FROM extensions.trip_gps_provenance WHERE trip_id NOT IN (SELECT trip_id FROM main.trip_gps);
CREATE TEMP TRIGGER extras_delete_trip AFTER DELETE ON main.trips BEGIN
    DELETE FROM trip_bookmarks WHERE trip_id = OLD.id;
    DELETE FROM gps_segment_checkpoints WHERE trip_id = OLD.id;
    DELETE FROM trip_gps_provenance WHERE trip_id = OLD.id;
END;
CREATE TEMP TRIGGER extras_update_trip AFTER UPDATE OF id ON main.trips BEGIN
    UPDATE trip_bookmarks SET trip_id = NEW.id WHERE trip_id = OLD.id;
    UPDATE gps_segment_checkpoints SET trip_id = NEW.id WHERE trip_id = OLD.id;
    UPDATE trip_gps_provenance SET trip_id = NEW.id WHERE trip_id = OLD.id;
END;
"#,
    )?;
    if version > super::migrations::HEAD_VERSION as i32 {
        conn.pragma_update(None, "locking_mode", "NORMAL")?;
        conn.execute_batch("BEGIN; SELECT 1 FROM main.trips LIMIT 1; COMMIT;")?;
    }
    Ok(())
}

/// Refuse an unknown schema rather than stripping columns on the assumption
/// that every database numbered 14..17 has the published fork layout.
fn verify_legacy_schema(conn: &Connection, version: i32) -> Result<(), AppError> {
    let mut expected = Connection::open_in_memory()?;
    super::migrations::apply(&mut expected)?;
    let migrations = [
        include_str!("migrations/0014_media_health_ranges.sql"),
        include_str!("migrations/0015_trip_gps_source_signature.sql"),
        include_str!("migrations/0016_gps_segment_checkpoints.sql"),
        include_str!("migrations/0017_trip_bookmarks.sql"),
    ];
    for sql in migrations.iter().take((version - 13) as usize) {
        expected.execute_batch(sql)?;
    }
    if schema(conn)? != schema(&expected)? {
        return Err(AppError::Internal(
            "archive schema differs from the known fork layout; no conversion performed".into(),
        ));
    }
    Ok(())
}

fn schema(conn: &Connection) -> Result<Vec<(String, String, String)>, AppError> {
    let mut stmt = conn.prepare("SELECT type, name, sql FROM main.sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get::<_, String>(2)?
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
        ))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn payload_identity(conn: &Connection, trip: &str) -> Result<Option<(String, i32)>, AppError> {
    let row: Option<(String, i32)> = conn
        .query_row(
            "SELECT points_json, parser_version FROM main.trip_gps WHERE trip_id = ?1",
            [trip],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(row.map(|(json, parser)| (format!("{:x}", Sha256::digest(json.as_bytes())), parser)))
}

pub(super) fn source_signature(conn: &Connection, trip: &str) -> Result<Option<String>, AppError> {
    let Some((hash, parser)) = payload_identity(conn, trip)? else {
        return Ok(None);
    };
    Ok(conn.query_row(
        "SELECT source_signature FROM extensions.trip_gps_provenance WHERE trip_id = ?1 AND payload_hash = ?2 AND parser_version = ?3",
        params![trip, hash, parser], |r| r.get(0),
    ).optional()?)
}

pub(super) fn set_source_signature(
    conn: &Connection,
    trip: &str,
    signature: &str,
) -> Result<(), AppError> {
    let Some((hash, parser)) = payload_identity(conn, trip)? else {
        return Ok(());
    };
    conn.execute(
        "INSERT INTO extensions.trip_gps_provenance(trip_id, source_signature, payload_hash, parser_version)
         VALUES (?1, ?2, ?3, ?4) ON CONFLICT(trip_id) DO UPDATE SET source_signature=excluded.source_signature,
         payload_hash=excluded.payload_hash, parser_version=excluded.parser_version",
        params![trip, signature, hash, parser],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy(path: &Path, version: i32) -> Connection {
        let mut conn = Connection::open(path).unwrap();
        super::super::migrations::apply(&mut conn).unwrap();
        for sql in [
            include_str!("migrations/0014_media_health_ranges.sql"),
            include_str!("migrations/0015_trip_gps_source_signature.sql"),
            include_str!("migrations/0016_gps_segment_checkpoints.sql"),
            include_str!("migrations/0017_trip_bookmarks.sql"),
        ]
        .iter()
        .take((version - 13) as usize)
        {
            conn.execute_batch(sql).unwrap();
        }
        conn.pragma_update(None, "user_version", version).unwrap();
        conn.execute_batch("INSERT INTO trips(id,start_time_ms,end_time_ms,camera_kind,gps_supported,last_seen_ms) VALUES('trip',0,1000,'generic',1,0);
INSERT INTO trip_gps(trip_id,points_json,point_count,parser_version,created_at_ms) VALUES('trip','[]',0,8,1);
INSERT INTO places(name,lat,lon,radius_m,created_ms) VALUES('synthetic',0,0,100,1);
INSERT INTO manual_trip_merges VALUES('absorbed','trip',1);
INSERT INTO timelapse_jobs(trip_id,tier,channel,status,output_path,created_at_ms) VALUES('trip','8x','F','done','timelapse/ready.mp4',1);").unwrap();
        conn
    }

    #[test]
    fn published_schemas_convert_without_losing_data_and_reopen_in_schema_13() {
        for version in 14..=17 {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("tripviewer.db");
            let conn = legacy(&path, version);
            conn.execute_batch("INSERT INTO media_health_ranges(file_path,file_size,modified_ms,start_ms,error_kind,updated_at_ms) VALUES('clip.mp4',1,1,0,'decode',1);").unwrap();
            if version >= 15 {
                conn.execute_batch("UPDATE trip_gps SET source_signature='source';")
                    .unwrap();
            }
            if version >= 16 {
                conn.execute_batch(
                    "INSERT INTO gps_segment_checkpoints VALUES('trip','segment','source',8,'{}');",
                )
                .unwrap();
            }
            if version >= 17 {
                conn.execute_batch("INSERT INTO trip_bookmarks VALUES('trip',500);")
                    .unwrap();
            }
            drop(conn);
            let db = crate::db::open_at_path(&path, dir.path()).unwrap();
            {
                let conn = db.lock().unwrap();
                assert_eq!(
                    conn.query_row("PRAGMA main.user_version", [], |r| r.get::<_, i32>(0))
                        .unwrap(),
                    13
                );
                assert_eq!(
                    conn.query_row(
                        "SELECT COUNT(*) FROM extensions.media_health_ranges",
                        [],
                        |r| r.get::<_, i64>(0)
                    )
                    .unwrap(),
                    1
                );
                if version >= 15 {
                    assert_eq!(
                        source_signature(&conn, "trip").unwrap().as_deref(),
                        Some("source")
                    );
                }
                if version >= 16 {
                    assert_eq!(
                        conn.query_row(
                            "SELECT COUNT(*) FROM extensions.gps_segment_checkpoints",
                            [],
                            |r| r.get::<_, i64>(0)
                        )
                        .unwrap(),
                        1
                    );
                }
                if version >= 17 {
                    assert_eq!(
                        conn.query_row("SELECT time_ms FROM extensions.trip_bookmarks", [], |r| {
                            r.get::<_, i64>(0)
                        })
                        .unwrap(),
                        500
                    );
                }
                assert_eq!(
                    conn.query_row("SELECT points_json FROM main.trip_gps", [], |r| r
                        .get::<_, String>(0))
                        .unwrap(),
                    "[]"
                );
                assert_eq!(conn.query_row("SELECT COUNT(*) FROM main.sqlite_master WHERE name IN ('trip_bookmarks','gps_segment_checkpoints','media_health_ranges')", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
            }
            drop(db);
            // Reproduce a published 0.13 opener upgrading the database again.
            // Re-conversion must merge extras, not reset their identifiers.
            if version == 17 {
                let old = Connection::open(&path).unwrap();
                for sql in [
                    include_str!("migrations/0014_media_health_ranges.sql"),
                    include_str!("migrations/0015_trip_gps_source_signature.sql"),
                    include_str!("migrations/0016_gps_segment_checkpoints.sql"),
                    include_str!("migrations/0017_trip_bookmarks.sql"),
                ] {
                    old.execute_batch(sql).unwrap();
                }
                old.pragma_update(None, "user_version", 17).unwrap();
                old.execute_batch("INSERT INTO media_health_ranges(file_path,file_size,modified_ms,start_ms,error_kind,updated_at_ms) VALUES('other.mp4',1,1,0,'decode',1);").unwrap();
                drop(old);
                let again = crate::db::open_at_path(&path, dir.path()).unwrap();
                assert_eq!(
                    again
                        .lock()
                        .unwrap()
                        .query_row(
                            "SELECT COUNT(*) FROM extensions.media_health_ranges",
                            [],
                            |r| r.get::<_, i64>(0)
                        )
                        .unwrap(),
                    2
                );
            }
            let backup = Connection::open(
                path.with_file_name(format!("tripviewer-schema-{version}-before-compat.sqlite")),
            )
            .unwrap();
            assert_eq!(
                backup
                    .query_row("PRAGMA user_version", [], |r| r.get::<_, i32>(0))
                    .unwrap(),
                version
            );
            // Exercise the old application's migration entry and GPS writer,
            // without attaching or even knowing about fork extensions.
            let mut old = Connection::open(&path).unwrap();
            crate::db::migrations::apply(&mut old).unwrap();
            assert_eq!(
                old.query_row("SELECT name FROM places", [], |r| r.get::<_, String>(0))
                    .unwrap(),
                "synthetic"
            );
            assert_eq!(
                old.query_row("SELECT primary_trip_id FROM manual_trip_merges", [], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap(),
                "trip"
            );
            assert_eq!(
                old.query_row("SELECT output_path FROM timelapse_jobs", [], |r| r
                    .get::<_, String>(0))
                    .unwrap(),
                "timelapse/ready.mp4"
            );
            old.execute(
                "UPDATE trip_gps SET points_json='[{}]' WHERE trip_id='trip'",
                [],
            )
            .unwrap();
            drop(old);
            let db = crate::db::open_at_path(&path, dir.path()).unwrap();
            let conn = db.lock().unwrap();
            assert_eq!(
                source_signature(&conn, "trip").unwrap(),
                None,
                "an older writer invalidates provenance"
            );
            assert_eq!(
                conn.query_row(
                    "SELECT COUNT(*) FROM extensions.media_health_ranges",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
                if version == 17 { 2 } else { 1 }
            );
        }
    }

    #[test]
    fn unknown_layout_and_newer_versions_are_not_converted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tripviewer.db");
        let conn = legacy(&path, 17);
        conn.execute_batch("CREATE TABLE unknown_feature(secret TEXT);")
            .unwrap();
        drop(conn);
        assert!(crate::db::open_at_path(&path, dir.path()).is_err());
        let conn = Connection::open(&path).unwrap();
        assert_eq!(
            conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i32>(0))
                .unwrap(),
            17
        );
        conn.pragma_update(None, "user_version", 18).unwrap();
        drop(conn);
        assert!(crate::db::open_at_path(&path, dir.path()).is_err());
        assert!(!path.with_file_name("archive-extras-v1.sqlite").exists());
    }

    #[test]
    fn failed_validation_rolls_back_main_and_leaves_recoverable_extras() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tripviewer.db");
        let conn = legacy(&path, 17);
        conn.pragma_update(None, "foreign_keys", "OFF").unwrap();
        conn.execute_batch("INSERT INTO trip_gps(trip_id,points_json,point_count,parser_version,created_at_ms) VALUES('missing','[]',0,8,1);").unwrap();
        drop(conn);
        assert!(crate::db::open_at_path(&path, dir.path()).is_err());
        let conn = Connection::open(&path).unwrap();
        assert_eq!(
            conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i32>(0))
                .unwrap(),
            17
        );
        assert!(conn
            .prepare("SELECT source_signature FROM trip_gps")
            .is_ok());
        assert!(path
            .with_file_name("tripviewer-schema-17-before-compat.sqlite")
            .exists());
    }
}
