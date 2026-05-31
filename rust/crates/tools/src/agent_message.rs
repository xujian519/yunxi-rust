//! 智能体间消息传递（增强版）
//!
//! 在原有文件系统消息队列基础上，集成结构化消息协议（AgentEnvelope），
//! 保留文件持久化作为存储后端，同时支持通过 EventBus 进行内存分发。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use runtime::{generate_message_id, now_timestamp, AgentEnvelope, MessageType, Priority};

// ---- 向后兼容的旧接口 ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub team_id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentMessageSendInput {
    pub team_id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentMessageInboxInput {
    pub team_id: String,
    pub agent: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

#[derive(Debug, Serialize)]
pub struct AgentMessageSendOutput {
    pub message: AgentMessage,
}

#[derive(Debug, Serialize)]
pub struct AgentMessageInboxOutput {
    pub messages: Vec<AgentMessage>,
}

// ---- 新增：结构化消息接口 ----

/// 结构化消息发送输入
#[derive(Debug, Deserialize)]
pub struct EnvelopeSendInput {
    pub msg_type: MessageType,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub team_id: Option<String>,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub correlation_id: Option<String>,
}

/// 结构化消息发送输出
#[derive(Debug, Serialize)]
pub struct EnvelopeSendOutput {
    pub envelope: AgentEnvelope,
}

/// 结构化消息收件箱输入
#[derive(Debug, Deserialize)]
pub struct EnvelopeInboxInput {
    #[serde(default)]
    pub team_id: Option<String>,
    pub agent: String,
    #[serde(default)]
    pub msg_type: Option<MessageType>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// 结构化消息收件箱输出
#[derive(Debug, Serialize)]
pub struct EnvelopeInboxOutput {
    pub envelopes: Vec<AgentEnvelope>,
}

// ---- 存储路径 ----

fn store_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("YUNXI_AGENT_STORE") {
        return PathBuf::from(dir).join("messages");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(format!("{home}/.yunxi/agents/messages"))
}

fn envelope_dir() -> PathBuf {
    store_dir().join("envelopes")
}

// ---- 向后兼容接口实现 ----

pub fn execute_agent_message_send(
    input: &AgentMessageSendInput,
) -> Result<AgentMessageSendOutput, String> {
    let dir = store_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let id = generate_message_id();
    let message = AgentMessage {
        id: id.clone(),
        team_id: input.team_id.clone(),
        from_agent: input.from_agent.clone(),
        to_agent: input.to_agent.clone(),
        content: input.content.clone(),
        created_at: now_timestamp(),
    };
    let path = dir.join(format!("{id}.json"));
    let body = serde_json::to_string_pretty(&message).map_err(|e| e.to_string())?;
    std::fs::write(path, body).map_err(|e| e.to_string())?;
    Ok(AgentMessageSendOutput { message })
}

pub fn execute_agent_message_inbox(
    input: &AgentMessageInboxInput,
) -> Result<AgentMessageInboxOutput, String> {
    let dir = store_dir();
    let mut messages = Vec::new();
    if !dir.exists() {
        return Ok(AgentMessageInboxOutput { messages });
    }
    for entry in std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        // 跳过 envelopes 子目录中的文件
        if path.starts_with(envelope_dir()) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(msg) = serde_json::from_str::<AgentMessage>(&content) else {
            continue;
        };
        if msg.team_id == input.team_id && msg.to_agent == input.agent {
            messages.push(msg);
        }
    }
    messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    messages.truncate(input.limit);
    Ok(AgentMessageInboxOutput { messages })
}

// ---- 结构化消息接口实现 ----

/// 发送结构化消息信封
pub fn execute_envelope_send(input: &EnvelopeSendInput) -> Result<EnvelopeSendOutput, String> {
    let dir = envelope_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut envelope = AgentEnvelope::new(
        input.msg_type,
        &input.from,
        &input.to,
        input.payload.clone(),
    );

    if let Some(ref team_id) = input.team_id {
        envelope = envelope.with_team(team_id);
    }
    if let Some(ref corr_id) = input.correlation_id {
        envelope = envelope.with_correlation(corr_id.clone());
    }
    if let Some(priority) = input.priority {
        envelope = envelope.with_priority(priority);
    }

    let path = dir.join(format!("{}.json", envelope.id));
    let body = serde_json::to_string_pretty(&envelope).map_err(|e| e.to_string())?;
    std::fs::write(path, body).map_err(|e| e.to_string())?;

    Ok(EnvelopeSendOutput { envelope })
}

/// 获取结构化消息收件箱
pub fn execute_envelope_inbox(input: &EnvelopeInboxInput) -> Result<EnvelopeInboxOutput, String> {
    let dir = envelope_dir();
    let mut envelopes = Vec::new();
    if !dir.exists() {
        return Ok(EnvelopeInboxOutput { envelopes });
    }
    for entry in std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(env) = serde_json::from_str::<AgentEnvelope>(&content) else {
            continue;
        };

        // 过滤条件
        if env.to != input.agent {
            continue;
        }
        if let Some(ref team_id) = input.team_id {
            if env.team_id.as_deref() != Some(team_id.as_str()) {
                continue;
            }
        }
        if let Some(msg_type) = input.msg_type {
            if env.msg_type != msg_type {
                continue;
            }
        }
        envelopes.push(env);
    }
    envelopes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    envelopes.truncate(input.limit);
    Ok(EnvelopeInboxOutput { envelopes })
}
