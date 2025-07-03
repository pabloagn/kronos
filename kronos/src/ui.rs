use crate::app::{App, AppMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Length(8),   // Global timer
            Constraint::Min(10),     // Tasks
            Constraint::Length(4),   // Status/Input
        ])
        .split(f.area());

    draw_header(f, chunks[0]);
    draw_global_timer(f, chunks[1], app);
    draw_tasks(f, chunks[2], app);
    draw_status_bar(f, chunks[3], app);

    // Draw overlays
    match &app.mode {
        AppMode::AddingTask => draw_input_overlay(f, "New Task", &app.input_buffer),
        AppMode::EditingTime(_) => draw_input_overlay(f, "Set Timer (minutes)", &app.input_buffer),
        AppMode::SelectingPreset(_) => draw_preset_overlay(f, app),
        _ => {}
    }
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("⟪ "),
            Span::styled("KRONOS", Style::default().fg(Color::Rgb(127, 180, 202)).add_modifier(Modifier::BOLD)),
            Span::raw(" ⟫"),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(138, 154, 123)))
    );
    f.render_widget(header, area);
}

fn draw_global_timer(f: &mut Frame, area: Rect, app: &App) {
    let timer = &app.global_timer;
    let remaining = timer.get_remaining();
    
    // Display remaining time for countdown
    let hours = remaining.num_hours();
    let mins = remaining.num_minutes() % 60;
    let secs = remaining.num_seconds() % 60;
    
    let state_symbol = match timer.state {
        kronos_ipc::TimerState::Running => "▶",
        kronos_ipc::TimerState::Paused => "⏸",
        kronos_ipc::TimerState::Idle => "■",
    };
    
    let state_color = match timer.state {
        kronos_ipc::TimerState::Running => Color::Rgb(135, 169, 135),
        kronos_ipc::TimerState::Paused => Color::Rgb(230, 195, 132),
        kronos_ipc::TimerState::Idle => Color::Rgb(164, 167, 164),
    };
    
    // Split the area
    let timer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Timer text
            Constraint::Length(3),  // Progress bar
        ])
        .split(area);
    
    // Timer display
    let timer_text = vec![
        Line::from(vec![
            Span::raw("┌─ "),
            Span::styled(state_symbol, Style::default().fg(state_color)),
            Span::raw(" ─┐"),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:02}:{:02}:{:02}", hours, mins, secs),
                Style::default().fg(Color::Rgb(197, 201, 199)).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::raw("└─ "),
            Span::styled(
                format!("{} min", timer.target_duration.num_minutes()),
                Style::default().fg(Color::Rgb(122, 168, 159))
            ),
            Span::raw(" ─┘"),
        ]),
    ];
    
    let timer_widget = Paragraph::new(timer_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("═⟨ Global Timer ⟩═")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Rgb(139, 164, 176)))
        );
    
    f.render_widget(timer_widget, timer_chunks[0]);
    
    // Progress bar
    let progress = timer.get_progress();
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Rgb(127, 180, 202)))
        .percent((progress * 100.0) as u16)
        .label(format!("{}%", (progress * 100.0) as u16));
    
    f.render_widget(gauge, timer_chunks[1]);
}

