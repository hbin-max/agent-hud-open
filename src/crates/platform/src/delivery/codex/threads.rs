//! Open, archived and active Codex thread metadata.
use super::*;

/// Thread ids whose writer lock an app currently holds in `data`, skipping any
/// id for which `exclude` returns true (for example, threads already present
/// in the 24-hour snapshot) so their lock files are not probed unnecessarily.
pub fn open_threads_excluding(data: &Path, exclude: impl Fn(&str) -> bool) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(data.join("thread-writer-locks")) else {
        return Vec::new();
    };
    let mut ids = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let text = name.to_str()?;
            if text.starts_with('.') {
                return None;
            }
            let id = text.strip_suffix(".lock")?;
            (!id.is_empty() && !exclude(id) && thread_open(data, id)).then(|| id.to_owned())
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn safe_thread_id(thread: &str) -> bool {
    !thread.is_empty() && !thread.contains('/') && !thread.contains('\\') && !thread.contains("..")
}

/// Whether `thread` has an archived rollout in `data/archived_sessions`.
fn archived_in_folder(data: &Path, thread: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(data.join("archived_sessions")) else {
        return false;
    };
    let suffix = format!("-{thread}.jsonl");
    let exact = format!("{thread}.jsonl");
    entries.flatten().any(|entry| {
        entry
            .file_name()
            .to_str()
            .is_some_and(|name| name == exact || name.ends_with(&suffix))
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SqliteThread {
    Active,
    Excluded,
    Missing,
}

fn sqlite_thread_state(data: &Path, thread: &str) -> SqliteThread {
    use rusqlite::{Connection, OpenFlags, OptionalExtension};
    let db_path = data.join("state_5.sqlite");
    if !db_path.is_file() {
        return SqliteThread::Missing;
    }
    let Ok(conn) = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) else {
        return SqliteThread::Missing;
    };
    let _ = conn.busy_timeout(Duration::from_millis(50));
    let row: rusqlite::Result<Option<(i64, String, String)>> = conn
        .query_row(
            "SELECT archived, rollout_path, source FROM threads WHERE id = ?1",
            [thread],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional();
    match row {
        Ok(Some((archived, rollout_path, source))) => {
            if archived != 0 || source.trim_start().starts_with("{\"subagent\"") {
                return SqliteThread::Excluded;
            }
            if (!rollout_path.is_empty() && Path::new(&rollout_path).is_file())
                || thread_open(data, thread)
            {
                SqliteThread::Active
            } else {
                SqliteThread::Excluded
            }
        }
        Ok(None) | Err(_) => SqliteThread::Missing,
    }
}

/// Whether `thread` is known to be archived (or a sub-agent in `state_5.sqlite`).
pub fn thread_archived(data: &Path, thread: &str) -> bool {
    !safe_thread_id(thread)
        || archived_in_folder(data, thread)
        || sqlite_thread_state(data, thread) == SqliteThread::Excluded
}

/// Whether `thread` still exists as a non-archived, non-subagent Codex thread
/// (either live-locked via `thread-writer-locks` or active with an existing
/// rollout in `state_5.sqlite`). Archived and deleted threads return `false`.
pub fn thread_active(data: &Path, thread: &str) -> bool {
    if !safe_thread_id(thread) || archived_in_folder(data, thread) {
        return false;
    }
    match sqlite_thread_state(data, thread) {
        SqliteThread::Active => true,
        SqliteThread::Excluded => false,
        SqliteThread::Missing => thread_open(data, thread),
    }
}
