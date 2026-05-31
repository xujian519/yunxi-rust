//! 检查点持久化。

use std::path::Path;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::flow::FlowResult;

/// 检查点记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub flow_id: String,
    pub run_id: String,
    pub step_index: usize,
    pub state: FlowResult,
    pub created_at: String,
}

/// 检查点存储
pub struct CheckpointStore {
    conn: Connection,
}

impl CheckpointStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn save(&self, checkpoint: &Checkpoint) -> Result<(), String> {
        let state_json = serde_json::to_string(&checkpoint.state).map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT INTO checkpoints (id, flow_id, run_id, step_index, state, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                step_index = excluded.step_index,
                state = excluded.state,
                created_at = excluded.created_at",
                params![
                    checkpoint.id,
                    checkpoint.flow_id,
                    checkpoint.run_id,
                    checkpoint.step_index,
                    state_json,
                    checkpoint.created_at,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_latest(&self, run_id: &str) -> Result<Option<Checkpoint>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, flow_id, run_id, step_index, state, created_at
             FROM checkpoints WHERE run_id = ?1
             ORDER BY created_at DESC LIMIT 1",
            )
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![run_id]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => {
                let state_json: String = row.get(4).map_err(|e| e.to_string())?;
                let state: FlowResult =
                    serde_json::from_str(&state_json).map_err(|e| e.to_string())?;
                Ok(Some(Checkpoint {
                    id: row.get(0).map_err(|e| e.to_string())?,
                    flow_id: row.get(1).map_err(|e| e.to_string())?,
                    run_id: row.get(2).map_err(|e| e.to_string())?,
                    step_index: row.get(3).map_err(|e| e.to_string())?,
                    state,
                    created_at: row.get(5).map_err(|e| e.to_string())?,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn list_by_flow(&self, flow_id: &str) -> Result<Vec<Checkpoint>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, flow_id, run_id, step_index, state, created_at
             FROM checkpoints WHERE flow_id = ?1
             ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![flow_id], |row| {
                let state_json: String = row.get(4)?;
                let state: FlowResult = serde_json::from_str(&state_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
                Ok(Checkpoint {
                    id: row.get(0)?,
                    flow_id: row.get(1)?,
                    run_id: row.get(2)?,
                    step_index: row.get(3)?,
                    state,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut checkpoints = Vec::new();
        for row in rows {
            checkpoints.push(row.map_err(|e| e.to_string())?);
        }
        Ok(checkpoints)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                flow_id TEXT NOT NULL,
                run_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                state TEXT NOT NULL,
                created_at TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_checkpoint_run ON checkpoints(run_id);
             CREATE INDEX IF NOT EXISTS idx_checkpoint_flow ON checkpoints(flow_id);",
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn generate_run_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::super::flow::{FlowResult, FlowStatus, StepResult};
    use super::*;
    use chrono::Utc;

    fn temp_db() -> (std::path::PathBuf, CheckpointStore) {
        let dir = std::env::temp_dir().join(format!(
            "yunxi-checkpoint-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = CheckpointStore::open(&dir.join("checkpoints.sqlite")).unwrap();
        (dir, store)
    }

    #[test]
    fn test_checkpoint_round_trip() {
        let (_dir, store) = temp_db();
        let run_id = generate_run_id();
        let checkpoint = Checkpoint {
            id: Uuid::new_v4().to_string(),
            flow_id: "flow-1".into(),
            run_id: run_id.clone(),
            step_index: 2,
            state: FlowResult {
                flow_id: "flow-1".into(),
                status: FlowStatus::Suspended,
                step_results: vec![
                    StepResult {
                        step_index: 0,
                        success: true,
                        output: None,
                        error: None,
                    },
                    StepResult {
                        step_index: 1,
                        success: true,
                        output: None,
                        error: None,
                    },
                ],
                current_step: 2,
            },
            created_at: Utc::now().to_rfc3339(),
        };

        store.save(&checkpoint).unwrap();
        let loaded = store.load_latest(&run_id).unwrap().unwrap();
        assert_eq!(loaded.step_index, 2);
        assert_eq!(loaded.state.status, FlowStatus::Suspended);
    }
}