fn draw_tasks(f: &mut Frame, area: Rect, app: &App) {
    let tasks: Vec<ListItem> = app.tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let is_selected = idx == app.selected_task;
            
            // Task completion symbol
            let check = if task.completed { "☑" } else { "☐" };
            
            // Timer state for this task
            let timer_symbol = match task.timer.state {
                kronos_ipc::TimerState::Running => "◉",
                kronos_ipc::TimerState::Paused => "◈",
                kronos_ipc::TimerState::Idle => "○",
            };
            
            // Timer display (remaining time)
            let remaining = task.timer.get_remaining();
            let timer_text = if task.timer.is_complete() {
                "00:00".to_string()
            } else {
                format!(
                    "{:02}:{:02}",
                    remaining.num_minutes(),
                    remaining.num_seconds() % 60
                )
            };
            
            // Progress bar using block elements
            let progress = task.timer.get_progress();
            let bar_width = 10;
            let filled = (progress * bar_width as f64) as usize;
            let bar = format!(
                "{}{}",
                "█".repeat(filled),
                "░".repeat(bar_width - filled)
            );
            
            // Build the line
            let mut spans = vec![
                Span::raw(if is_selected { "▸ " } else { "  " }),
                Span::raw(format!("{} ", check)),
                Span::styled(
                    &task.description,
                    if task.completed {
                        Style::default().fg(Color::Rgb(164, 167, 164)).add_modifier(Modifier::CROSSED_OUT)
                    } else if is_selected {
                        Style::default().fg(Color::Rgb(230, 195, 132))
                    } else {
                        Style::default()
                    }
                ),
            ];
            
            // Add timer info
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[{} {} {}]", timer_symbol, timer_text, bar),
                Style::default().fg(Color::Rgb(122, 168, 159))
            ));
            
            ListItem::new(Line::from(spans))
        })
        .collect();
    
    let tasks_widget = List::new(tasks)
        .block(
            Block::default()
                .title("═⟨ Tasks ⟩═")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(138, 154, 123)))
        );
    
    f.render_widget(tasks_widget, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let mode_text = match app.mode {
        AppMode::Normal => "NORMAL",
        AppMode::AddingTask => "INSERT",
        AppMode::EditingTime(_) => "TIME",
        AppMode::SelectingPreset(_) => "PRESET",
    };
    
    let mode_color = match app.mode {
        AppMode::Normal => Color::Rgb(122, 168, 159),
        AppMode::AddingTask => Color::Rgb(230, 195, 132),
        AppMode::EditingTime(_) => Color::Rgb(127, 180, 202),
        AppMode::SelectingPreset(_) => Color::Rgb(147, 146, 169),
    };
    
    let help_text = match app.mode {
        AppMode::Normal => {
            vec![
                Span::raw("⟪"),
                Span::styled(mode_text, Style::default().fg(mode_color).add_modifier(Modifier::BOLD)),
                Span::raw("⟫ "),
                Span::raw("│ "),
                Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":add "),
                Span::styled("space", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":toggle "),
                Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":reset "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":delete "),
                Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":time "),
                Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":preset "),
                Span::styled("x", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":done "),
                Span::styled("g/G", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":global "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":quit"),
            ]
        }
        _ => {
            vec![
                Span::raw("⟪"),
                Span::styled(mode_text, Style::default().fg(mode_color).add_modifier(Modifier::BOLD)),
                Span::raw("⟫ "),
                Span::raw("│ "),
                Span::styled("enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":confirm "),
                Span::styled("esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(":cancel"),
            ]
        }
    };
    
    let status = Paragraph::new(Line::from(help_text))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Rgb(57, 59, 68)))
        );
    
    f.render_widget(status, area);
}

fn draw_input_overlay(f: &mut Frame, title: &str, input: &str) {
    let area = centered_rect(50, 20, f.area());
    
    f.render_widget(Clear, area);
    
    let input_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("▸ "),
            Span::raw(input),
            Span::styled("▊", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]),
    ])
    .block(
        Block::default()
            .title(format!("⟨ {} ⟩", title))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Rgb(230, 195, 132)))
    );
    
    f.render_widget(input_widget, area);
}

fn draw_preset_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.area());
    
    f.render_widget(Clear, area);
    
    let presets = app.get_preset_names();
    let items: Vec<ListItem> = presets
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let minutes = app.presets.get(name).unwrap_or(&0);
            let line = Line::from(vec![
                Span::styled(
                    format!("{}", idx + 1),
                    Style::default().fg(Color::Rgb(127, 180, 202)).add_modifier(Modifier::BOLD)
                ),
                Span::raw(". "),
                Span::raw(name),
                Span::raw(" ⟨"),
                Span::styled(
                    format!("{}m", minutes),
                    Style::default().fg(Color::Rgb(122, 168, 159))
                ),
                Span::raw("⟩"),
            ]);
            ListItem::new(line)
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .title("⟨ Select Preset ⟩")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Rgb(147, 146, 169)))
        );
    
    f.render_widget(list, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
