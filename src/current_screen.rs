use std::cmp::Ordering;

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Bar, BarChart, BarGroup, Block, Borders, Clear, List, ListItem, Padding, Paragraph,
        Scrollbar, ScrollbarOrientation, Wrap,
    },
    Frame,
};

use crate::app::App;

#[derive(Clone, Copy, Default)]
pub enum CurrentScreen {
    #[default]
    Overview,
    ProjectList,
    Project,
    DependabotDetails,
    Update,
    Updating,
}

pub fn render_screen(app: &mut App, frame: &mut Frame, chunks: &[Rect]) {
    match app.current_screen {
        CurrentScreen::Overview => render_overview(app, frame, chunks),
        CurrentScreen::ProjectList => render_project_list(app, frame, chunks),
        CurrentScreen::Project => render_project(app, frame, chunks),
        CurrentScreen::DependabotDetails => render_dependabot_details(app, frame, chunks),
        _ => {}
    }
}

pub fn render_popup(app: &mut App, frame: &mut Frame) {
    match app.current_screen {
        CurrentScreen::Update => render_update_popup(frame),
        CurrentScreen::Updating => render_updating_popup(app, frame),
        _ => {}
    }
}

pub fn get_key_hint_text(app: &App) -> Span {
    match app.current_screen {
        CurrentScreen::Overview => Span::styled(
            "(r) to view repositories / (u) to update repositories / (q) to quit",
            Style::default().fg(Color::Red),
        ),
        CurrentScreen::ProjectList => Span::styled(
            "(↑/↓) to navigate / (enter) to view repository / (q) to quit / (o) to view overview / (u) to update repositories",
            Style::default().fg(Color::Red),
        ),
        CurrentScreen::Update => Span::styled(
            "(y/n) to confirm update",
            Style::default().fg(Color::Red),
        ),
        CurrentScreen::Updating => Span::styled(
            "(y/n) to confirm update",
            Style::default().fg(Color::Red),
        ),
        CurrentScreen::Project => Span::styled(
            "(q) to quit / (o) to view overview / (r) to view repositories / (tab) to switch tabs",
            Style::default().fg(Color::Red),
        ),
        CurrentScreen::DependabotDetails => Span::styled(
            "(↑/↓) to navigate / (q) to quit / (r) to view repositories / (tab) to switch tabs",
            Style::default().fg(Color::Red),
        ),
    }
}

pub fn get_navigation_text(app: &App) -> Span {
    match app.current_screen {
        CurrentScreen::Overview => Span::styled("Overview", Style::default().fg(Color::Green)),
        CurrentScreen::ProjectList => {
            Span::styled("Repository List", Style::default().fg(Color::Yellow))
        }
        CurrentScreen::Project => {
            if let Some(current_repo) = app.repositories.get_selected_repository() {
                Span::styled(
                    current_repo.name.clone(),
                    Style::default().fg(Color::Yellow),
                )
            } else {
                Span::styled("Repository", Style::default().fg(Color::Yellow))
            }
        }
        CurrentScreen::DependabotDetails => Span::styled(
            app.repositories
                .get_selected_repository()
                .unwrap()
                .name
                .clone(),
            Style::default().fg(Color::Yellow),
        ),
        CurrentScreen::Update => Span::styled("Updating", Style::default().fg(Color::LightRed)),
        CurrentScreen::Updating => Span::styled("Updating", Style::default().fg(Color::LightRed)),
    }
    .to_owned()
}

fn render_overview(app: &mut App, frame: &mut Frame, chunks: &[Rect]) {
    let repository_count = app.repositories.repos.len();
    let mut low_alerts_count = 0;
    let mut medium_alerts_count = 0;
    let mut high_alerts_count = 0;
    let mut critical_alerts_count = 0;
    if repository_count > 0 {
        low_alerts_count = app
            .repositories
            .repos
            .iter()
            .map(|r| r.low_alerts as u64)
            .sum();
        medium_alerts_count = app
            .repositories
            .repos
            .iter()
            .map(|r| r.medium_alerts as u64)
            .sum();
        high_alerts_count = app
            .repositories
            .repos
            .iter()
            .map(|r| r.high_alerts as u64)
            .sum();
        critical_alerts_count = app
            .repositories
            .repos
            .iter()
            .map(|r| r.critical_alerts as u64)
            .sum();
    }
    let title = format!("Alert Levels for {} Repositories", repository_count);

    let barchart = get_dependabot_bar_chart(
        &title,
        low_alerts_count,
        medium_alerts_count,
        high_alerts_count,
        critical_alerts_count,
    );

    frame.render_widget(barchart, chunks[1]);
}

fn render_project_list(app: &mut App, frame: &mut Frame, chunks: &[Rect]) {
    let mut list_repos = Vec::<ListItem>::new();

    for repo in app.repositories.repos.iter() {
        list_repos.push(ListItem::new(Line::from(Span::styled(
            format!("{: <35} : {} alerts", repo.name, repo.total_active_alerts),
            Style::default().fg(Color::Yellow),
        ))));
    }

    let list = List::new(list_repos)
        .highlight_style(Style::default().fg(Color::Blue))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, chunks[1], app.repositories.get_mut_state());
}

