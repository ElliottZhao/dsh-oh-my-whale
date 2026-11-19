//! 单文件持久化：Core 的全部长期状态存放在一个 SQLite 数据文件里。
//!
//! 数据文件是唯一持久化产物；SQLite 的临时 journal 只在一次写入事务期间存在。
//! 表结构版本记在 `PRAGMA user_version`，由 [`SCHEMA_VERSION`] 与下面的 `migrate` 管理。

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{Connection, params};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::protocol::{CreateObservationRequest, Observation};

/// 当前数据文件的 schema 版本。任何表结构变更都必须加一，并在 `migrate` 里补迁移。
pub const SCHEMA_VERSION: u32 = 2;

/// 数据目录里唯一的持久化文件。
pub const DATA_FILE_NAME: &str = "core.db";

/// 一次查询 Observation 的默认条数。
pub const DEFAULT_OBSERVATION_LIMIT: u32 = 100;

/// 一次查询 Observation 允许的最大条数。
pub const MAX_OBSERVATION_LIMIT: u32 = 1000;

/// 打开或读写数据文件时可能出现的、需要给人看的错误。
#[derive(Debug)]
pub enum StateError {
    CreateDataDir {
        path: PathBuf,
        source: std::io::Error,
    },
    DataDirNotDirectory {
        path: PathBuf,
    },
    Open {
        path: PathBuf,
        source: rusqlite::Error,
    },
    SchemaTooNew {
        path: PathBuf,
        found: u32,
        supported: u32,
    },
    Corrupt {
        path: PathBuf,
        source: rusqlite::Error,
    },
    Locked {
        path: PathBuf,
        source: rusqlite::Error,
    },
    Query {
        path: PathBuf,
        source: rusqlite::Error,
    },
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::CreateDataDir { path, source } => write!(
                f,
                "无法创建数据目录 {}：{source}。请用 --data-dir 指定一个可写目录。",
                path.display()
            ),
            StateError::DataDirNotDirectory { path } => write!(
                f,
                "数据目录 {} 已存在但不是目录。请换一个路径，或先移走这个文件。",
                path.display()
            ),
            StateError::Open { path, source } => write!(
                f,
                "无法打开数据文件 {}：{source}。",
                path.display()
            ),
            StateError::SchemaTooNew {
                path,
                found,
                supported,
            } => write!(
                f,
                "数据文件 {} 的 schema 版本是 {found}，高于当前 Core 支持的 {supported}。请升级 Core，或换一个数据目录。",
                path.display()
            ),
            StateError::Corrupt { path, source } => write!(
                f,
                "数据文件 {} 无法读取（可能已损坏）：{source}。请从备份恢复，或换一个数据目录。",
                path.display()
            ),
            StateError::Locked { path, source } => write!(
                f,
                "数据文件 {} 正被另一个进程占用：{source}。请稍后重试。",
                path.display()
            ),
            StateError::Query { path, source } => write!(
                f,
                "数据文件 {} 的读写失败：{source}。",
                path.display()
            ),
        }
    }
}

impl std::error::Error for StateError {}

/// 持有数据文件连接与本次进程的启动时间。连接在进程内串行使用，读多写少。
#[derive(Debug)]
pub struct StateStore {
    conn: Mutex<Connection>,
    path: PathBuf,
    started_at: String,
}

impl StateStore {
    /// 打开（必要时创建）数据目录里的唯一数据文件，并把它迁移到当前 schema。
    ///
    /// 同一个数据文件可以被重复打开；已有的数据保持不变，旧版本的数据文件会被迁移。
    pub fn open(data_dir: &Path) -> Result<Self, StateError> {
        // 先区分「路径被一个普通文件占了」与「目录不可写」，两者给用户的下一步不同。
        if data_dir.exists() && !data_dir.is_dir() {
            return Err(StateError::DataDirNotDirectory {
                path: data_dir.to_path_buf(),
            });
        }
        std::fs::create_dir_all(data_dir).map_err(|source| StateError::CreateDataDir {
            path: data_dir.to_path_buf(),
            source,
        })?;

        let path = data_dir.join(DATA_FILE_NAME);
        let conn = Connection::open(&path).map_err(|source| classify_open_error(&path, source))?;
        conn.busy_timeout(Duration::from_secs(5))
            .map_err(|source| StateError::Open {
                path: path.clone(),
                source,
            })?;

        let found = user_version(&conn).map_err(|source| classify_open_error(&path, source))?;
        if found > SCHEMA_VERSION {
            return Err(StateError::SchemaTooNew {
                path,
                found,
                supported: SCHEMA_VERSION,
            });
        }
        if found < SCHEMA_VERSION {
            migrate(&conn, found).map_err(|source| classify_open_error(&path, source))?;
        }

        Ok(Self {
            conn: Mutex::new(conn),
            path,
            started_at: now_rfc3339(),
        })
    }

