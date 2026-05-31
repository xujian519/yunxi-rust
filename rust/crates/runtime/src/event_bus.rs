//! 内存事件总线
//!
//! 基于 tokio broadcast channel 的发布-订阅模式事件总线，
//! 为智能体间通信提供低延迟、解耦的消息分发机制。
//! 文件持久化作为可选的事件 sink 保留。

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::agent_protocol::{AgentEnvelope, MessageId, TaskStepResult, TeamId};

/// 默认事件总线容量
const DEFAULT_CAPACITY: usize = 256;

/// 事件总线事件类型
#[derive(Debug, Clone)]
pub enum Event {
    /// 智能体消息
    AgentMessage(AgentEnvelope),
    /// 任务步骤状态变更
    TaskStepChanged {
        team_id: TeamId,
        step_id: String,
        result: TaskStepResult,
    },
    /// 工作流步骤完成
    WorkflowStepCompleted {
        team_id: TeamId,
        step_index: usize,
        total_steps: usize,
    },
    /// 错误事件
    Error {
        source_agent: String,
        error: String,
        correlation_id: Option<MessageId>,
    },
}

impl Event {
    /// 获取事件的来源智能体（如果有）
    pub fn source_agent(&self) -> Option<&str> {
        match self {
            Self::AgentMessage(env) => Some(&env.from),
            Self::TaskStepChanged { .. } => None,
            Self::WorkflowStepCompleted { .. } => None,
            Self::Error { source_agent, .. } => Some(source_agent),
        }
    }

    /// 获取事件关联的团队 ID
    pub fn team_id(&self) -> Option<&str> {
        match self {
            Self::AgentMessage(env) => env.team_id.as_deref(),
            Self::TaskStepChanged { team_id, .. } => Some(team_id),
            Self::WorkflowStepCompleted { team_id, .. } => Some(team_id),
            Self::Error { .. } => None,
        }
    }
}

/// 事件总线错误
#[derive(Debug)]
pub enum EventBusError {
    /// 发送时没有活跃的接收者（非致命）
    NoReceivers,
    /// 事件总线已关闭
    Closed,
}

impl std::fmt::Display for EventBusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoReceivers => write!(f, "no active receivers"),
            Self::Closed => write!(f, "event bus closed"),
        }
    }
}

impl std::error::Error for EventBusError {}

/// 事件订阅者
pub struct EventSubscriber {
    receiver: broadcast::Receiver<Event>,
}

impl EventSubscriber {
    /// 接收下一个事件（异步）
    ///
    /// # Errors
    ///
    /// 如果发送端已关闭或发生 lag，返回错误
    pub async fn recv(&mut self) -> Result<Event, broadcast::error::RecvError> {
        self.receiver.recv().await
    }

    /// 尝试非阻塞接收
    pub fn try_recv(&mut self) -> Result<Event, broadcast::error::TryRecvError> {
        self.receiver.try_recv()
    }
}

/// 事件总线（线程安全，可克隆共享）
#[derive(Clone)]
pub struct EventBus {
    inner: Arc<broadcast::Sender<Event>>,
}

impl EventBus {
    /// 创建新的事件总线
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// 创建指定容量的事件总线
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            inner: Arc::new(sender),
        }
    }

    /// 发布事件到所有订阅者
    ///
    /// # Errors
    ///
    /// 如果没有活跃的订阅者，返回 `EventBusError::NoReceivers`（非致命）
    pub fn publish(&self, event: Event) -> Result<(), EventBusError> {
        let event_type = match &event {
            Event::AgentMessage(env) => format!("AgentMessage({}->{})", env.from, env.to),
            Event::TaskStepChanged { step_id, .. } => format!("TaskStepChanged({step_id})"),
            Event::WorkflowStepCompleted { step_index, .. } => {
                format!("WorkflowStepCompleted({step_index})")
            }
            Event::Error { source_agent, .. } => format!("Error(from:{source_agent})"),
        };
        let subscriber_count = self.inner.receiver_count();
        eprintln!(
            "[event_bus] publish: type={event_type}, subscribers={subscriber_count}"
        );
        match self.inner.send(event) {
            Ok(receiver_count) => {
                eprintln!(
                    "[event_bus] delivered to {receiver_count} subscribers"
                );
                Ok(())
            }
            Err(_) => {
                eprintln!(
                    "[event_bus] no active receivers for {event_type}"
                );
                Ok(())
            }
        }
    }

    /// 发布智能体消息事件（便捷方法）
    pub fn send_agent_message(&self, envelope: AgentEnvelope) -> Result<(), EventBusError> {
        self.publish(Event::AgentMessage(envelope))
    }

    /// 订阅事件流
    #[must_use]
    pub fn subscribe(&self) -> EventSubscriber {
        EventSubscriber {
            receiver: self.inner.subscribe(),
        }
    }

    /// 订阅并过滤特定团队的事件
    #[must_use]
    pub fn subscribe_team(&self, team_id: TeamId) -> TeamEventSubscriber {
        TeamEventSubscriber {
            receiver: self.inner.subscribe(),
            team_id,
        }
    }

    /// 获取当前活跃订阅者数量
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.inner.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// 团队事件订阅者（过滤特定团队的事件）
pub struct TeamEventSubscriber {
    receiver: broadcast::Receiver<Event>,
    team_id: TeamId,
}

