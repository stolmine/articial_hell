use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::game::{GamePhase, GameState};
use crate::theme::Theme;

pub fn render_title(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    let lines = vec![
        Line::from(Span::styled(
            "ARTICIAL HELL",
            Style::default().fg(theme.heading).add_modifier(Modifier::BOLD),
        )).centered(),
        Line::from(Span::styled("— Tarot Strategy —", Style::default().fg(theme.text))).centered(),
        Line::from("").centered(),
        Line::from(Span::styled(
            "[N] New Campaign  [Q] Quit",
            Style::default().fg(theme.muted),
        )).centered(),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered()).centered(),
        area,
    );
}

pub fn render_game_over(frame: &mut Frame, game: &GameState) {
    let t = &game.theme;
    let victory = matches!(game.phase, GamePhase::GameOver { victory: true });

    let (heading, heading_color) = if victory {
        ("CAMPAIGN COMPLETE!", t.positive)
    } else {
        ("DEFEATED", t.negative)
    };

    let area = frame.area();
    let lines = vec![
        Line::from(Span::styled(
            heading,
            Style::default().fg(heading_color).add_modifier(Modifier::BOLD),
        )).centered(),
        Line::from(Span::styled("— Tarot Strategy —", Style::default().fg(t.text))).centered(),
        Line::from("").centered(),
        Line::from(Span::styled(
            format!("Fights won: {} / {}", game.fights_won, game.fight),
            Style::default().fg(t.text),
        )).centered(),
        Line::from("").centered(),
        Line::from(Span::styled(
            "[N] New Campaign  [Q] Quit",
            Style::default().fg(t.muted),
        )).centered(),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered()).centered(),
        area,
    );
}
