//! 后台对话轮次：权限确认 + 回合完成通知。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use runtime::{
    AssistantEvent, ConversationRuntime, PermissionPromptDecision, PermissionPrompter,
    PermissionRequest, RuntimeError, TurnSummary,
};

use crate::runtime_bridge::CliToolExecutor;

pub(crate) type SharedRuntime = Arc<Mutex<ConversationRuntime<llm::LlmClient, CliToolExecutor>>>;

/// 后台轮次向主线程发送的事件。
pub(crate) enum TurnEvent {
    Stream(AssistantEvent),
    ToolUse { name: String },
    Permission(PermissionRequest),
    Done(Result<TurnSummary, RuntimeError>),
}

/// 进行中的后台对话轮次。
pub(crate) struct ActiveTurn {
    pub(crate) rx: mpsc::Receiver<TurnEvent>,
    pub(crate) permission_tx: mpsc::Sender<bool>,
    cancelled: Arc<AtomicBool>,
    handle: JoinHandle<()>,
    finished: bool,
}

impl ActiveTurn {
    pub(crate) fn poll(&mut self) -> Vec<TurnEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            if matches!(event, TurnEvent::Done(_)) {
                self.finished = true;
            }
            events.push(event);
        }
        events
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.finished
    }

    #[must_use]
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// 请求中断：若正等待权限则拒绝以解锁；完成后主线程忽略摘要。
    pub(crate) fn request_cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        let _ = self.permission_tx.send(false);
    }

    pub(crate) fn join(self) {
        let _ = self.handle.join();
    }
}

struct ChannelPrompter {
    tx: mpsc::Sender<TurnEvent>,
    decision_rx: mpsc::Receiver<bool>,
}

impl PermissionPrompter for ChannelPrompter {
    fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision {
        let _ = self.tx.send(TurnEvent::Permission(request.clone()));
        match self.decision_rx.recv() {
            Ok(true) => PermissionPromptDecision::Allow,
            Ok(false) | Err(_) => PermissionPromptDecision::Deny {
                reason: format!(
                    "tool '{}' denied by user approval prompt",
                    request.tool_name
                ),
            },
        }
    }
}

pub(crate) fn spawn_turn(runtime: SharedRuntime, input: String) -> ActiveTurn {
    let (tx, rx) = mpsc::channel();
    let (permission_tx, permission_rx) = mpsc::channel();
    let cancelled = Arc::new(AtomicBool::new(false));

    let handle = thread::spawn(move || {
        let result = (|| {
            let mut runtime = runtime
                .lock()
                .map_err(|_| RuntimeError::new("runtime lock poisoned"))?;
            let mut prompter = ChannelPrompter {
                tx: tx.clone(),
                decision_rx: permission_rx,
            };
            let stream_tx = tx.clone();
            let on_stream: Box<dyn FnMut(AssistantEvent) + Send> = Box::new(move |event| {
                if matches!(
                    &event,
                    AssistantEvent::ReasoningDelta(_)
                        | AssistantEvent::TextDelta(_)
                        | AssistantEvent::ToolUse { .. }
                ) {
                    let _ = stream_tx.send(TurnEvent::Stream(event));
                }
            });
            runtime.run_turn_with_stream(input, Some(&mut prompter), Some(on_stream))
        })();
        let _ = tx.send(TurnEvent::Done(result));
    });

    ActiveTurn {
        rx,
        permission_tx,
        cancelled,
        handle,
        finished: false,
    }
}
