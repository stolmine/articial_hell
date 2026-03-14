use ratatui::{prelude::*, widgets::*};

use crate::card::{Card, Suit};
use crate::game::{GamePhase, GameState, WeaponState};

pub fn draw_ui(frame: &mut Frame, game: &GameState) {
    match game.phase {
        GamePhase::Title => render_title_screen(frame),
        GamePhase::Won => render_end_screen(frame, game, true),
        GamePhase::Dead => render_end_screen(frame, game, false),
        _ => render_game(frame, game),
    }
}

fn render_title_screen(frame: &mut Frame) {
    let area = frame.area();
    let lines = vec![
        Line::from(Span::styled(
            "ARTICIAL HELL",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .centered(),
        Line::from("─── Scoundrel ───").centered(),
        Line::from("").centered(),
        Line::from(Span::styled(
            "Press [N] to start a new game",
            Style::default().fg(Color::Green),
        ))
        .centered(),
        Line::from(Span::styled(
            "[Q] Quit",
            Style::default().fg(Color::DarkGray),
        ))
        .centered(),
    ];
    let paragraph = Paragraph::new(lines)
        .block(Block::bordered())
        .centered();
    frame.render_widget(paragraph, area);
}

fn render_end_screen(frame: &mut Frame, game: &GameState, won: bool) {
    let area = frame.area();
    let (heading, heading_color) = if won {
        ("YOU ESCAPED!", Color::Green)
    } else {
        ("YOU DIED", Color::Red)
    };
    let lines = vec![
        Line::from(Span::styled(
            heading,
            Style::default()
                .fg(heading_color)
                .add_modifier(Modifier::BOLD),
        ))
        .centered(),
        Line::from("─── Scoundrel ───").centered(),
        Line::from("").centered(),
        Line::from(game.message.as_str()).centered(),
        Line::from("").centered(),
        Line::from(Span::styled(
            "[N] New game  [Q] Quit",
            Style::default().fg(Color::DarkGray),
        ))
        .centered(),
    ];
    let paragraph = Paragraph::new(lines)
        .block(Block::bordered())
        .centered();
    frame.render_widget(paragraph, area);
}

fn render_game(frame: &mut Frame, game: &GameState) {
    let area = frame.area();

    let vertical = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(12),
        Constraint::Length(4),
        Constraint::Min(3),
    ]);
    let [title_area, cards_area, status_area, message_area] = vertical.areas(area);

    render_title(frame, title_area);
    render_cards(frame, cards_area, game);
    render_status(frame, status_area, game);
    render_message(frame, message_area, game);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "ARTICIAL HELL",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .centered(),
        Line::from("─── Scoundrel ───").centered(),
    ];
    let paragraph = Paragraph::new(lines).block(Block::bordered());
    frame.render_widget(paragraph, area);
}

fn render_cards(frame: &mut Frame, area: Rect, game: &GameState) {
    let block = Block::bordered().title(" Room ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let horizontal = Layout::horizontal([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ]);
    let card_areas: [Rect; 4] = horizontal.areas(inner);

    for (i, slot) in game.room.iter().enumerate() {
        render_card_slot(frame, card_areas[i], slot.as_ref(), i + 1);
    }
}

fn render_card_slot(frame: &mut Frame, area: Rect, card: Option<&Card>, index: usize) {
    let vertical = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]);
    let [card_area, label_area] = vertical.areas(area);

    match card {
        Some(c) => {
            let color = card_color(c);
            let type_label = card_type_label(c);
            let rank_str = format!("{}", c.rank);
            let suit_art = big_suit(c.suit);

            let mut lines = vec![
                Line::from(Span::styled(
                    format!(" {}", rank_str),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )),
            ];
            for row in &suit_art {
                lines.push(Line::from(Span::styled(
                    *row,
                    Style::default().fg(color),
                )).centered());
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                type_label,
                Style::default().fg(Color::DarkGray),
            )).centered());

            let paragraph = Paragraph::new(lines).block(
                Block::bordered()
                    .border_style(Style::default().fg(color)),
            );
            frame.render_widget(paragraph, card_area);
        }
        None => {
            let paragraph = Paragraph::new(vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled("  --  ", Style::default().fg(Color::DarkGray))).centered(),
            ])
            .block(
                Block::bordered()
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(paragraph, card_area);
        }
    }

    let label = Line::from(Span::styled(
        format!("[{index}]"),
        Style::default().fg(Color::DarkGray),
    ))
    .centered();
    frame.render_widget(Paragraph::new(label), label_area);
}

fn big_suit(suit: Suit) -> [&'static str; 5] {
    match suit {
        Suit::Spades => [
            "  ▄  ",
            " ▟█▙ ",
            "▟███▙",
            "  █  ",
            " ▟▀▙ ",
        ],
        Suit::Clubs => [
            " ▄█▄ ",
            "▐███▌",
            " ▀█▀ ",
            " ▄█▄ ",
            "▀▀▀▀▀",
        ],
        Suit::Hearts => [
            "▗▄ ▄▖",
            "█████",
            "▜███▛",
            " ▜█▛ ",
            "  ▀  ",
        ],
        Suit::Diamonds => [
            "  ▄  ",
            " ▟█▙ ",
            "▟███▙",
            " ▜█▛ ",
            "  ▀  ",
        ],
    }
}

fn card_color(card: &Card) -> Color {
    match card.suit {
        Suit::Hearts | Suit::Diamonds => Color::Red,
        Suit::Clubs | Suit::Spades => Color::White,
    }
}

fn card_type_label(card: &Card) -> String {
    if card.is_monster() {
        format!("{:>3}", card.value())
    } else if card.is_weapon() {
        "WPN".to_string()
    } else {
        " HP".to_string()
    }
}

fn render_status(frame: &mut Frame, area: Rect, game: &GameState) {
    let hp_bar = build_hp_bar(game.hp, game.max_hp, 16);

    let weapon_line = match &game.weapon {
        Some(WeaponState { card, bound_to }) => {
            let bound = match bound_to {
                Some(v) => format!("(bound: ≤{})", v),
                None => String::new(),
            };
            format!("Weapon: {} {}", card, bound)
        }
        None => "Weapon: None (bare hands)".to_string(),
    };

    let lines = vec![
        Line::from(vec![
            Span::raw("HP: "),
            Span::styled(&hp_bar, Style::default().fg(Color::Green)),
            Span::raw(format!("  {}/{}", game.hp, game.max_hp)),
        ]),
        Line::from(weapon_line.as_str()),
        Line::from(format!("Deck: {} remaining", game.deck.remaining())),
    ];

    let paragraph = Paragraph::new(lines).block(Block::bordered().title(" Status "));
    frame.render_widget(paragraph, area);
}

fn build_hp_bar(hp: u32, max_hp: u32, width: usize) -> String {
    let filled = if max_hp == 0 {
        0
    } else {
        ((hp as usize) * width) / (max_hp as usize)
    };
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn render_message(frame: &mut Frame, area: Rect, game: &GameState) {
    let run_hint = if game.can_run { "  [R] Run" } else { "" };
    let controls = format!("[1-4] Play card{}  [N] New game  [Q] Quit", run_hint);

    let lines = vec![
        Line::from(format!("> {}", game.message)),
        Line::from(""),
        Line::from(Span::styled(controls, Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(lines).block(Block::bordered().title(" Messages "));
    frame.render_widget(paragraph, area);
}
