use super::{manager::SessionManager, Session};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

pub struct SessionReplay {
    manager: SessionManager,
    current_session: Option<Session>,
    playback_queue: VecDeque<ReplayCommand>,
    is_paused: bool,
    speed: PlaybackSpeed,
    position: usize,
}

#[derive(Debug, Clone)]
pub enum ReplayCommand {
    Command {
        input: String,
        output: String,
        timestamp: u64,
    },
    FileOpen {
        path: String,
        timestamp: u64,
    },
    FileEdit {
        path: String,
        changes: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackSpeed {
    X1,
    X2,
    X4,
}

impl SessionReplay {
    pub fn new(manager: SessionManager) -> Self {
        Self {
            manager,
            current_session: None,
            playback_queue: VecDeque::new(),
            is_paused: false,
            speed: PlaybackSpeed::X1,
            position: 0,
        }
    }

    pub fn load_session(&mut self, session_id: &str) -> Option<()> {
        let session = self.manager.get_session(session_id)?.clone();
        self.current_session = Some(session.clone());
        self.load_replay_queue(&session);
        self.position = 0;
        Some(())
    }

    fn load_replay_queue(&mut self, session: &Session) {
        self.playback_queue.clear();

        for record in &session.commands {
            self.playback_queue.push_back(ReplayCommand::Command {
                input: record.input.clone(),
                output: record.output.clone(),
                timestamp: record.timestamp,
            });
        }

        for file in &session.files {
            self.playback_queue.push_back(ReplayCommand::FileOpen {
                path: file.path.clone(),
                timestamp: file.timestamp,
            });
        }
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn stop(&mut self) {
        self.is_paused = true;
        self.playback_queue.clear();
        self.position = 0;
    }

    pub fn set_speed(&mut self, speed: PlaybackSpeed) {
        self.speed = speed;
    }

    pub fn get_speed(&self) -> PlaybackSpeed {
        self.speed
    }

    pub fn next(&mut self) -> Option<ReplayCommand> {
        if self.is_paused || self.playback_queue.is_empty() {
            return None;
        }

        let cmd = self.playback_queue.pop_front()?;
        self.position += 1;

        let delay = self.calculate_delay();
        if delay > 0 {
            thread::sleep(Duration::from_millis(delay));
        }

        Some(cmd)
    }

    pub fn previous(&mut self) -> Option<ReplayCommand> {
        None
    }

    pub fn seek(&mut self, index: usize) -> Option<()> {
        let total = self
            .current_session
            .as_ref()
            .map(|s| s.commands.len() + s.files.len())
            .unwrap_or(0);

        if index > total {
            return None;
        }
        self.position = index;
        self.playback_queue.drain(..index);
        Some(())
    }

    pub fn progress(&self) -> (usize, usize) {
        let total = self
            .current_session
            .as_ref()
            .map(|s| s.commands.len() + s.files.len())
            .unwrap_or(0);
        (self.position, total)
    }

    fn calculate_delay(&self) -> u64 {
        let base_delay = 500;
        match self.speed {
            PlaybackSpeed::X1 => base_delay,
            PlaybackSpeed::X2 => base_delay / 2,
            PlaybackSpeed::X4 => base_delay / 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_speed() {
        let manager = SessionManager::new();
        let mut replay = SessionReplay::new(manager);

        replay.set_speed(PlaybackSpeed::X2);
        assert_eq!(replay.get_speed(), PlaybackSpeed::X2);

        replay.set_speed(PlaybackSpeed::X4);
        assert_eq!(replay.get_speed(), PlaybackSpeed::X4);
    }

    #[test]
    fn test_pause_resume() {
        let manager = SessionManager::new();
        let mut replay = SessionReplay::new(manager);

        assert!(!replay.is_paused());
        replay.pause();
        assert!(replay.is_paused());
        replay.resume();
        assert!(!replay.is_paused());
    }
}
