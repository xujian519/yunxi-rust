//! 会话 JSON 扩展字段：`athena`（路由快照、挂起工作流）

use std::fs;
use std::path::Path;

use runtime::Session;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tools::FlowHitlDisplayInfo;

use crate::routing::RoutingSnapshot;

/// 挂起中的工作流（等待 HITL 恢复）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspendedFlowRecord {
    pub flow_id: String,
    pub run_id: String,
    #[serde(default)]
    pub noted_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_step: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_description: Option<String>,
}

impl SuspendedFlowRecord {
    pub(crate) fn merge_hitl_display(&mut self, info: &FlowHitlDisplayInfo) {
        if self.flow_name.is_none() {
            self.flow_name = info.flow_name.clone();
        }
        if self.current_step.is_none() {
            self.current_step = info.current_step;
        }
        if self.step_title.is_none() {
            self.step_title = info.step_title.clone();
        }
        if self.step_description.is_none() {
            self.step_description = info.step_description.clone();
        }
    }

    pub(crate) fn enrich_from_checkpoint(&mut self) {
        if self.step_title.is_some() && self.flow_name.is_some() {
            return;
        }
        let info = tools::lookup_flow_hitl_display(&self.flow_id, &self.run_id);
        self.merge_hitl_display(&info);
    }
}

/// 写入会话文件的 Athena 元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AthenaSessionMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_routing: Option<RoutingSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suspended_flows: Vec<SuspendedFlowRecord>,
}

impl AthenaSessionMeta {
    #[must_use]
    pub fn pending_flow_label(&self) -> Option<String> {
        self.suspended_flows.last().map(|f| {
            let label = f
                .step_title
                .as_deref()
                .or(f.flow_name.as_deref())
                .unwrap_or(&f.flow_id);
            let fid = truncate_id(label, 8);
            format!("⏸{fid}")
        })
    }
}

/// 从 `FlowTool` JSON 输出解析挂起记录
#[must_use]
pub fn flow_suspend_from_tool_output(output: &str) -> Option<SuspendedFlowRecord> {
    let value: Value = serde_json::from_str(output).ok()?;
    if !value
        .get("suspended")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return None;
    }
    let flow_id = value
        .get("flow_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())?
        .to_string();
    let run_id = value
        .get("run_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())?
        .to_string();
    let mut record = SuspendedFlowRecord {
        flow_id,
        run_id,
        noted_at: chrono_lite_timestamp(),
        flow_name: value
            .get("flow_name")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        current_step: value
            .get("current_step")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize),
        step_title: value
            .get("hitl_title")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        step_description: value
            .get("hitl_description")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    };
    record.enrich_from_checkpoint();
    Some(record)
}

/// 读取会话文件中的 `athena` 块
pub fn load_athena_meta(path: &Path) -> AthenaSessionMeta {
    read_session_root(path)
        .ok()
        .and_then(|root| root.get("athena").cloned())
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// 合并保存会话（保留 `patentCase` 等已有顶层字段）
pub fn merge_save_session(
    path: &Path,
    session: &Session,
    athena: &AthenaSessionMeta,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root =
        read_session_root(path).unwrap_or_else(|_| json!({ "version": 1, "messages": [] }));

    let session_text = session.to_json().render();
    let session_value: Value = serde_json::from_str(&session_text)?;
    if let Some(obj) = root.as_object_mut() {
        if let Some(version) = session_value.get("version") {
            obj.insert("version".into(), version.clone());
        }
        if let Some(messages) = session_value.get("messages") {
            obj.insert("messages".into(), messages.clone());
        }
        if athena.last_routing.is_some() || !athena.suspended_flows.is_empty() {
            obj.insert(
                "athena".into(),
                serde_json::to_value(athena).map_err(|e| e.to_string())?,
            );
        } else {
            obj.remove("athena");
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

#[allow(dead_code)]
// 保留原因: 预留给用户从 UI 主动取消挂起的工作流
pub fn remove_suspended_flow(meta: &mut AthenaSessionMeta, flow_id: &str, run_id: &str) {
    meta.suspended_flows
        .retain(|f| !(f.flow_id == flow_id && f.run_id == run_id));
}

/// 调用 `FlowTool` 恢复挂起的工作流
pub fn execute_flow_resume(flow_id: &str, run_id: &str, approved: bool) -> Result<String, String> {
    tools::execute_tool(
        "FlowTool",
        &json!({
            "operation": "resume_flow",
            "flowId": flow_id,
            "runId": run_id,
            "approved": approved
        }),
    )
}

#[allow(dead_code)]
// 保留原因: 预留给工作流引擎主动挂起时记录状态
pub fn push_suspended_flow(meta: &mut AthenaSessionMeta, record: SuspendedFlowRecord) {
    if meta
        .suspended_flows
        .iter()
        .any(|f| f.flow_id == record.flow_id && f.run_id == record.run_id)
    {
        return;
    }
    meta.suspended_flows.push(record);
}

fn read_session_root(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn truncate_id(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flow_suspend_json_with_hitl_fields() {
        let out = r#"{
            "flow_id":"f1",
            "run_id":"r1",
            "suspended":true,
            "flow_name":"专利答复",
            "current_step":2,
            "hitl_title":"确认检索",
            "hitl_description":"step-2"
        }"#;
        let rec = flow_suspend_from_tool_output(out).expect("suspend");
        assert_eq!(rec.flow_id, "f1");
        assert_eq!(rec.step_title.as_deref(), Some("确认检索"));
        assert_eq!(rec.flow_name.as_deref(), Some("专利答复"));
    }
}
