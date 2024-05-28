use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::current_screen::{get_key_hint_text, get_navigation_text, render_popup, render_screen};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(4),
        ])
        .split(f.size());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Dependabot Tracker",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    f.render_widget(title, chunks[0]);

    render_screen(app, f, &chunks);

    let current_navigation_text = vec![
        // The first half of the text
        get_navigation_text(app),
    ];

    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    let current_keys_hint = get_key_hint_text(app);

    let key_notes_footer = Paragraph::new(Line::from(current_keys_hint))
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[2]);

    f.render_widget(mode_footer, footer_chunks[0]);
    f.render_widget(key_notes_footer, footer_chunks[1]);

    render_popup(app, f);
}