impl TeamEventSubscriber {
    /// 接收下一个属于当前团队的事件
    pub async fn recv(&mut self) -> Result<Event, broadcast::error::RecvError> {
        loop {
            let event = self.receiver.recv().await?;
            if event.team_id() == Some(self.team_id.as_str()) {
                return Ok(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_protocol::{AgentEnvelope, MessageType};
    use serde_json::json;

    fn test_envelope(from: &str, to: &str, team: &str) -> AgentEnvelope {
        AgentEnvelope::new(MessageType::TaskAssignment, from, to, json!({"task": "test"}))
            .with_team(team)
    }

    #[tokio::test]
    async fn publish_and_subscribe() {
        let bus = EventBus::new();
        let mut sub = bus.subscribe();

        let envelope = test_envelope("agent-1", "agent-2", "team-a");
        bus.send_agent_message(envelope.clone()).unwrap();

        let event = sub.recv().await.expect("should receive event");
        match event {
            Event::AgentMessage(env) => {
                assert_eq!(env.from, "agent-1");
                assert_eq!(env.to, "agent-2");
            }
            _ => panic!("expected AgentMessage event"),
        }
    }

    #[tokio::test]
    async fn team_subscriber_filters_events() {
        let bus = EventBus::new();
        let mut team_sub = bus.subscribe_team("team-a".to_string());

        bus.send_agent_message(test_envelope("a1", "a2", "team-b"))
            .unwrap();
        bus.send_agent_message(test_envelope("a3", "a4", "team-a"))
            .unwrap();

        let event = team_sub.recv().await.expect("should receive team-a event");
        match event {
            Event::AgentMessage(env) => {
                assert_eq!(env.team_id.as_deref(), Some("team-a"));
            }
            _ => panic!("expected AgentMessage"),
        }
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_event() {
        let bus = EventBus::new();
        let mut sub1 = bus.subscribe();
        let mut sub2 = bus.subscribe();

        bus.send_agent_message(test_envelope("a", "b", "t"))
            .unwrap();

        let e1 = sub1.recv().await.expect("sub1 should receive");
        let e2 = sub2.recv().await.expect("sub2 should receive");

        assert!(matches!(e1, Event::AgentMessage(_)));
        assert!(matches!(e2, Event::AgentMessage(_)));
    }

    #[test]
    fn publish_without_subscribers_is_ok() {
        let bus = EventBus::new();
        let result = bus.send_agent_message(test_envelope("a", "b", "t"));
        assert!(result.is_ok());
    }

    #[test]
    fn subscriber_count_tracks_subscriptions() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);

        let sub1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        let _sub2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);

        drop(sub1);
        assert_eq!(bus.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn event_source_agent_extraction() {
        let envelope = test_envelope("retriever", "writer", "team-x");
        let event = Event::AgentMessage(envelope);
        assert_eq!(event.source_agent(), Some("retriever"));
        assert_eq!(event.team_id(), Some("team-x"));

        let error_event = Event::Error {
            source_agent: "analyzer".to_string(),
            error: "timeout".to_string(),
            correlation_id: None,
        };
        assert_eq!(error_event.source_agent(), Some("analyzer"));
        assert!(error_event.team_id().is_none());
    }
}
