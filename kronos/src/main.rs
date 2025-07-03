use anyhow::Result;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod ui;
mod persistence;

use app::{App, AppMode};
use persistence::Persistence;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load or create app
    let app = Persistence::load()?.unwrap_or_else(App::new);
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut last_save = std::time::Instant::now();
    
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Check for timer completions and send notifications
        app.check_and_notify_completions();

        // Auto-save every 5 seconds
        if last_save.elapsed().as_secs() > 5 {
            Persistence::save(&app)?;
            last_save = std::time::Instant::now();
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        AppMode::Normal => match key.code {
                            KeyCode::Char('q') => {
                                Persistence::save(&app)?;
                                return Ok(());
                            }
                            KeyCode::Char('a') => {
                                app.mode = AppMode::AddingTask;
                                app.input_buffer.clear();
                            }
                            KeyCode::Char(' ') => app.toggle_selected_timer(),
                            KeyCode::Char('r') => app.reset_selected_timer(),
                            KeyCode::Char('d') => app.delete_selected_task(),
                            KeyCode::Char('x') => app.toggle_selected_task_completion(),
                            KeyCode::Char('t') => {
                                if !app.tasks.is_empty() {
                                    app.mode = AppMode::EditingTime(app.selected_task);
                                    app.input_buffer.clear();
                                }
                            }
                            KeyCode::Char('p') => {
                                if !app.tasks.is_empty() {
                                    app.mode = AppMode::SelectingPreset(app.selected_task);
                                }
                            }
                            KeyCode::Char('g') => app.global_timer.toggle(),
                            KeyCode::Char('G') => {
                                app.global_timer.reset();
                                app.notifications_sent.retain(|&id| id != 0);
                            }
                            KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
                            _ => {}
                        },
                        AppMode::AddingTask | AppMode::EditingTime(_) => match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input_buffer.clear();
                            }
                            KeyCode::Enter => app.handle_char('\n'),
                            KeyCode::Backspace => app.handle_backspace(),
                            KeyCode::Char(c) => app.handle_char(c),
                            _ => {}
                        },
                        AppMode::SelectingPreset(task_idx) => match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                            }
                            KeyCode::Char(c) if c.is_numeric() => {
                                let num = c.to_digit(10).unwrap_or(0) as usize;
                                if num > 0 && num <= app.presets.len() {
                                    let preset_names = app.get_preset_names();
                                    if let Some(preset_name) = preset_names.get(num - 1) {
                                        app.set_task_duration_from_preset(task_idx, preset_name);
                                        app.mode = AppMode::Normal;
                                    }
                                }
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}
