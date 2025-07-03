use crate::config::Config;
use chrono::{DateTime, Duration, Local};
use kronos_ipc::TimerState;
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::time::Duration as StdDuration;
use tachyonfx::{fx, EffectManager, Motion};

#[derive(Serialize, Deserialize)]
pub struct App {
    pub tasks: Vec<Task>,
    pub selected_task: usize,
    pub next_task_id: u32,
    pub global_timer: Timer,
    pub presets: HashMap<String, i64>,
    #[serde(skip)]
    pub mode: AppMode,
    #[serde(skip)]
    pub input_buffer: String,
    #[serde(skip)]
    pub notifications_sent: Vec<u32>,
    #[serde(skip)]
    pub config: Config,
    #[serde(skip, default = "default_effect_manager")]
    pub effect_manager: EffectManager<u32>,
    #[serde(skip)]
    pub should_quit: bool,
    pub stats: Stats,
}

pub fn default_effect_manager() -> EffectManager<u32> {
    EffectManager::default()
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            tasks: self.tasks.clone(),
            selected_task: self.selected_task,
            next_task_id: self.next_task_id,
            global_timer: self.global_timer.clone(),
            presets: self.presets.clone(),
            mode: self.mode.clone(),
            input_buffer: self.input_buffer.clone(),
            notifications_sent: self.notifications_sent.clone(),
            config: self.config.clone(),
            effect_manager: EffectManager::default(),
            should_quit: self.should_quit,
            stats: self.stats.clone(),
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum AppMode {
    #[default]
    Normal,
    AddingTask,
    EditingTime(usize),
    SelectingPreset(usize),
    StartupAnimation,
    ShowStats,
    ShowHelp,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub timer: Timer,
    pub completed: bool,
    pub category: TaskCategory,
    pub priority: Priority,
    pub created_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TaskCategory {
    Work,
    Personal,
    Study,
    Exercise,
    Other(String),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_completed: u32,
    pub total_time_worked: Duration,
    pub daily_streak: u32,
    pub last_active_date: DateTime<Local>,
    pub tasks_by_category: HashMap<String, u32>,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            total_completed: 0,
            total_time_worked: Duration::zero(),
            daily_streak: 0,
            last_active_date: Local::now(),
            tasks_by_category: HashMap::new(),
        }
    }
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
                    self.accumulated_time = self.accumulated_time + (Local::now() - started);
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
        if let (TimerState::Running, Some(started)) = (self.state.clone(), self.started_at) {
            self.accumulated_time + (Local::now() - started)
        } else {
            self.accumulated_time
        }
    }
    pub fn is_complete(&self) -> bool {
        self.get_elapsed() >= self.target_duration
    }
    pub fn get_remaining(&self) -> Duration {
        self.target_duration
            .checked_sub(&self.get_elapsed())
            .unwrap_or_else(Duration::zero)
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
    pub fn new(config: Config) -> Self {
        let mut presets = HashMap::new();
        presets.insert("Pomodoro".to_string(), 25);
        presets.insert("Short Break".to_string(), 5);
        presets.insert("Long Break".to_string(), 15);
        Self {
            tasks: vec![],
            selected_task: 0,
            mode: AppMode::StartupAnimation,
            input_buffer: String::new(),
            next_task_id: 1,
            global_timer: Timer::new(25),
            presets,
            notifications_sent: vec![],
            config,
            effect_manager: EffectManager::default(),
            should_quit: false,
            stats: Stats::default(),
        }
    }

    pub fn add_task(&mut self, description: String) {
        self.tasks.push(Task {
            id: self.next_task_id,
            description,
            timer: Timer::new(25),
            completed: false,
            category: TaskCategory::Other("General".to_string()),
            priority: Priority::Medium,
            created_at: Local::now(),
            completed_at: None,
        });
        self.next_task_id += 1;
    }

    pub fn delete_selected_task(&mut self) {
        if self.tasks.get(self.selected_task).is_some() {
            let task = self.tasks.remove(self.selected_task);
            self.notifications_sent.retain(|&id| id != task.id);
            if !self.tasks.is_empty() && self.selected_task >= self.tasks.len() {
                self.selected_task = self.tasks.len() - 1;
            }
        }
    }

    pub fn toggle_selected_task_completion(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_task) {
            task.completed = !task.completed;
            if task.completed {
                task.completed_at = Some(Local::now());
                // Store the data we need before dropping the mutable borrow
                let elapsed = task.timer.get_elapsed();
                let category = task.category.clone();
            } else {
                task.completed_at = None;
                return; // Early return if uncompleting
            }
        }

        // Now we can safely call update_stats without borrow issues
        if self.tasks[self.selected_task].completed {
            self.stats.total_completed += 1;
            self.stats.total_time_worked =
                self.stats.total_time_worked + self.tasks[self.selected_task].timer.get_elapsed();

            // Update category stats
            let category_name = match &self.tasks[self.selected_task].category {
                TaskCategory::Work => "Work",
                TaskCategory::Personal => "Personal",
                TaskCategory::Study => "Study",
                TaskCategory::Exercise => "Exercise",
                TaskCategory::Other(name) => name,
            };
            *self
                .stats
                .tasks_by_category
                .entry(category_name.to_string())
                .or_insert(0) += 1;

            // Update streak
            let today = Local::now().date_naive();
            let last_active = self.stats.last_active_date.date_naive();
            if today == last_active {
                // Same day, no change
            } else if today.signed_duration_since(last_active).num_days() == 1 {
                self.stats.daily_streak += 1;
            } else {
                self.stats.daily_streak = 1;
            }
            self.stats.last_active_date = Local::now();
        }
    }

    pub fn update_stats(&mut self, task: &Task) {
        self.stats.total_completed += 1;
        self.stats.total_time_worked = self.stats.total_time_worked + task.timer.get_elapsed();

        // Update category stats
        let category_name = match &task.category {
            TaskCategory::Work => "Work",
            TaskCategory::Personal => "Personal",
            TaskCategory::Study => "Study",
            TaskCategory::Exercise => "Exercise",
            TaskCategory::Other(name) => name,
        };
        *self
            .stats
            .tasks_by_category
            .entry(category_name.to_string())
            .or_insert(0) += 1;

        // Update streak
        let today = Local::now().date_naive();
        let last_active = self.stats.last_active_date.date_naive();
        if today == last_active {
            // Same day, no change
        } else if today.signed_duration_since(last_active).num_days() == 1 {
            self.stats.daily_streak += 1;
        } else {
            self.stats.daily_streak = 1;
        }
        self.stats.last_active_date = Local::now();
    }

    pub fn toggle_selected_timer(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_task) {
            task.timer.toggle();
        }
    }

    pub fn reset_selected_timer(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_task) {
            task.timer.reset();
            self.notifications_sent.retain(|&id| id != task.id);
        }
    }

    pub fn move_selection_up(&mut self) {
        self.selected_task = self.selected_task.saturating_sub(1);
    }

    pub fn move_selection_down(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task = (self.selected_task + 1).min(self.tasks.len() - 1);
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
                    }
                    self.input_buffer.clear();
                    self.mode = AppMode::Normal;
                } else {
                    self.input_buffer.push(c);
                }
            }
            AppMode::EditingTime(task_idx) => {
                if c == '\n' {
                    if let Ok(minutes) = self.input_buffer.parse() {
                        self.set_task_duration(task_idx, minutes);
                    }
                    self.input_buffer.clear();
                    self.mode = AppMode::Normal;
                } else if c.is_numeric() {
                    self.input_buffer.push(c);
                }
            }
            AppMode::SelectingPreset(task_idx) => {
                if c.is_numeric() {
                    let index = c.to_digit(10).unwrap_or(0) as usize;
                    if index > 0 && index <= self.presets.len() {
                        let preset_names = self.get_preset_names();
                        if let Some(preset_name) = preset_names.get(index - 1) {
                            self.set_task_duration_from_preset(task_idx, preset_name);
                            self.mode = AppMode::Normal;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        if matches!(self.mode, AppMode::AddingTask | AppMode::EditingTime(_)) {
            self.input_buffer.pop();
        }
    }

    pub fn check_and_notify_completions(&mut self) {
        if self.global_timer.is_complete()
            && self.global_timer.state == TimerState::Running
            && !self.notifications_sent.contains(&0)
        {
            self.send_notification("Global Timer", "Timer completed!");
            self.notifications_sent.push(0);
        }
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
        if let Err(e) = notify_rust::Notification::new()
            .summary(title)
            .body(body)
            .appname("kronos")
            .show()
        {
            eprintln!("Failed to send notification: {}", e);
        }
    }

    pub fn get_preset_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.presets.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn trigger_mode_change_effect(&mut self, area: Rect) {
        let effect = fx::slide_in(Motion::LeftToRight, 8, 4, self.config.theme.selection, 300)
            .with_area(area);
        self.effect_manager.add_effect(effect);
    }

    pub fn trigger_delete_effect(&mut self, area: Rect) {
        let effect = fx::dissolve(500).with_area(area);
        self.effect_manager.add_effect(effect);
    }

    pub fn trigger_complete_effect(&mut self, area: Rect) {
        let effect = fx::dissolve(250).with_area(area);
        self.effect_manager.add_effect(effect);
    }

    pub fn trigger_task_complete_celebration(&mut self, area: Rect) {
        // Celebration animation for completing tasks
        self.effect_manager
            .add_effect(fx::fade_to_fg(self.config.theme.green, 500).with_area(area));
    }

    pub fn trigger_streak_animation(&mut self, area: Rect) {
        // Use dissolve effect with color instead of hsl_shift_fg
        self.effect_manager
            .add_effect(fx::fade_to_fg(self.config.theme.magenta, 2000).with_area(area));
    }

    pub fn show_stats_summary(&self) -> String {
        format!(
            "📊 Total: {} tasks | ⏱️ {} hours | 🔥 {} day streak",
            self.stats.total_completed,
            self.stats.total_time_worked.num_hours(),
            self.stats.daily_streak
        )
    }

    pub fn export_to_csv(&self) -> Result<String, std::fmt::Error> {
        let mut csv =
            String::from("Task,Category,Priority,Time Spent,Completed,Created,Completed At\n");
        for task in &self.tasks {
            let category = match &task.category {
                TaskCategory::Work => "Work",
                TaskCategory::Personal => "Personal",
                TaskCategory::Study => "Study",
                TaskCategory::Exercise => "Exercise",
                TaskCategory::Other(name) => name,
            };
            let priority = match task.priority {
                Priority::Low => "Low",
                Priority::Medium => "Medium",
                Priority::High => "High",
                Priority::Urgent => "Urgent",
            };
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                task.description,
                category,
                priority,
                task.timer.get_elapsed().num_minutes(),
                task.completed,
                task.created_at.format("%Y-%m-%d %H:%M"),
                task.completed_at.map_or("N/A".to_string(), |d| d
                    .format("%Y-%m-%d %H:%M")
                    .to_string())
            ));
        }
        Ok(csv)
    }
}
