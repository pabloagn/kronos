use chrono::{DateTime, Duration, Local};
use kronos_ipc::TimerState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct App {
    pub tasks: Vec<Task>,
    pub selected_task: usize,
    #[serde(skip)]
    pub mode: AppMode,
    #[serde(skip)]
    pub input_buffer: String,
    pub next_task_id: u32,
    pub global_timer: Timer,
    pub presets: HashMap<String, i64>, // name -> minutes
    #[serde(skip)]
    pub notifications_sent: Vec<u32>, // task ids that already sent notifications
}

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Normal,
    AddingTask,
    EditingTime(usize),
    SelectingPreset(usize),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub timer: Timer,
    pub completed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Timer {
    pub state: TimerState,
    pub started_at: Option<DateTime<Local>>,
    pub accumulated_time: Duration,
    pub target_duration: Duration,
}

impl Timer {
    pub fn new(minutes: i64) -> Self {
        Self {
            state: TimerState::Idle,
            started_at: None,
            accumulated_time: Duration::zero(),
            target_duration: Duration::minutes(minutes),
        }
    }

    pub fn toggle(&mut self) {
        match self.state {
            TimerState::Idle => {
                self.state = TimerState::Running;
                self.started_at = Some(Local::now());
            }
            TimerState::Running => {
                self.state = TimerState::Paused;
                if let Some(started) = self.started_at {
                    let elapsed = Local::now() - started;
                    self.accumulated_time = self.accumulated_time + elapsed;
                }
                self.started_at = None;
            }
            TimerState::Paused => {
                self.state = TimerState::Running;
                self.started_at = Some(Local::now());
            }
        }
    }

    pub fn reset(&mut self) {
        self.state = TimerState::Idle;
        self.started_at = None;
        self.accumulated_time = Duration::zero();
    }

    pub fn get_elapsed(&self) -> Duration {
        match self.state {
            TimerState::Running => {
                if let Some(started) = self.started_at {
                    self.accumulated_time + (Local::now() - started)
                } else {
                    self.accumulated_time
                }
            }
            _ => self.accumulated_time,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.get_elapsed() >= self.target_duration
    }

    pub fn get_remaining(&self) -> Duration {
        let elapsed = self.get_elapsed();
        if elapsed >= self.target_duration {
            Duration::zero()
        } else {
            self.target_duration - elapsed
        }
    }

    pub fn get_progress(&self) -> f64 {
        let elapsed = self.get_elapsed().num_seconds() as f64;
        let total = self.target_duration.num_seconds() as f64;
        if total > 0.0 {
            (elapsed / total).min(1.0)
        } else {
            0.0
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let mut presets = HashMap::new();
        presets.insert("Pomodoro".to_string(), 25);
        presets.insert("Short Break".to_string(), 5);
        presets.insert("Long Break".to_string(), 15);
        presets.insert("Focus".to_string(), 45);
        presets.insert("Deep Work".to_string(), 90);

        Self {
            tasks: vec![],
            selected_task: 0,
            mode: AppMode::Normal,
            input_buffer: String::new(),
            next_task_id: 1,
            global_timer: Timer::new(25),
            presets,
            notifications_sent: vec![],
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_task(&mut self, description: String) {
        let task = Task {
            id: self.next_task_id,
            description,
            timer: Timer::new(25), // Default pomodoro
            completed: false,
        };
        self.tasks.push(task);
        self.next_task_id += 1;
    }

    pub fn delete_selected_task(&mut self) {
        if !self.tasks.is_empty() {
            let task_id = self.tasks[self.selected_task].id;
            self.tasks.remove(self.selected_task);
            self.notifications_sent.retain(|&id| id != task_id);
            if self.selected_task >= self.tasks.len() && self.selected_task > 0 {
                self.selected_task -= 1;
            }
        }
    }

    pub fn toggle_selected_task_completion(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_task) {
            task.completed = !task.completed;
        }
    }

    pub fn toggle_selected_timer(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_task) {
            task.timer.toggle();
        }
    }

    pub fn reset_selected_timer(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_task) {
            task.timer.reset();
            // Remove from notifications sent
            self.notifications_sent.retain(|&id| id != task.id);
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_task > 0 {
            self.selected_task -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_task < self.tasks.len().saturating_sub(1) {
            self.selected_task += 1;
        }
    }

    pub fn set_task_duration(&mut self, task_idx: usize, minutes: i64) {
        if let Some(task) = self.tasks.get_mut(task_idx) {
            task.timer.target_duration = Duration::minutes(minutes);
            task.timer.reset();
            self.notifications_sent.retain(|&id| id != task.id);
        }
    }

    pub fn set_task_duration_from_preset(&mut self, task_idx: usize, preset_name: &str) {
        if let Some(&minutes) = self.presets.get(preset_name) {
            self.set_task_duration(task_idx, minutes);
        }
    }

    pub fn handle_char(&mut self, c: char) {
        match self.mode {
            AppMode::AddingTask => {
                if c == '\n' {
                    if !self.input_buffer.is_empty() {
                        self.add_task(self.input_buffer.clone());
                        self.input_buffer.clear();
                        self.mode = AppMode::Normal;
                    }
                } else {
                    self.input_buffer.push(c);
                }
            }
            AppMode::EditingTime(task_idx) => {
                if c == '\n' {
                    if let Ok(minutes) = self.input_buffer.parse::<i64>() {
                        self.set_task_duration(task_idx, minutes);
                    }
                    self.input_buffer.clear();
                    self.mode = AppMode::Normal;
                } else if c.is_numeric() {
                    self.input_buffer.push(c);
                }
            }
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.mode {
            AppMode::AddingTask | AppMode::EditingTime(_) => {
                self.input_buffer.pop();
            }
            _ => {}
        }
    }

    pub fn check_and_notify_completions(&mut self) {
        // Check global timer
        if self.global_timer.is_complete()
            && self.global_timer.state == TimerState::Running
            && !self.notifications_sent.contains(&0)
        {
            self.send_notification("Global Timer", "Timer completed!");
            self.notifications_sent.push(0);
        }

        // Check task timers
        for task in &self.tasks {
            if task.timer.is_complete()
                && task.timer.state == TimerState::Running
                && !self.notifications_sent.contains(&task.id)
            {
                self.send_notification(&task.description, "Task timer completed!");
                self.notifications_sent.push(task.id);
            }
        }
    }

    fn send_notification(&self, title: &str, body: &str) {
        if let Err(e) = std::process::Command::new("notify-send")
            .arg("--app-name=kronos")
            .arg(title)
            .arg(body)
            .spawn()
        {
            eprintln!("Failed to send notification: {}", e);
        }
    }

    pub fn get_preset_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.presets.keys().cloned().collect();
        names.sort();
        names
    }
}
