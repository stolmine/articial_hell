use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::game::{GameState, QUEEN_PERMUTATIONS, MAX_FIGHTS};
use crate::combat::Fighter;
use crate::stats::derive_stats;
use crate::theme::Theme;
use super::widgets;

pub fn render_combat(frame: &mut Frame, game: &GameState) {
    let combat = match game.combat.as_ref() {
        Some(c) => c,
        None => return,
    };
    let t = &game.theme;

    let area = frame.area();
    let [title_area, fighters_area, actions_area, log_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(10),
        Constraint::Length(3),
        Constraint::Min(3),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("ARTICIAL HELL — Fight {}/{} — Cycle {}", game.fight, MAX_FIGHTS, combat.cycle),
            Style::default().fg(t.heading).add_modifier(Modifier::BOLD),
        )).centered())
        .block(Block::bordered()),
        title_area,
    );

    let [player_area, ai_area] = Layout::horizontal([
        Constraint::Ratio(1, 2),
        Constraint::Ratio(1, 2),
    ])
    .areas(fighters_area);

    render_fighter(frame, player_area, &combat.player, "YOU", t);
    render_fighter(frame, ai_area, &combat.ai, "OPPONENT", t);

    if combat.combat_over {
        let (result_text, result_color) = if combat.player_won {
            ("VICTORY — [Space] Continue", t.positive)
        } else {
            ("DEFEATED — [Space] Continue", t.negative)
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                result_text,
                Style::default().fg(result_color).add_modifier(Modifier::BOLD),
            )).centered())
            .block(Block::bordered().title(" Result ")),
            actions_area,
        );
    } else if combat.awaiting_queen_reassign {
        render_queen_reassign(frame, actions_area, game, t);
    } else {
        use crate::combat::CombatAction;
        let actions: [(CombatAction, &str, &str); 3] = [
            (CombatAction::Weapon, combat.player.weapon_action_name(), "Weapon"),
            (CombatAction::Apparel, combat.player.apparel_action_name(), "Apparel"),
            (CombatAction::Item, combat.player.item_action_name(), "Item"),
        ];
        let mut spans: Vec<Span> = Vec::new();
        let is_knight = combat.player.is_knight();
        for (i, (action, name, slot)) in actions.iter().enumerate() {
            if i > 0 { spans.push(Span::raw("   ")); }
            let available = combat.action_available(crate::combat::Side::Player, *action);
            let doubled = is_knight && combat.knight_doubled[0] == Some(*action);
            let suffix = if doubled {
                let uses = combat.knight_action_uses(crate::combat::Side::Player, *action);
                if uses == 0 { " x2" } else { " [1/2]" }
            } else { "" };
            let label = format!("[{}] {} ({}){}", i + 1, name, slot, suffix);
            if available {
                let style = if doubled {
                    Style::default().fg(t.heading).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(t.text)
                };
                spans.push(Span::styled(label, style));
            } else {
                spans.push(Span::styled(label, Style::default().fg(t.muted).add_modifier(Modifier::CROSSED_OUT)));
            }
        }
        let cycle_title = format!(" Actions — Cycle {} ", combat.cycle);
        frame.render_widget(
            Paragraph::new(Line::from(spans).centered())
                .block(Block::bordered().title(cycle_title)),
            actions_area,
        );
    }

    // Combat log: fill available space, show as many entries as fit
    let log_inner_height = log_area.height.saturating_sub(2) as usize; // border takes 2 lines
    let log_entries: Vec<Line> = combat
        .log
        .iter()
        .rev()
        .take(log_inner_height.max(1))
        .rev()
        .map(|entry| Line::from(Span::styled(entry.as_str(), Style::default().fg(t.text))))
        .collect();

    frame.render_widget(
        Paragraph::new(log_entries).block(Block::bordered().title(" Combat Log ")),
        log_area,
    );
}

fn render_fighter(frame: &mut Frame, area: Rect, fighter: &Fighter, label: &str, t: &Theme) {
    let block = Block::bordered().title(format!(" {label} "));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hero_name = fighter.hero.to_string();
    let bar = widgets::hp_bar(fighter.current_hp, fighter.max_hp, 16);
    let hp_line = format!("HP: [{bar}] {}/{}", fighter.current_hp.max(0), fighter.max_hp);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(hero_name, Style::default().fg(t.text).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(hp_line, Style::default().fg(t.positive))),
    ];

    for stat_line in widgets::stat_block(&fighter.stats) {
        lines.push(stat_line);
    }

    lines.push(Line::from(Span::styled(
        format!("Wpn: {}  App: {}", fighter.weapon, fighter.apparel),
        Style::default().fg(t.muted),
    )));
    lines.push(Line::from(Span::styled(
        format!("Itm: {}  Arc: {}", fighter.item, fighter.arcana),
        Style::default().fg(t.muted),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_queen_reassign(frame: &mut Frame, area: Rect, game: &GameState, t: &Theme) {
    let combat = game.combat.as_ref().unwrap();
    let cards = match combat.queen_original_cards[0] {
        Some(c) => c,
        None => return,
    };
    let perm = QUEEN_PERMUTATIONS[game.queen_perm_index];
    let w = cards[perm[0]];
    let a = cards[perm[1]];
    let i = cards[perm[2]];
    let preview = derive_stats(combat.player.hero, w, a, i);
    let text = format!(
        "[</>] Wpn:{} App:{} Itm:{} | ATK:{} DEF:{} HP:{} SPD:{} | [Enter] Confirm",
        w, a, i, preview.attack, preview.defense, preview.hp, preview.speed,
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, Style::default().fg(t.heading).add_modifier(Modifier::BOLD))).centered())
            .block(Block::bordered().title(" Queen Reassign ")),
        area,
    );
}
