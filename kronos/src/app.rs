use chrono::{DateTime, Duration, Local};
use kronos_ipc::TimerState;

#[derive(Clone)]
pub struct App {
    pub tasks: Vec<Task>,
    pub selected_task: usize,
    pub mode: AppMode,
    pub input_buffer: String,
    pub next_task_id: u32,
    pub global_timer: Timer,
}

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Normal,
    AddingTask,
    EditingTime(usize), // task index
}

#[derive(Clone)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub timer: Timer,
    pub completed: bool,
}

#[derive(Clone)]
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

impl App {
    pub fn new() -> Self {
        Self {
            tasks: vec![],
            selected_task: 0,
            mode: AppMode::Normal,
            input_buffer: String::new(),
            next_task_id: 1,
            global_timer: Timer::new(25), // Default pomodoro
        }
    }

    pub fn add_task(&mut self, description: String) {
        let task = Task {
            id: self.next_task_id,
            description,
            timer: Timer::new(25), // Default 25 minutes
            completed: false,
        };
        self.tasks.push(task);
        self.next_task_id += 1;
    }

    pub fn delete_selected_task(&mut self) {
        if !self.tasks.is_empty() {
            self.tasks.remove(self.selected_task);
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
            task.timer.reset(); // Reset when changing duration
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
            AppMode::Normal => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.mode {
            AppMode::AddingTask | AppMode::EditingTime(_) => {
                self.input_buffer.pop();
            }
            AppMode::Normal => {}
        }
    }
}
