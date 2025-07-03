use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(6),  // Timer display
            Constraint::Length(3),  // Progress bar
            Constraint::Min(5),     // Tasks
            Constraint::Length(3),  // Help
        ])
        .split(f.area());

    draw_title(f, chunks[0]);
    draw_timer(f, chunks[1], app);
    draw_progress(f, chunks[2], app);
    draw_tasks(f, chunks[3], app);
    draw_help(f, chunks[4]);
}

fn draw_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("Kronos Timer")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn draw_timer(f: &mut Frame, area: Rect, app: &App) {
    let elapsed = app.get_elapsed();
    let hours = elapsed.num_hours();
    let mins = elapsed.num_minutes() % 60;
    let secs = elapsed.num_seconds() % 60;
    
    let time_text = format!("{:02}:{:02}:{:02}", hours, mins, secs);
    
    let status = app.get_status();
    let state_color = match status.state {
        kronos_ipc::TimerState::Running => Color::Green,
        kronos_ipc::TimerState::Paused => Color::Yellow,
        kronos_ipc::TimerState::Idle => Color::Gray,
    };
    
    let timer = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Time: "),
            Span::styled(time_text, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("State: "),
            Span::styled(
                format!("{:?}", status.state),
                Style::default().fg(state_color),
            ),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).title("Timer"));
    
    f.render_widget(timer, area);
}

fn draw_progress(f: &mut Frame, area: Rect, app: &App) {
    let status = app.get_status();
    let progress = if status.total > 0 {
        (status.elapsed as f64 / status.total as f64).min(1.0)
    } else {
        0.0
    };
    
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent((progress * 100.0) as u16);
    
    f.render_widget(gauge, area);
}

fn draw_tasks(f: &mut Frame, area: Rect, app: &App) {
    let tasks = app.get_tasks();
    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let check = if task.completed { "✓" } else { "○" };
            let style = if task.completed {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} {}", check, task.description)).style(style)
        })
        .collect();
    
    let tasks_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"));
    
    f.render_widget(tasks_list, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("Space", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Start/Pause  "),
            Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Reset  "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Quit"),
        ]),
    ];
    
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(help, area);
}
