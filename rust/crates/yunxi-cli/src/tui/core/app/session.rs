//! Session-related action handling (extracted from dispatch_action).

use crate::session_mgr::{create_managed_session_handle, list_managed_sessions};
use crate::tui::components::session_picker::SessionPicker;
use crate::tui::core::action::Action;

use super::App;

impl App {
    /// 处理会话管理相关 Action，返回 true 表示已处理。
    pub(crate) fn dispatch_session_action(
        &mut self,
        action: &Action,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match action {
            Action::NewSession => {
                match create_managed_session_handle() {
                    Ok(handle) => {
                        let session = runtime::Session::new();
                        match crate::runtime_bridge::build_runtime(
                            session,
                            self.model.clone(),
                            self.system_prompt.clone(),
                            true,
                            false,
                            self.allowed_tools.clone(),
                            self.permission_mode,
                        ) {
                            Ok(new_runtime) => {
                                *self.runtime.lock().map_err(|_| "lock")? = new_runtime;
                                self.chat.clear();
                                self.tools.clear();
                                self.session_handle = handle;
                                self.session_picker = None;
                                self.push_system_message("已创建新会话");
                            }
                            Err(e) => self.push_system_message(&format!("创建会话失败: {e}")),
                        }
                    }
                    Err(e) => self.push_system_message(&format!("创建会话句柄失败: {e}")),
                }
                Ok(true)
            }
            Action::SwitchSession(id) => {
                if let Err(e) = self.switch_session(id) {
                    self.push_system_message(&format!("切换会话失败: {e}"));
                }
                Ok(true)
            }
            Action::DeleteSession(id) => {
                match crate::session_mgr::sessions_dir() {
                    Ok(dir) => {
                        let path = dir.join(format!("{id}.json"));
                        if path.exists() {
                            let _ = std::fs::remove_file(&path);
                            self.push_system_message(&format!("已删除会话 {id}"));
                            if self.session_picker.is_some() {
                                if let Ok(sessions) = list_managed_sessions() {
                                    self.session_picker = Some(SessionPicker::new(
                                        sessions,
                                        self.session_handle.id.clone(),
                                    ));
                                }
                            }
                        } else {
                            self.push_system_message(&format!("会话文件不存在: {id}"));
                        }
                    }
                    Err(e) => self.push_system_message(&format!("获取会话目录失败: {e}")),
                }
                Ok(true)
            }
            Action::RenameSession(old_id, new_name) => {
                match crate::session_mgr::rename_session(old_id, new_name) {
                    Ok(handle) => {
                        if handle.id == self.session_handle.id {
                            self.session_handle = handle;
                        }
                        self.push_system_message(&format!("已重命名会话为 {new_name}"));
                        if self.session_picker.is_some() {
                            if let Ok(sessions) = list_managed_sessions() {
                                self.session_picker = Some(SessionPicker::new(
                                    sessions,
                                    self.session_handle.id.clone(),
                                ));
                            }
                        }
                    }
                    Err(e) => self.push_system_message(&format!("重命名失败: {e}")),
                }
                Ok(true)
            }
            Action::SaveSession => {
                self.persist_session();
                Ok(true)
            }
            Action::ShowSessionPicker | Action::OpenSessionPicker => {
                self.open_session_picker();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(super) fn open_session_picker(&mut self) {
        if let Ok(sessions) = list_managed_sessions() {
            self.session_picker =
                Some(SessionPicker::new(sessions, self.session_handle.id.clone()));
        }
    }

    pub(super) fn switch_session(
        &mut self,
        target: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let handle = crate::session_mgr::resolve_session_reference(target)?;
        let session = runtime::Session::load_from_path(&handle.path)?;
        let new_runtime = crate::runtime_bridge::build_runtime(
            session,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            false,
            self.allowed_tools.clone(),
            self.permission_mode,
        )?;
        *self.runtime.lock().map_err(|_| "lock")? = new_runtime;
        self.session_handle = handle;
        self.push_system_message(&format!("已切换至会话 {}", self.session_handle.id));
        Ok(())
    }
}
