use crate::app::{App, AppMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph},
    Frame,
};
use tachyonfx::{Duration as TachyonDuration, EffectRenderer};

#[derive(Default, Clone)]
pub struct UiLayout {
    pub tasks: Vec<Rect>,
    pub status_bar: Rect,
}

impl EffectRenderer<u32> for UiLayout {
    fn render_effect(&mut self, _key: &mut u32, _area: Rect, _delta: TachyonDuration) {}
}

pub fn draw(f: &mut Frame, app: &mut App) -> UiLayout {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(f, chunks[0], app);
    draw_global_timer(f, chunks[1], app);
    let task_rects = draw_tasks(f, chunks[2], app);
    draw_status_bar(f, chunks[3], app);

    match &app.mode {
        AppMode::AddingTask => draw_input_overlay(f, "New Task", &app.input_buffer, app),
        AppMode::EditingTime(_) => {
            draw_input_overlay(f, "Set Timer (minutes)", &app.input_buffer, app)
        }
        AppMode::SelectingPreset(_) => draw_preset_overlay(f, app),
        _ => {}
    }
    UiLayout {
        tasks: task_rects,
        status_bar: chunks[3],
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.config.theme;
    let icons = &app.config.icons;
    let text = Line::from(vec![
        Span::raw(icons.header_left.clone()),
        Span::styled(
            "KRONOS",
            Style::default().fg(theme.blue).add_modifier(Modifier::BOLD),
        ),
        Span::raw(icons.header_right.clone()),
    ]);
    f.render_widget(
        Paragraph::new(text).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.black)),
        ),
        area,
    );
}

fn draw_global_timer(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.config.theme;
    let icons = &app.config.icons;
    let timer = &app.global_timer;
    let remaining = timer.get_remaining();
    let time_str = format!(
        "{:02}:{:02}:{:02}",
        remaining.num_hours(),
        remaining.num_minutes() % 60,
        remaining.num_seconds() % 60
    );
    let block = Block::default()
        .title(Span::styled(
            format!(" {} Global ", icons.global_timer),
            Style::default().fg(theme.gray),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.green));
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner_area);
    f.render_widget(
        Paragraph::new(time_str)
            .style(
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
        v_chunks[0],
    );
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(theme.blue).bg(theme.black))
            .percent((timer.get_progress() * 100.0) as u16),
        v_chunks[1],
    );
}

