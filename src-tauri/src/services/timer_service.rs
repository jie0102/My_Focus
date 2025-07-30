use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::models::focus_session::*;

#[derive(Debug, Clone)]
pub enum TimerState {
    Stopped,
    Running,
    Paused,
}

pub struct TimerService {
    current_session: Arc<Mutex<Option<FocusSession>>>,
    timer_state: Arc<Mutex<TimerState>>,
    start_time: Arc<Mutex<Option<tokio::time::Instant>>>,
    elapsed_when_paused: Arc<Mutex<u32>>, // 暂停时的已过时间（秒）
}

impl TimerService {
    pub fn new() -> Self {
        Self {
            current_session: Arc::new(Mutex::new(None)),
            timer_state: Arc::new(Mutex::new(TimerState::Stopped)),
            start_time: Arc::new(Mutex::new(None)),
            elapsed_when_paused: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn start_session(&self, session_type: SessionType, duration_minutes: u32) -> Result<String> {
        let mut current_session = self.current_session.lock().await;
        let mut timer_state = self.timer_state.lock().await;
        let mut start_time = self.start_time.lock().await;
        let mut elapsed_when_paused = self.elapsed_when_paused.lock().await;

        let session = FocusSession {
            id: uuid::Uuid::new_v4().to_string(),
            session_type,
            status: SessionStatus::Active,
            duration_minutes,
            elapsed_seconds: 0,
            started_at: Some(chrono::Utc::now()),
            ..Default::default()
        };

        let session_id = session.id.clone();
        *current_session = Some(session);
        *timer_state = TimerState::Running;
        *start_time = Some(tokio::time::Instant::now());
        *elapsed_when_paused = 0;

        println!("开始会话: {} ({} 分钟)", session_id, duration_minutes);
        Ok(session_id)
    }

    pub async fn pause_session(&self) -> Result<()> {
        let mut timer_state = self.timer_state.lock().await;
        let mut elapsed_when_paused = self.elapsed_when_paused.lock().await;
        let start_time = self.start_time.lock().await;

        if let TimerState::Running = *timer_state {
            if let Some(start) = *start_time {
                *elapsed_when_paused += start.elapsed().as_secs() as u32;
            }
            *timer_state = TimerState::Paused;
            println!("暂停会话");
        }

        Ok(())
    }

    pub async fn resume_session(&self) -> Result<()> {
        let mut timer_state = self.timer_state.lock().await;
        let mut start_time = self.start_time.lock().await;

        if let TimerState::Paused = *timer_state {
            *timer_state = TimerState::Running;
            *start_time = Some(tokio::time::Instant::now());
            println!("恢复会话");
        }

        Ok(())
    }

    pub async fn stop_session(&self) -> Result<Option<FocusSession>> {
        let mut current_session = self.current_session.lock().await;
        let mut timer_state = self.timer_state.lock().await;
        let mut start_time = self.start_time.lock().await;
        let mut elapsed_when_paused = self.elapsed_when_paused.lock().await;

        if let Some(mut session) = current_session.take() {
            session.status = SessionStatus::Completed;
            session.completed_at = Some(chrono::Utc::now());
            
            // 计算总的已用时间
            let current_elapsed = if let Some(start) = *start_time {
                start.elapsed().as_secs() as u32
            } else {
                0
            };
            session.elapsed_seconds = *elapsed_when_paused + current_elapsed;

            *timer_state = TimerState::Stopped;
            *start_time = None;
            *elapsed_when_paused = 0;

            println!("停止会话: {}", session.id);
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub async fn get_current_session(&self) -> Option<FocusSession> {
        self.current_session.lock().await.clone()
    }

    pub async fn get_elapsed_seconds(&self) -> u32 {
        let timer_state = self.timer_state.lock().await;
        let start_time = self.start_time.lock().await;
        let elapsed_when_paused = self.elapsed_when_paused.lock().await;

        match *timer_state {
            TimerState::Running => {
                if let Some(start) = *start_time {
                    *elapsed_when_paused + start.elapsed().as_secs() as u32
                } else {
                    *elapsed_when_paused
                }
            }
            _ => *elapsed_when_paused,
        }
    }

    pub async fn get_remaining_seconds(&self) -> u32 {
        if let Some(session) = self.get_current_session().await {
            let total_seconds = session.duration_minutes * 60;
            let elapsed = self.get_elapsed_seconds().await;
            if elapsed >= total_seconds {
                0
            } else {
                total_seconds - elapsed
            }
        } else {
            0
        }
    }
} 