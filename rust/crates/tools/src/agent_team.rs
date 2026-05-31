//! 智能体团队管理（TeamCreate / TeamDelete / TeamList）

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub name: String,
    #[serde(default)]
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTeam {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub members: Vec<TeamMember>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TeamStore {
    teams: BTreeMap<String, AgentTeam>,
}

#[derive(Debug, Deserialize)]
pub struct TeamCreateInput {
    pub name: String,
    #[serde(default)]
    pub members: Vec<TeamMember>,
}

#[derive(Debug, Deserialize)]
pub struct TeamDeleteInput {
    pub team_id: String,
}

#[derive(Debug, Serialize)]
pub struct TeamCreateOutput {
    pub team: AgentTeam,
}

#[derive(Debug, Serialize)]
pub struct TeamDeleteOutput {
    pub deleted: bool,
    pub team_id: String,
}

#[derive(Debug, Serialize)]
pub struct TeamListOutput {
    pub teams: Vec<AgentTeam>,
}

fn store_path() -> PathBuf {
    if let Ok(dir) = std::env::var("YUNXI_AGENT_STORE") {
        return PathBuf::from(dir).join("teams.json");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(format!("{home}/.yunxi/agents/teams.json"))
}

fn load_store() -> Result<TeamStore, String> {
    let path = store_path();
    if !path.exists() {
        return Ok(TeamStore::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn save_store(store: &TeamStore) -> Result<(), String> {
    let path = store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

fn now_ts() -> String {
    std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

pub fn execute_team_create(input: &TeamCreateInput) -> Result<TeamCreateOutput, String> {
    let mut store = load_store()?;
    let id = format!(
        "team-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let team = AgentTeam {
        id: id.clone(),
        name: input.name.clone(),
        members: input.members.clone(),
        created_at: now_ts(),
    };
    store.teams.insert(id, team.clone());
    save_store(&store)?;
    Ok(TeamCreateOutput { team })
}

pub fn execute_team_delete(input: &TeamDeleteInput) -> Result<TeamDeleteOutput, String> {
    let mut store = load_store()?;
    let deleted = store.teams.remove(&input.team_id).is_some();
    save_store(&store)?;
    Ok(TeamDeleteOutput {
        deleted,
        team_id: input.team_id.clone(),
    })
}

pub fn execute_team_list() -> Result<TeamListOutput, String> {
    let store = load_store()?;
    Ok(TeamListOutput {
        teams: store.teams.into_values().collect(),
    })
}
