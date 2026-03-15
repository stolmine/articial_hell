use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::card::{TarotCard, CourtRank, MinorSuit};
use crate::game::{GamePhase, GameState, DraftStep, PlayerState};
use crate::stats::{partial_derive, Stats};
use crate::theme::Theme;
use super::{widgets, tooltip};

fn current_stats(player: &PlayerState) -> Stats {
    partial_derive(player.hero, player.weapon, player.apparel, player.item)
}

fn prospective_stats(player: &PlayerState, step: &DraftStep, card: TarotCard) -> Stats {
    let mut p = player.clone();
    match step {
        DraftStep::PickHero => p.hero = Some(card),
        DraftStep::PickWeapon => p.weapon = Some(card),
        DraftStep::PickApparel => p.apparel = Some(card),
        DraftStep::PickItem => p.item = Some(card),
    }
    partial_derive(p.hero, p.weapon, p.apparel, p.item)
}

fn prospective_combat_stats(game: &GameState, step: &DraftStep, card: TarotCard) -> Option<(Stats, Stats)> {
    let prog = game.campaign.as_ref()?;
    let mut p = game.player.clone();
    match step {
        DraftStep::PickHero => p.hero = Some(card),
        DraftStep::PickWeapon => p.weapon = Some(card),
        DraftStep::PickApparel => p.apparel = Some(card),
        DraftStep::PickItem => p.item = Some(card),
    }
    let hero = p.hero?;
    let equip: Vec<TarotCard> = [p.weapon, p.apparel, p.item].into_iter().flatten().collect();
    let prog_delta = crate::progression::progression_bonus(prog, hero, &equip);
    let base = prospective_stats(&game.player, step, card);
    let mut combat = base;
    combat.add(&prog_delta);
    if prog_delta.attack != 0 || prog_delta.speed != 0 || prog_delta.hp != 0 || prog_delta.defense != 0 {
        Some((prog_delta, combat))
    } else {
        None
    }
}

fn stat_diff_line(label: &str, cur: i32, next: i32, t: &Theme) -> Line<'static> {
    if cur == next || (cur == 0 && next == 0) {
        return Line::from(Span::styled(
            format!("{label}: {cur}"),
            Style::default().fg(t.muted),
        ));
    }
    let delta = next - cur;
    let (arrow_color, sign) = if delta > 0 {
        (t.positive, format!("+{delta}"))
    } else {
        (t.negative, format!("{delta}"))
    };
    Line::from(vec![
        Span::styled(format!("{label}: "), Style::default().fg(t.text)),
        Span::styled(format!("{cur}"), Style::default().fg(t.muted)),
        Span::styled(" -> ", Style::default().fg(t.text)),
        Span::styled(format!("{next}"), Style::default().fg(arrow_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" ({sign})"), Style::default().fg(arrow_color)),
    ])
}

pub fn render_draft(frame: &mut Frame, game: &GameState) {
    let (step, choices) = match &game.phase {
        GamePhase::Draft { step, choices, .. } => (step.clone(), choices.clone()),
        _ => return,
    };
    let t = &game.theme;

    let area = frame.area();
    let [title_area, cards_area, info_area, controls_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(12),
        Constraint::Min(6),
        Constraint::Length(3),
    ])
    .areas(area);

    let step_label = match step {
        DraftStep::PickHero => "Hero",
        DraftStep::PickWeapon => "Weapon",
        DraftStep::PickApparel => "Apparel",
        DraftStep::PickItem => "Item",
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            if game.fight == crate::game::MAX_FIGHTS {
                format!("ARTICIAL HELL — FINAL BATTLE — Pick {}", step_label)
            } else {
                format!("ARTICIAL HELL — Fight {}/{} Draft — Pick {}", game.fight, crate::game::MAX_FIGHTS, step_label)
            },
            Style::default().fg(t.heading).add_modifier(Modifier::BOLD),
        )).centered())
        .block(Block::bordered()),
        title_area,
    );

    let card_rects: [Rect; 4] = Layout::horizontal([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ])
    .areas(cards_area);

    for (i, card) in choices.iter().enumerate().take(4) {
        widgets::render_card_widget(frame, card_rects[i], card, i + 1, i == game.cursor, t);
    }

    let [tooltip_area, stats_area] = Layout::horizontal([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ])
    .areas(info_area);

    let tooltip_lines = if game.cursor < choices.len() {
        tooltip::card_tooltip(&choices[game.cursor], &step, &game.player, game.campaign.as_ref(), t)
    } else {
        vec![]
    };
    frame.render_widget(
        Paragraph::new(tooltip_lines).block(Block::bordered().title(" Card Info ")),
        tooltip_area,
    );

    render_stats_pane(frame, stats_area, game, &step, &choices, t);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "◀ ▶ Navigate   [Enter] Pick   [1-4] Quick pick   [Q] Quit",
            Style::default().fg(t.muted),
        )).centered())
        .block(Block::bordered()),
        controls_area,
    );
}

