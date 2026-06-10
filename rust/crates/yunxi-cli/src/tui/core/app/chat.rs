//! Chat/turn-related action handling (extracted from dispatch_action).

use std::sync::Arc;
use std::time::Instant;

use crate::model_routing::{load_router_config, select_model_for_request};
use crate::runtime_bridge::build_runtime;
use crate::tui::core::action::Action;
use crate::tui::turn::spawn_turn;

use super::App;

impl App {
    /// 处理聊天/轮次相关 Action，返回 true 表示已处理。
    pub(crate) fn dispatch_chat_action(
        &mut self,
        action: &Action,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match action {
            Action::Submit(text, as_intervention) => {
                self.handle_submit(text.clone(), *as_intervention)?;
                Ok(true)
            }
            Action::InterruptTurn => {
                self.interrupt_turn();
                Ok(true)
            }
            Action::PermissionDecision(allow) => {
                if let Some(turn) = self.active_turn.as_mut() {
                    let _ = turn.permission_tx.send(*allow);
                }
                self.set_pending_permission(None);
                Ok(true)
            }
            Action::FlowResume(flow_id, run_id) => {
                self.handle_flow_resume(flow_id, run_id)?;
                self.set_pending_flow_hitl(None);
                Ok(true)
            }
            Action::ShowCommandPalette => {
                self.command_palette.show();
                Ok(true)
            }
            Action::HideCommandPalette => {
                self.command_palette.hide();
                Ok(true)
            }
            Action::ExecuteCommand(cmd) => {
                match cmd.as_str() {
                    "sessions" => {
                        self.open_session_picker();
                    }
                    "interrupt" => self.interrupt_turn(),
                    "edit_keymap" => {
                        let bindings: Vec<String> = self
                            .keymap
                            .list_all_bindings()
                            .iter()
                            .map(|b| format!("{}: {}", b.sequence, b.command))
                            .collect();
                        let msg = if bindings.is_empty() {
                            "暂无快捷键绑定".to_string()
                        } else {
                            format!(
                                "当前快捷键绑定 (共 {} 条):\n{}",
                                bindings.len(),
                                bindings.join("\n")
                            )
                        };
                        self.push_system_message(&msg);
                    }
                    _ => {
                        self.push_system_message(&format!("未接入的命令 '{}'", cmd));
                    }
                }
                Ok(true)
            }
            Action::SwitchModel => {
                self.push_system_message("切换模型（开发中）");
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    // ── Turn management ──

    fn handle_submit(
        &mut self,
        text: String,
        as_intervention: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut interrupted = false;
        if self.active_turn.is_some() {
            if let Some(turn) = self.active_turn.take() {
                turn.request_cancel();
                turn.join();
            }
            interrupted = true;
            self.end_turn_stream();
            self.set_thinking(false);
            self.active_tool = None;
            self.set_pending_permission(None);
            self.turn_started_at = None;
        }

        // Handle slash commands
        if text.trim().starts_with('/') {
            let result = self.handle_slash_command(&text)?;
            if !result {
                self.push_system_message("未识别的斜杠命令。输入 /help 查看可用命令。");
            }
            return Ok(());
        }

        let prompt = if as_intervention || interrupted {
            self.wrap_human_intervention(&text, interrupted)
        } else {
            text.clone()
        };

        self.start_llm_turn(&prompt);
        Ok(())
    }

    pub(super) fn start_llm_turn(&mut self, text: &str) {
        let effective_model = self.resolve_model(text);

        if let Some(msg) = crate::llm_auth::missing_api_key_message(&effective_model) {
            self.push_assistant_message(&msg);
            return;
        }

        self.set_thinking(true);
        self.begin_turn_stream();
        self.turn_started_at = Some(Instant::now());
        self.active_tool = None;
        self.close_human_guide();

        // Routing
        let decision = crate::routing::route(text);
        let prefix = crate::routing::routing_user_prefix(&decision);
        let _snapshot = crate::routing::RoutingSnapshot::from_decision(&decision);
        crate::routing::merge_suggested_tools(&mut self.allowed_tools, &decision);

        let routed_input = format!("{prefix}{text}");
        self.active_turn = Some(spawn_turn(Arc::clone(&self.runtime), routed_input));
    }

    fn resolve_model(&mut self, text: &str) -> String {
        if self.model != "auto" {
            return self.model.clone();
        }

        let history_rounds = self
            .runtime
            .lock()
            .map_or(0, |r| r.session().messages.len());
        let router_config = load_router_config();
        let resolved = select_model_for_request("auto", text, history_rounds, 0, &router_config);

        let need_rebuild = match &self.active_model {
            None => true,
            Some(current) => current != &resolved,
        };

        if need_rebuild {
            let session = self.runtime.lock().ok().map(|r| r.session().clone());
            if let Some(session) = session {
                match build_runtime(
                    session,
                    resolved.clone(),
                    self.system_prompt.clone(),
                    true,
                    false,
                    self.allowed_tools.clone(),
                    self.permission_mode,
                ) {
                    Ok(new_runtime) => {
                        if let Ok(mut rt) = self.runtime.lock() {
                            *rt = new_runtime;
                        }
                    }
                    Err(e) => {
                        self.push_system_message(&format!("auto 路由切换失败: {e}"));
                    }
                }
            } else {
                self.push_system_message("auto 路由读取会话失败: runtime lock poisoned");
            }
            self.last_auto_decision = Some(format!("auto→{}", resolved));
        }
        self.active_model = Some(resolved.clone());
        resolved
    }

    fn interrupt_turn(&mut self) {
        if let Some(turn) = self.active_turn.take() {
            turn.request_cancel();
            turn.join();
        }
        self.end_turn_stream();
        self.set_thinking(false);
        self.active_tool = None;
        self.set_pending_permission(None);
        self.set_pending_flow_hitl(None);
        self.turn_started_at = None;
        self.open_human_guide();
        self.push_system_message("已请求中断当前轮次。请在底栏编辑引导内容后 Enter 发送。");
    }
}
