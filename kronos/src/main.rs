use tachyonfx::{fx, Motion};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

mod app;
mod config;
mod persistence;
mod ui;

use app::{App, AppMode};
use persistence::Persistence;
use ui::UiLayout;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = config::load_config()?;
    let mut app = Persistence::load(&config)?.unwrap_or_else(|| App::new(config.clone()));
    app.config = config;

    let res = run_app(&mut terminal, &mut app);

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    let mut last_save = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut ui_layout = UiLayout::default();

    // Add startup animation here
    if app.mode == AppMode::StartupAnimation {
        app.effect_manager.add_effect(fx::sweep_in(
            Motion::UpToDown,
            20,
            0,
            app.config.theme.selection,
            800,
        ));
        app.mode = AppMode::Normal;
    }

    loop {
        let now = Instant::now();
        let delta = now.duration_since(last_frame_time);
        last_frame_time = now;

        terminal.draw(|f| {
            let frame_area = f.area();
            ui_layout = ui::draw(f, app);
            app.effect_manager
                .process_effects(delta.into(), f.buffer_mut(), frame_area);
        })?;

        app.check_and_notify_completions();

        if last_save.elapsed() > Duration::from_secs(5) {
            if Persistence::save(app).is_ok() {
                last_save = Instant::now();
            }
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let prev_mode = app.mode.clone();
                    match app.mode {
                        AppMode::Normal => match key.code {
                            KeyCode::Char('q') => app.should_quit = true,
                            KeyCode::Char('d') => {
                                if let Some(rect) = ui_layout.tasks.get(app.selected_task) {
                                    app.trigger_delete_effect(*rect);
                                }
                                app.delete_selected_task();
                            }
                            KeyCode::Char('x') => {
                                if let Some(task) = app.tasks.get(app.selected_task) {
                                    if !task.completed {
                                        if let Some(rect) = ui_layout.tasks.get(app.selected_task) {
                                            app.trigger_complete_effect(*rect);
                                        }
                                    }
                                }
                                app.toggle_selected_task_completion();
                            }
                            KeyCode::Char('a') => {
                                app.mode = AppMode::AddingTask;
                                app.input_buffer.clear();
                            }
                            KeyCode::Char(' ') => app.toggle_selected_timer(),
                            KeyCode::Char('r') => app.reset_selected_timer(),
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
                        _ => match key.code {
                            KeyCode::Enter => app.handle_char('\n'),
                            KeyCode::Esc => app.mode = AppMode::Normal,
                            KeyCode::Backspace => app.handle_backspace(),
                            KeyCode::Char(c) => app.handle_char(c),
                            _ => {}
                        },
                    }
                    if app.mode != prev_mode {
                        app.trigger_mode_change_effect(ui_layout.status_bar);
                    }
                }
            }
        }
        if app.should_quit {
            Persistence::save(app)?;
            break;
        }
    }
    Ok(())
}