    /// 唯一数据文件的路径。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 本次进程启动的时间。
    pub fn started_at(&self) -> &str {
        &self.started_at
    }

    /// 写入一条 Observation。`occurredAt` 由请求给出；省略时用摄入时刻补齐。
    pub fn insert_observation(
        &self,
        request: CreateObservationRequest,
    ) -> Result<Observation, StateError> {
        let occurred_at = request.occurred_at.clone().unwrap_or_else(now_rfc3339);
        let conn = self.conn();
        conn.execute(
            "INSERT INTO observations (session_id, source, speaker, content, occurred_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                request.session_id,
                request.source,
                request.speaker,
                request.content,
                occurred_at
            ],
        )
        .map_err(|source| StateError::Query {
            path: self.path.clone(),
            source,
        })?;
        let id = conn.last_insert_rowid();

        Ok(Observation {
            id,
            session_id: request.session_id,
            source: request.source,
            speaker: request.speaker,
            content: request.content,
            occurred_at,
        })
    }

    /// 按会话读取 Observation，按写入顺序返回。
    ///
    /// `session_id` 为 `None` 时读**最近**的 `limit` 条（含界内的最早一条起按写入顺序）。
    pub fn observations(
        &self,
        session_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Observation>, StateError> {
        let limit = limit.clamp(1, MAX_OBSERVATION_LIMIT);
        let conn = self.conn();
        // 两条语句分开写：`?1 IS NULL OR …` 会让 SQLite 放弃按会话的索引。
        // 先按 id 倒序取最近 limit 条，再翻回写入顺序返回，调用方拿到的时间线仍是正向的。
        let sql = if session_id.is_some() {
            "SELECT id, session_id, source, speaker, content, occurred_at
             FROM observations
             WHERE session_id = ?1
             ORDER BY id DESC
             LIMIT ?2"
        } else {
            "SELECT id, session_id, source, speaker, content, occurred_at
             FROM observations
             ORDER BY id DESC
             LIMIT ?1"
        };
        let mut statement = conn.prepare(sql).map_err(|source| StateError::Query {
            path: self.path.clone(),
            source,
        })?;

        let mapped = if session_id.is_some() {
            statement
                .query_map(params![session_id, limit], map_observation)
                .map_err(|source| StateError::Query {
                    path: self.path.clone(),
                    source,
                })?
        } else {
            statement
                .query_map(params![limit], map_observation)
                .map_err(|source| StateError::Query {
                    path: self.path.clone(),
                    source,
                })?
        };

        let mut observations = Vec::new();
        for row in mapped {
            observations.push(row.map_err(|source| StateError::Query {
                path: self.path.clone(),
                source,
            })?);
        }
        observations.reverse();
        Ok(observations)
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// SQLite 不会拒绝比自己新的 `user_version`，所以迁移由这里显式拦住。
fn user_version(conn: &Connection) -> rusqlite::Result<u32> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
}

fn table_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// 从 `from` 逐版本迁移到 [`SCHEMA_VERSION`]。
///
/// v1 的 `meta` 表只存启动计数与文件创建时间，这两项已不再属于数据模型。
/// 迁移不丢旧数据：把 `meta` 改名成归档表，内容仍可读，只是不再有模块写它。
fn migrate(conn: &Connection, from: u32) -> rusqlite::Result<()> {
    if from < 2 {
        if table_exists(conn, "meta")? {
            conn.execute_batch("ALTER TABLE meta RENAME TO meta_v1_archive;")?;
        }
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS observations (
                 id          INTEGER PRIMARY KEY AUTOINCREMENT,
                 session_id  TEXT NOT NULL,
                 source      TEXT NOT NULL,
                 speaker     TEXT,
                 content     TEXT NOT NULL,
                 occurred_at TEXT NOT NULL
             ) STRICT;
             CREATE INDEX IF NOT EXISTS observations_by_session
                 ON observations (session_id, id);",
        )?;
    }
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

/// 把 rusqlite 的底层错误翻译成用户能据此行动的 [`StateError`]。
fn classify_open_error(path: &Path, source: rusqlite::Error) -> StateError {
    match &source {
        rusqlite::Error::SqliteFailure(err, _)
            if err.code == rusqlite::ErrorCode::NotADatabase =>
        {
            StateError::Corrupt {
                path: path.to_path_buf(),
                source,
            }
        }
        rusqlite::Error::SqliteFailure(err, _)
            if err.code == rusqlite::ErrorCode::DatabaseBusy
                || err.code == rusqlite::ErrorCode::DatabaseLocked =>
        {
            StateError::Locked {
                path: path.to_path_buf(),
                source,
            }
        }
        _ => StateError::Open {
            path: path.to_path_buf(),
            source,
        },
    }
}

fn now_rfc3339() -> String {
    // RFC3339 是固定无损格式，除了年份越界（now 不可能）之外不会失败。
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC3339 格式化当前时间不应失败")
}

/// 一行 observations 表 → [`Observation`]。
fn map_observation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Observation> {
    Ok(Observation {
        id: row.get(0)?,
        session_id: row.get(1)?,
        source: row.get(2)?,
        speaker: row.get(3)?,
        content: row.get(4)?,
        occurred_at: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 每个测试一个一次性目录；不引入额外的临时目录依赖。
    struct TempDataDir {
        path: PathBuf,
    }

    impl TempDataDir {
        fn new(label: &str) -> Self {
            let unique = format!(
                "omw-core-test-{}-{}-{label}",
                std::process::id(),
                OffsetDateTime::now_utc().unix_timestamp_nanos()
            );
            let path = std::env::temp_dir().join(unique);
            std::fs::create_dir_all(&path).expect("创建测试目录");
            Self { path }
        }
    }

    impl Drop for TempDataDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("读取数据目录")
            .map(|entry| entry.expect("目录项").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn sample(session_id: &str, content: &str) -> CreateObservationRequest {
        CreateObservationRequest {
            session_id: session_id.to_owned(),
            source: "host.message".to_owned(),
            speaker: Some("user".to_owned()),
            content: content.to_owned(),
            occurred_at: None,
        }
    }

    #[test]
    fn opening_creates_exactly_one_data_file() {
        let dir = TempDataDir::new("single-file");
        let store = StateStore::open(&dir.path).expect("打开数据文件");

        assert_eq!(entries(&dir.path), vec![DATA_FILE_NAME.to_owned()]);
        assert_eq!(store.path(), dir.path.join(DATA_FILE_NAME));
    }

    #[test]
    fn observations_survive_reopening() {
        let dir = TempDataDir::new("reopen");

        let first = StateStore::open(&dir.path).expect("第一次打开");
        let written = first
            .insert_observation(sample("s1", "我偏好简洁的回答"))
            .expect("写入 Observation");
        drop(first);

        let second = StateStore::open(&dir.path).expect("第二次打开");
        let read = second.observations(Some("s1"), 10).expect("读取 Observation");

        assert_eq!(read.len(), 1, "重启后应读回同一条 Observation");
        assert_eq!(read[0].id, written.id);
        assert_eq!(read[0].content, "我偏好简洁的回答");
        assert_eq!(read[0].session_id, "s1");
        assert_eq!(read[0].speaker.as_deref(), Some("user"));
        assert_eq!(entries(&dir.path), vec![DATA_FILE_NAME.to_owned()]);
    }

    #[test]
    fn observations_are_filtered_by_session() {
        let dir = TempDataDir::new("sessions");
        let store = StateStore::open(&dir.path).expect("打开数据文件");
        store.insert_observation(sample("a", "第一条")).expect("写入 a");
        store.insert_observation(sample("b", "第二条")).expect("写入 b");

        let only_a = store.observations(Some("a"), 10).expect("读取 a");
        assert_eq!(only_a.len(), 1);
        assert_eq!(only_a[0].content, "第一条");

        let all = store.observations(None, 10).expect("读取全部");
        assert_eq!(all.len(), 2, "不按会话过滤时应读回全部");
    }

    #[test]
    fn schema_v1_file_is_migrated_and_observations_start_working() {
        let dir = TempDataDir::new("migrate-v1");
        let path = dir.path.join(DATA_FILE_NAME);
        {
            // 手工造一个 v1 数据文件：只有 meta 表，版本号 1。
            let conn = Connection::open(&path).expect("建 v1 库");
            conn.execute_batch(
                "CREATE TABLE meta (
                     key   TEXT PRIMARY KEY,
                     value TEXT NOT NULL
                 ) STRICT;
                 INSERT INTO meta (key, value) VALUES ('boot_count', '7');",
            )
            .expect("写 v1 结构");
            conn.pragma_update(None, "user_version", 1).expect("标 v1");
        }

        let store = StateStore::open(&dir.path).expect("迁移 v1 数据文件");
        store
            .insert_observation(sample("s1", "迁移之后的写入"))
            .expect("迁移后应能写 Observation");

        let conn = Connection::open(&path).expect("重新直连数据文件");
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("读版本号");
        assert_eq!(version, SCHEMA_VERSION, "迁移应把版本号推到当前值");

        let archived: String = conn
            .query_row(
                "SELECT value FROM meta_v1_archive WHERE key = 'boot_count'",
                [],
                |row| row.get(0),
            )
            .expect("读归档的旧数据");
        assert_eq!(archived, "7", "迁移应保住 v1 的旧数据，不静默丢弃");
    }

    #[test]
    fn latest_observations_are_returned_in_writing_order() {
        let dir = TempDataDir::new("latest");
        let store = StateStore::open(&dir.path).expect("打开数据文件");
        for index in 0..5 {
            store
                .insert_observation(sample("s", &format!("第 {index} 条")))
                .expect("写入");
        }

        let latest_two = store.observations(None, 2).expect("读最近两条");
        assert_eq!(latest_two.len(), 2);
        assert_eq!(
            latest_two.iter().map(|item| item.content.as_str()).collect::<Vec<_>>(),
            vec!["第 3 条", "第 4 条"],
            "应返回最近的若干条，且仍按写入顺序排列",
        );
    }

    #[test]
    fn supplied_occurred_at_is_kept_instead_of_ingestion_time() {
        let dir = TempDataDir::new("occurred-at");
        let store = StateStore::open(&dir.path).expect("打开数据文件");
        let mut request = sample("s", "带事件时间");
        request.occurred_at = Some("2025-03-04T05:06:07Z".to_owned());

        let written = store.insert_observation(request).expect("写入");
        assert_eq!(written.occurred_at, "2025-03-04T05:06:07Z", "应保留事件时刻");
    }

    #[test]
    fn data_file_newer_than_core_is_refused_with_readable_message() {
        let dir = TempDataDir::new("schema-too-new");
        let path = dir.path.join(DATA_FILE_NAME);
        {
            let conn = Connection::open(&path).expect("建库");
            conn.pragma_update(None, "user_version", SCHEMA_VERSION + 1)
                .expect("写入更高 schema 版本");
        }

        let error = StateStore::open(&dir.path).expect_err("应拒绝更新的数据文件");
        let message = error.to_string();
        assert!(matches!(error, StateError::SchemaTooNew { .. }));
        assert!(message.contains(&(SCHEMA_VERSION + 1).to_string()), "报错应带上文件里的版本号：{message}");
        assert!(message.contains(&SCHEMA_VERSION.to_string()), "报错应带上 Core 支持的版本号：{message}");
    }

    #[test]
    fn corrupt_data_file_is_refused_with_readable_message() {
        let dir = TempDataDir::new("corrupt");
        let path = dir.path.join(DATA_FILE_NAME);
        std::fs::write(&path, b"this is definitely not a sqlite database").expect("写入垃圾");

        let error = StateStore::open(&dir.path).expect_err("应拒绝损坏的数据文件");
        let message = error.to_string();
        assert!(matches!(error, StateError::Corrupt { .. }), "实际错误：{error:?}");
        assert!(message.contains(&path.display().to_string()), "报错应带上路径：{message}");
    }

    #[test]
    fn data_dir_that_is_a_file_is_refused_with_readable_message() {
        let dir = TempDataDir::new("dir-is-file");
        let not_a_dir = dir.path.join("core-data");
        std::fs::write(&not_a_dir, b"x").expect("写入占位文件");

        let error = StateStore::open(&not_a_dir).expect_err("应拒绝把文件当目录");
        assert!(matches!(error, StateError::DataDirNotDirectory { .. }));
        assert!(error.to_string().contains(&not_a_dir.display().to_string()));
    }
}
