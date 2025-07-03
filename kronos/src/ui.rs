use crate::app::{App, AppMode, TaskCategory};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Table},
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
        AppMode::SelectingCategory(_) => draw_category_overlay(f, app),
        AppMode::ShowStats => draw_stats_overlay(f, app),
        AppMode::ShowHelp => draw_help_overlay(f, app),
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
    let time_str = if app.config.features.show_seconds {
        format!(
            "{:02}:{:02}:{:02}",
            remaining.num_hours(),
            remaining.num_minutes() % 60,
            remaining.num_seconds() % 60
        )
    } else {
        format!(
            "{:02}:{:02}",
            remaining.num_hours() * 60 + remaining.num_minutes(),
            remaining.num_seconds() % 60
        )
    };
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
                Span::styled(
                    icons.select.clone(),
                    Style::default().fg(theme.selection),
                )
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
            left.push(Span::styled(
                format!(" ({})", task.category.to_string()),
                Style::default().fg(theme.yellow),
            ));

            let state_icon = match task.timer.state {
                kronos_ipc::TimerState::Running => &icons.play,
                kronos_ipc::TimerState::Paused => &icons.pause,
                kronos_ipc::TimerState::Idle => &icons.stop,
            };

            let rem = task.timer.get_remaining();
            let timer_txt = if app.config.features.show_seconds {
                format!(
                    "{:02}:{:02}",
                    rem.num_minutes().max(0),
                    (rem.num_seconds() % 60).max(0)
                )
            } else {
                format!("{:02}m", rem.num_minutes().max(0))
            };

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
        AppMode::SelectingCategory(_) => ("CATEGORY", theme.cyan),
        AppMode::ShowStats => ("STATS", theme.magenta),
        AppMode::ShowHelp => ("HELP", theme.magenta),
        AppMode::StartupAnimation => ("NORMAL", theme.magenta),
    };

    let help = match app.mode {
        AppMode::Normal => "a:add | d:del | x:done | t:time | p:preset | c:cat | r:reset | s:stats | ?:help | q:quit",
        _ => "enter:confirm | esc:cancel",
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
    let area = centered_rect(60, 60, f.area());
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

fn draw_category_overlay(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 40, f.area());
    f.render_widget(Clear, area);
    let items: Vec<ListItem> = app
        .get_category_names()
        .iter()
        .map(|name| ListItem::new(Line::from(vec![Span::raw(name.clone())])))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Select Category ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(app.config.theme.cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(app.config.theme.selection)
                .fg(app.config.theme.background),
        )
        .highlight_symbol(&app.config.icons.select);

    f.render_stateful_widget(list, area, &mut app.category_list_state);
}

fn draw_stats_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Statistics ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(app.config.theme.magenta));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let stats_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(inner_area);

    let summary_text = vec![
        Line::from(vec![
            Span::styled("Tasks Completed: ", Style::default().fg(app.config.theme.blue)),
            Span::raw(app.stats.total_completed.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Total Time Worked: ", Style::default().fg(app.config.theme.blue)),
            Span::raw(format!("{} hours", app.stats.total_time_worked.num_hours())),
        ]),
        Line::from(vec![
            Span::styled("Daily Streak: ", Style::default().fg(app.config.theme.blue)),
            Span::raw(format!("{} days", app.stats.daily_streak)),
        ]),
    ];

    f.render_widget(Paragraph::new(summary_text), stats_chunks[0]);

    let category_rows = app.stats.tasks_by_category.iter().map(|(category, count)| {
        ratatui::widgets::Row::new(vec![
            category.to_string(),
            count.to_string(),
        ])
    });

    let category_table = Table::new(
        category_rows,
        &[Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(ratatui::widgets::Row::new(vec!["Category", "Tasks"]).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(
        Block::default()
            .title("Tasks by Category")
            .borders(Borders::TOP)
            .border_style(Style::default().fg(app.config.theme.gray)),
    );

    f.render_widget(category_table, stats_chunks[1]);
}

fn draw_help_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let shortcuts = vec![
        (
            "General",
            vec![
                ("q", "Quit"),
                ("s", "Show Stats"),
                ("?", "Toggle help"),
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
                ("t", "Set time"),
                ("p", "Select preset"),
                ("c", "Change category"),
            ],
        ),
        (
            "Navigation",
            vec![
                ("j/↓", "Move down"),
                ("k/↑", "Move up"),
            ],
        ),
        (
            "Global Timer",
            vec![
                ("g", "Start/pause global timer"),
                ("G", "Reset global timer"),
            ],
        ),
    ];

    let mut lines = vec![];
    for (section, keys) in shortcuts {
        lines.push(Line::from(Span::styled(
            section,
            Style::default()
                .fg(app.config.theme.blue)
                .add_modifier(Modifier::BOLD),
        )));
        for (key, desc) in keys {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:>6} : ", key), Style::default().fg(app.config.theme.yellow)),
                Span::raw(desc),
            ]));
        }
        lines.push(Line::from(""));
    }

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(app.config.theme.magenta)),
        ),
        area,
    );
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