fn draw_tasks(f: &mut Frame, area: Rect, app: &App) -> Vec<Rect> {
    let theme = &app.config.theme;
    let icons = &app.config.icons;
    let block = Block::default()
        .title(Span::styled(
            format!(" {} Tasks ", icons.task_list),
            Style::default().fg(theme.gray),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.green));
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    if app.tasks.is_empty() {
        f.render_widget(
            Paragraph::new("No tasks. Press 'a' to add one.")
                .style(Style::default().fg(theme.gray))
                .alignment(Alignment::Center),
            inner_area,
        );
        return vec![];
    }
    let constraints: Vec<Constraint> = app.tasks.iter().map(|_| Constraint::Length(1)).collect();
    let task_chunks = Layout::default().constraints(constraints).split(inner_area);
    for (i, task) in app.tasks.iter().enumerate() {
        if let Some(item_area) = task_chunks.get(i) {
            let mut left = vec![if i == app.selected_task {
                Span::styled(icons.select.clone(), Style::default().fg(theme.selection))
            } else {
                Span::raw(" ")
            }];
            left.push(Span::raw(format!(
                " {} ",
                if task.completed {
                    &icons.done
                } else {
                    &icons.pending
                }
            )));
            left.push(Span::styled(
                task.description.clone(),
                if task.completed {
                    Style::default()
                        .fg(theme.gray)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(theme.foreground)
                },
            ));
            let state_icon = match task.timer.state {
                kronos_ipc::TimerState::Running => &icons.play,
                kronos_ipc::TimerState::Paused => &icons.pause,
                kronos_ipc::TimerState::Idle => &icons.stop,
            };
            let rem = task.timer.get_remaining();
            let timer_txt = format!(
                "{:02}:{:02}",
                rem.num_minutes().max(0),
                (rem.num_seconds() % 60).max(0)
            );
            let bar = format!(
                "{}{}",
                icons
                    .progress_filled
                    .repeat((task.timer.get_progress() * 10.0) as usize),
                icons
                    .progress_empty
                    .repeat(10 - (task.timer.get_progress() * 10.0) as usize)
            );
            let right = Span::styled(
                format!(" {} {} {} ", state_icon, timer_txt, bar),
                Style::default().fg(theme.cyan),
            );
            if i == app.selected_task {
                f.render_widget(
                    Block::default().style(Style::default().bg(theme.black)),
                    *item_area,
                );
            }
            f.render_widget(Paragraph::new(Line::from(left)), *item_area);
            f.render_widget(
                Paragraph::new(Line::from(right)).alignment(Alignment::Right),
                *item_area,
            );
        }
    }
    task_chunks.to_vec()
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.config.theme;
    let (mode_text, mode_color) = match app.mode {
        AppMode::Normal => ("NORMAL", theme.green),
        AppMode::AddingTask => ("INSERT", theme.yellow),
        AppMode::EditingTime(_) => ("TIME", theme.blue),
        AppMode::SelectingPreset(_) => ("PRESET", theme.magenta),
        AppMode:: ShowStats => ("STATS", theme.magenta),
        AppMode::ShowHelp => ("HELP", theme.magenta),
        AppMode::StartupAnimation => ("NORMAL", theme.magenta),
    };
    let help = if app.mode == AppMode::Normal {
        "a:add │ d:del │ t:time │ p:preset │ r:reset │ x:done │ g/G:global │ q:quit"
    } else {
        "enter:confirm │ esc:cancel"
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {} ", mode_text),
                Style::default()
                    .bg(mode_color)
                    .fg(theme.background)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(help),
        ]))
        .block(Block::default().style(Style::default().bg(theme.black).fg(theme.gray))),
        area,
    );
}

fn draw_input_overlay(f: &mut Frame, title: &str, input: &str, app: &App) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.config.theme.yellow))
        .border_type(BorderType::Double)
        .style(Style::default().bg(app.config.theme.background));
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▸ ", Style::default().fg(app.config.theme.foreground)),
            Span::styled(input, Style::default().fg(app.config.theme.foreground)),
            Span::styled(
                &app.config.icons.input_cursor,
                Style::default()
                    .fg(app.config.theme.foreground)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ])),
        inner_area,
    );
}

fn draw_preset_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 40, f.area());
    f.render_widget(Clear, area);
    let items: Vec<ListItem> = app
        .get_preset_names()
        .iter()
        .enumerate()
        .map(|(i, name)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}. ", i + 1),
                    Style::default().fg(app.config.theme.blue),
                ),
                Span::raw(name.clone()),
                Span::styled(
                    format!(" ({}m)", app.presets.get(name).unwrap_or(&0)),
                    Style::default().fg(app.config.theme.gray),
                ),
            ]))
        })
        .collect();
    f.render_widget(
        List::new(items).block(
            Block::default()
                .title(" Select Preset ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(app.config.theme.magenta)),
        ),
        area,
    );
}

fn draw_help_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let shortcuts = vec![
        (
            "General",
            vec![
                ("q", "Quit"),
                ("?", "Toggle help"),
                ("Tab", "Switch themes"),
            ],
        ),
        (
            "Tasks",
            vec![
                ("a", "Add task"),
                ("d", "Delete task"),
                ("x", "Toggle complete"),
                ("Space", "Start/pause timer"),
                ("r", "Reset timer"),
            ],
        ),
        (
            "Navigation",
            vec![
                ("j/↓", "Move down"),
                ("k/↑", "Move up"),
                ("gg", "Go to top"),
                ("G", "Go to bottom"),
            ],
        ),
    ];
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