fn render_stats_pane(
    frame: &mut Frame,
    area: Rect,
    game: &GameState,
    step: &DraftStep,
    choices: &[TarotCard],
    t: &Theme,
) {
    let mut lines: Vec<Line> = Vec::new();

    let cur = current_stats(&game.player);
    let has_hero = game.player.hero.is_some();

    if game.cursor < choices.len() {
        let focused = choices[game.cursor];
        let next = prospective_stats(&game.player, step, focused);

        if *step == DraftStep::PickHero {
            lines.push(Line::from(Span::styled("Prospective Stats", Style::default().fg(t.heading).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(""));
            lines.push(stat_diff_line("ATK", 0, next.attack, t));
            lines.push(stat_diff_line("SPD", 0, next.speed, t));
            lines.push(stat_diff_line("HP ", 0, next.hp, t));
            lines.push(stat_diff_line("DEF", 0, next.defense, t));
        } else if has_hero {
            lines.push(Line::from(Span::styled("Stats Preview", Style::default().fg(t.heading).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(""));
            lines.push(stat_diff_line("ATK", cur.attack, next.attack, t));
            lines.push(stat_diff_line("SPD", cur.speed, next.speed, t));
            lines.push(stat_diff_line("HP ", cur.hp, next.hp, t));
            lines.push(stat_diff_line("DEF", cur.defense, next.defense, t));
        }

        // Show actual combat stats when progression is active
        if let Some((prog_delta, combat)) = prospective_combat_stats(game, step, focused) {
            lines.push(Line::from(Span::styled(
                format!("In combat: {}/{}/{}/{}  (+{}A {}S {}H {}D)",
                    combat.attack, combat.speed, combat.hp, combat.defense,
                    prog_delta.attack, prog_delta.speed, prog_delta.hp, prog_delta.defense),
                Style::default().fg(t.info),
            )));
        }
    } else if has_hero {
        lines.push(Line::from(Span::styled("Current Stats", Style::default().fg(t.heading).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(format!("ATK: {}", cur.attack), Style::default().fg(t.text))));
        lines.push(Line::from(Span::styled(format!("SPD: {}", cur.speed), Style::default().fg(t.text))));
        lines.push(Line::from(Span::styled(format!("HP : {}", cur.hp), Style::default().fg(t.text))));
        lines.push(Line::from(Span::styled(format!("DEF: {}", cur.defense), Style::default().fg(t.text))));
    }

    if has_hero {
        lines.push(Line::from(""));
        let opp = opponent_summary(game, step, t);
        lines.extend(opp);
    }

    lines.push(Line::from(""));
    let picks = picks_summary(game, step, t);
    lines.extend(picks);

    // Campaign progression
    if let Some(ref prog) = game.campaign {
        let has_any = prog.suit_affinity.iter().any(|&a| a > 0)
            || prog.diversity_picks > 0
            || prog.matching_picks > 0;
        if has_any {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Campaign", Style::default().fg(t.heading).add_modifier(Modifier::BOLD))));

            for (i, suit) in MinorSuit::ALL.iter().enumerate() {
                let count = prog.suit_affinity[i];
                if count > 0 {
                    let pips = "+".repeat(count as usize);
                    let stat = suit.stat_name();
                    let sname = match suit {
                        MinorSuit::Swords => "Swords",
                        MinorSuit::Wands => "Wands",
                        MinorSuit::Cups => "Cups",
                        MinorSuit::Pentacles => "Pents",
                    };
                    lines.push(Line::from(Span::styled(
                        format!("{sname}: {pips} (+{count} {stat})"),
                        Style::default().fg(t.text),
                    )));
                }
            }

            let hero_rank = game.player.hero.and_then(|h| h.court_rank());
            let tier_str = match hero_rank {
                Some(CourtRank::Page) | Some(CourtRank::Knight) => {
                    let tier = (prog.diversity_picks as i32).min(2);
                    if tier > 0 { Some(format!("Diversity Lv{}: +{}/suit", tier, 2 + tier)) } else { None }
                }
                Some(CourtRank::Queen) | Some(CourtRank::King) => {
                    let tier = (prog.matching_picks as i32).min(2);
                    if tier > 0 { Some(format!("Matching Lv{}: +{}/match", tier, 3 + tier)) } else { None }
                }
                None => None,
            };
            if let Some(ts) = tier_str {
                lines.push(Line::from(Span::styled(ts, Style::default().fg(t.text))));
            }

            let survivor_bonus = prog.fights_survived as i32 / 2;
            if survivor_bonus > 0 {
                lines.push(Line::from(Span::styled(
                    format!("Survivor: +{survivor_bonus} all stats"),
                    Style::default().fg(t.positive),
                )));
            }

            let ai_lvl = crate::progression::ai_scaling_bonus(prog);
            if ai_lvl > 0 {
                lines.push(Line::from(Span::styled(
                    format!("AI Scaling: +{ai_lvl}"),
                    Style::default().fg(t.negative),
                )));
            }

            let boss = crate::progression::ai_boss_bonus(game.fight, crate::game::MAX_FIGHTS);
            if boss > 0 {
                lines.push(Line::from(Span::styled(
                    format!("BOSS: +{boss} all enemy stats"),
                    Style::default().fg(t.negative).add_modifier(Modifier::BOLD),
                )));
            }
        }
    }

    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Build ")),
        area,
    );
}

fn opponent_summary<'a>(game: &GameState, step: &DraftStep, t: &Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let opp = match step {
        DraftStep::PickHero => return lines,
        DraftStep::PickWeapon => format!(
            "Opp: {}",
            game.ai_state.hero.map(|c| c.to_string()).unwrap_or_else(|| "???".into())
        ),
        DraftStep::PickApparel => format!(
            "Opp: {} / {}",
            game.ai_state.hero.map(|c| c.to_string()).unwrap_or_else(|| "???".into()),
            game.ai_state.weapon.map(|c| c.to_string()).unwrap_or_else(|| "???".into()),
        ),
        DraftStep::PickItem => format!(
            "Opp: {} / {} / {}",
            game.ai_state.hero.map(|c| c.to_string()).unwrap_or_else(|| "???".into()),
            game.ai_state.weapon.map(|c| c.to_string()).unwrap_or_else(|| "???".into()),
            game.ai_state.apparel.map(|c| c.to_string()).unwrap_or_else(|| "???".into()),
        ),
    };
    lines.push(Line::from(Span::styled(opp, Style::default().fg(t.muted))));
    lines
}

fn picks_summary<'a>(game: &GameState, step: &DraftStep, t: &Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled("Your Picks", Style::default().fg(t.heading).add_modifier(Modifier::BOLD))));

    let slots: &[(&str, Option<TarotCard>, DraftStep)] = &[
        ("Hero", game.player.hero, DraftStep::PickHero),
        ("Wpn ", game.player.weapon, DraftStep::PickWeapon),
        ("App ", game.player.apparel, DraftStep::PickApparel),
        ("Itm ", game.player.item, DraftStep::PickItem),
    ];

    for (label, pick, slot_step) in slots {
        let is_current = *step == *slot_step;
        match pick {
            Some(c) => lines.push(Line::from(Span::styled(
                format!("{label}: {c}"),
                Style::default().fg(t.text),
            ))),
            None if is_current => lines.push(Line::from(Span::styled(
                format!("{label}: < picking >"),
                Style::default().fg(t.info),
            ))),
            None => lines.push(Line::from(Span::styled(
                format!("{label}: —"),
                Style::default().fg(t.muted),
            ))),
        }
    }
    lines
}

pub fn render_draft_reveal(frame: &mut Frame, game: &GameState) {
    let step = match &game.phase {
        GamePhase::DraftReveal { step } => step.clone(),
        _ => return,
    };
    let t = &game.theme;

    let step_label = match step {
        DraftStep::PickHero => "Hero",
        DraftStep::PickWeapon => "Weapon",
        DraftStep::PickApparel => "Apparel",
        DraftStep::PickItem => "Item",
    };

    let player_pick = match step {
        DraftStep::PickHero    => game.player.hero,
        DraftStep::PickWeapon  => game.player.weapon,
        DraftStep::PickApparel => game.player.apparel,
        DraftStep::PickItem    => game.player.item,
    };
    let ai_pick = match step {
        DraftStep::PickHero    => game.ai_state.hero,
        DraftStep::PickWeapon  => game.ai_state.weapon,
        DraftStep::PickApparel => game.ai_state.apparel,
        DraftStep::PickItem    => game.ai_state.item,
    };

    let area = frame.area();
    let lines = vec![
        Line::from(Span::styled(
            format!("— {} Pick Reveal —", step_label),
            Style::default().fg(t.heading).add_modifier(Modifier::BOLD),
        )).centered(),
        Line::from("").centered(),
        Line::from(Span::styled(
            format!("You picked: {}", player_pick.map(|c| c.to_string()).unwrap_or_else(|| "—".into())),
            Style::default().fg(t.text),
        )).centered(),
        Line::from(Span::styled(
            format!("Opponent picked: {}", ai_pick.map(|c| c.to_string()).unwrap_or_else(|| "—".into())),
            Style::default().fg(t.text),
        )).centered(),
        Line::from("").centered(),
        Line::from(Span::styled("[Space] Continue", Style::default().fg(t.muted))).centered(),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered()).centered(),
        area,
    );
}