fn render_project(app: &mut App, frame: &mut Frame, chunks: &[Rect]) {
    let tab_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(chunks[1]);
    let project_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(tab_chunks[1]);

    let current_repo = app.current_repository.as_ref().unwrap();
    let mut lines = Vec::<Line>::new();
    lines.push(Line::from(vec![Span::styled(
        format!("ID: {}", current_repo.id),
        Style::default().fg(Color::Blue),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("Name: {}", current_repo.name),
        Style::default().fg(Color::Blue),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("Private: {}", current_repo.private),
        Style::default().fg(Color::Blue),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("URL: {}", current_repo.url),
        Style::default().fg(Color::Blue),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("Archived: {}", current_repo.archived),
        Style::default().fg(Color::Blue),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("Total active alerts: {}", current_repo.total_active_alerts),
        Style::default().fg(Color::Blue),
    )]));

    let project_info = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    let title = format!("Alert Levels for {}", current_repo.name);

    let barchart = get_dependabot_bar_chart(
        &title,
        current_repo.low_alerts as u64,
        current_repo.medium_alerts as u64,
        current_repo.high_alerts as u64,
        current_repo.critical_alerts as u64,
    );

    frame.render_widget(get_tab_info(app), tab_chunks[0]);
    frame.render_widget(project_info, project_chunks[0]);
    frame.render_widget(barchart, project_chunks[1]);
}

fn render_dependabot_details(app: &mut App, frame: &mut Frame, chunks: &[Rect]) {
    let tab_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(chunks[1]);

    let current_repo = app.current_repository.as_ref().unwrap();
    let dependabots: Vec<Line> = current_repo
        .dependabots
        .iter()
        .flat_map(|dependabot| dependabot.to_text())
        .collect();
    let dependabot_line_count = dependabots.len();
    let resized_window = app.chunk_height != tab_chunks[1].height;

    let paragraph = Paragraph::new(dependabots)
        .scroll((app.scrollbar.position as u16, 0))
        .block(Block::default().borders(Borders::RIGHT));

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

    if resized_window {
        match app
            .scrollbar
            .get_length()
            .cmp(&(tab_chunks[1].height as usize))
        {
            Ordering::Greater => {
                app.scrollbar
                    .resize(dependabot_line_count - tab_chunks[1].height as usize);
            }
            Ordering::Less => {
                if dependabot_line_count < tab_chunks[1].height as usize {
                    app.scrollbar.resize(0);
                } else {
                    app.scrollbar
                        .resize(dependabot_line_count - tab_chunks[1].height as usize);
                }
            }
            Ordering::Equal => {
                app.scrollbar.resize(0);
            }
        }
    }
    app.chunk_height = tab_chunks[1].height;

    frame.render_widget(get_tab_info(app), tab_chunks[0]);
    frame.render_widget(paragraph, tab_chunks[1]);
    frame.render_stateful_widget(
        scrollbar,
        tab_chunks[1].inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        app.scrollbar.get_mut_state(),
    );
}

fn get_tab_info(app: &App) -> Paragraph {
    let mut lines = Vec::<Line>::new();
    let mut project_style = Style::default().fg(Color::Green).underlined();
    let mut dependabot_style = Style::default().fg(Color::Blue);

    if let CurrentScreen::DependabotDetails = app.current_screen {
        project_style = Style::default().fg(Color::Blue);
        dependabot_style = Style::default().fg(Color::Green).underlined();
    }

    lines.push(Line::from(vec![
        Span::styled("Project", project_style),
        Span::styled(" | ", Style::default().fg(Color::Blue)),
        Span::styled("Dependabot Details", dependabot_style),
    ]));

    Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true })
}

fn render_update_popup(frame: &mut Frame) {
    frame.render_widget(Clear, frame.size()); //this clears the entire screen and anything already drawn
    let popup_block = Block::default()
        .title("Repositories Update")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default());

    let update_text = Text::styled(
        "Would you like to update the current list of repositories? (y/n)",
        Style::default().fg(Color::Red),
    );
    // the `trim: false` will stop the text from being cut off when over the edge of the block
    let update_paragraph = Paragraph::new(update_text)
        .block(popup_block)
        .wrap(Wrap { trim: false });

    let area = centered_rect(60, 25, frame.size());
    frame.render_widget(update_paragraph, area);
}

fn render_updating_popup(app: &mut App, frame: &mut Frame) {
    frame.render_widget(Clear, frame.size()); //this clears the entire screen and anything already drawn

    let spinner = throbber_widgets_tui::Throbber::default()
        .label("Fetching GitHub Repositories...")
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
        .throbber_style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Red)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::BRAILLE_SIX);

    let area = centered_rect(60, 25, frame.size());
    frame.render_stateful_widget(spinner, area, &mut app.spinner_state);
}

fn get_dependabot_bar_chart(
    title: &str,
    low_alerts_count: u64,
    medium_alerts_count: u64,
    high_alerts_count: u64,
    critical_alerts_count: u64,
) -> BarChart {
    let barchart = BarChart::default()
        .data(
            BarGroup::default().bars(&[
                Bar::default()
                    .label("Low Alerts".into())
                    .value(low_alerts_count)
                    .style(Style::default().fg(Color::Blue)),
                Bar::default()
                    .label("Medium Alerts".into())
                    .value(medium_alerts_count)
                    .style(Style::default().fg(Color::Green)),
                Bar::default()
                    .label("High Alerts".into())
                    .value(high_alerts_count)
                    .style(Style::default().fg(Color::Rgb(255, 165, 0))),
                Bar::default()
                    .label("Critical Alerts".into())
                    .value(critical_alerts_count)
                    .style(Style::default().fg(Color::Red)),
            ]),
        )
        .bar_width(3)
        .block(Block::default().title(title).padding(Padding::vertical(1)))
        .direction(Direction::Horizontal);

    barchart
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
