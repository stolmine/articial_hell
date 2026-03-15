use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::card::TarotCard;
use crate::stats::Stats;
use crate::theme::Theme;
use super::card_art;

pub fn render_card_widget(frame: &mut Frame, area: Rect, card: &TarotCard, index: usize, focused: bool, theme: &Theme) {
    let (base_color, art_lines, name_line) = match card {
        TarotCard::Court { suit, rank } => {
            let color = theme.suit_color(*suit);
            let art = card_art::suit_art(*suit);
            let name = format!("{rank} of {suit}");
            (color, art, name)
        }
        TarotCard::Numbered { suit, value } => {
            let color = theme.suit_color(*suit);
            let art = card_art::suit_art(*suit);
            let label = if *value == 1 { "Ace".to_string() } else { value.to_string() };
            let name = format!("{label} of {suit}");
            (color, art, name)
        }
        TarotCard::Major(arcana) => {
            let art = card_art::arcana_art();
            let name = format!("{arcana}");
            (theme.fate, art, name)
        }
    };

    let border_color = if focused { theme.card_focus_border } else { base_color };
    let border_type = if focused { BorderType::Double } else { BorderType::Plain };
    let label_style = if focused {
        Style::default().fg(theme.card_focus_label).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };

    let block = Block::bordered()
        .border_style(Style::default().fg(border_color))
        .border_type(border_type)
        .title(Span::styled(format!("[{index}]"), label_style));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text_color = if focused { theme.text } else { base_color };
    let mut lines: Vec<Line> = art_lines
        .iter()
        .map(|row| Line::from(Span::styled(*row, Style::default().fg(text_color))).centered())
        .collect();
    lines.push(Line::from("").centered());
    lines.push(
        Line::from(Span::styled(name_line, Style::default().fg(text_color).add_modifier(Modifier::BOLD)))
            .centered(),
    );

    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn hp_bar(current: i32, max: i32, width: usize) -> String {
    let filled = if max <= 0 {
        0
    } else {
        ((current.max(0) as usize) * width) / (max as usize)
    };
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

pub fn stat_block(stats: &Stats) -> Vec<Line<'static>> {
    vec![
        Line::from(format!("ATK: {:>3}  SPD: {:>3}", stats.attack, stats.speed)),
        Line::from(format!("HP:  {:>3}  DEF: {:>3}", stats.hp, stats.defense)),
    ]
}
