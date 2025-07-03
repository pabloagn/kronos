//! Application state and business logic

use chrono::{DateTime, Duration, Local};
use kronos_ipc::{Task, TimerState, TimerStatus};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct App {
    pub state: Arc<Mutex<AppState>>,
}

pub struct AppState {
    pub timer_state: TimerState,
    pub started_at: Option<DateTime<Local>>,
    pub paused_at: Option<DateTime<Local>>,
    pub accumulated_time: Duration,
    pub target_duration: Duration,
    pub tasks: Vec<Task>,
    pub next_task_id: u32,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AppState {
                timer_state: TimerState::Idle,
                started_at: None,
                paused_at: None,
                accumulated_time: Duration::zero(),
                target_duration: Duration::minutes(25), // Default pomodoro
                tasks: Vec::new(),
                next_task_id: 1,
            })),
        }
    }

    pub fn toggle_timer(&mut self) {
        let mut state = self.state.lock().unwrap();
        match state.timer_state {
            TimerState::Idle => {
                state.timer_state = TimerState::Running;
                state.started_at = Some(Local::now());
            }
            TimerState::Running => {
                state.timer_state = TimerState::Paused;
                state.paused_at = Some(Local::now());
                if let Some(started) = state.started_at {
                    let elapsed = Local::now() - started;
                    state.accumulated_time = state.accumulated_time + elapsed;
                }
            }
            TimerState::Paused => {
                state.timer_state = TimerState::Running;
                state.started_at = Some(Local::now());
                state.paused_at = None;
            }
        }
    }

    pub fn reset_timer(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.timer_state = TimerState::Idle;
        state.started_at = None;
        state.paused_at = None;
        state.accumulated_time = Duration::zero();
    }

    pub fn tick(&mut self) {
        // This will be called every frame to update any time-dependent state
        // For now, we don't need to do anything here
    }

    pub fn get_elapsed(&self) -> Duration {
        let state = self.state.lock().unwrap();
        match state.timer_state {
            TimerState::Running => {
                if let Some(started) = state.started_at {
                    state.accumulated_time + (Local::now() - started)
                } else {
                    state.accumulated_time
                }
            }
            _ => state.accumulated_time,
        }
    }

    pub fn get_status(&self) -> TimerStatus {
        let state = self.state.lock().unwrap();
        let elapsed = self.get_elapsed();
        
        TimerStatus {
            state: state.timer_state.clone(),
            elapsed: elapsed.num_seconds() as u64,
            total: state.target_duration.num_seconds() as u64,
        }
    }

    pub fn add_task(&mut self, description: String) -> u32 {
        let mut state = self.state.lock().unwrap();
        let id = state.next_task_id;
        state.tasks.push(Task {
            id,
            description,
            completed: false,
        });
        state.next_task_id += 1;
        id
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        let state = self.state.lock().unwrap();
        state.tasks.clone()
    }

    pub fn toggle_task(&mut self, id: u32) {
        let mut state = self.state.lock().unwrap();
        if let Some(task) = state.tasks.iter_mut().find(|t| t.id == id) {
            task.completed = !task.completed;
        }
    }
}
