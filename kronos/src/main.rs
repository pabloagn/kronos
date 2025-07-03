use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

mod app;
mod ui;

use app::{App, AppMode};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = App::new();
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
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        AppMode::Normal => match key.code {
                            KeyCode::Char('q') => return Ok(()),
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
                            KeyCode::Char('g') => app.global_timer.toggle(),
                            KeyCode::Char('G') => app.global_timer.reset(),
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
                    }
                }
            }
        }
    }
}
