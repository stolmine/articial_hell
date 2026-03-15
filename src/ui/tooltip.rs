use ratatui::prelude::*;
use crate::card::{TarotCard, MinorSuit, CourtRank};
use crate::game::{DraftStep, PlayerState};
use crate::stats::{hero_base_stats, equipment_primary, equipment_secondary, EquipSlot};
use crate::theme::Theme;

pub fn card_tooltip<'a>(card: &TarotCard, step: &DraftStep, player: &PlayerState, t: &Theme) -> Vec<Line<'a>> {
    match card {
        TarotCard::Court { suit, rank } => court_tooltip(*suit, *rank, t),
        TarotCard::Numbered { suit, value } => numbered_tooltip(*suit, *value, step, player, t),
        TarotCard::Major(_) => vec![],
    }
}

fn court_tooltip(suit: MinorSuit, rank: CourtRank, t: &Theme) -> Vec<Line<'static>> {
    let stats = hero_base_stats(suit);
    let role = match suit {
        MinorSuit::Swords => "Offensive — high ATK base",
        MinorSuit::Wands => "Agile — high SPD base",
        MinorSuit::Cups => "Durable — high HP base",
        MinorSuit::Pentacles => "Tanky — high DEF base",
    };
    let rank_desc = match rank {
        CourtRank::Page => "Diversity: +3 all stats per unique equipment suit (max +9)",
        CourtRank::Knight => "Diversity: +2 all stats per unique equipment suit (max +6)",
        CourtRank::Queen => "Matching: +2 all stats per equipment matching hero suit (max +6)",
        CourtRank::King => "Matching: +3 all stats per equipment matching hero suit (max +9)",
    };
    let strategy = match rank {
        CourtRank::Page | CourtRank::Knight => "Best with mixed-suit equipment",
        CourtRank::Queen | CourtRank::King => "Best with same-suit equipment",
    };

    let mut lines = vec![
        Line::from(Span::styled(format!("{rank} of {suit}"), Style::default().fg(t.text).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(role, Style::default().fg(t.info))),
        Line::from(Span::styled(
            format!("ATK: {}  SPD: {}  HP: {}  DEF: {}", stats.attack, stats.speed, stats.hp, stats.defense),
            Style::default().fg(t.text),
        )),
        Line::from(""),
        Line::from(Span::styled("Rank Ability", Style::default().fg(t.heading).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(rank_desc, Style::default().fg(t.text))),
        Line::from(Span::styled(strategy, Style::default().fg(t.muted))),
    ];

    match rank {
        CourtRank::Knight => {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Combat: Wildcard", Style::default().fg(t.heading).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(Span::styled("One random action is doubled each cycle (x2 uses)", Style::default().fg(t.info))));
            lines.push(Line::from(Span::styled("Skips one other action to compensate", Style::default().fg(t.muted))));
        }
        CourtRank::Queen => {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Combat: Shapeshifter", Style::default().fg(t.heading).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(Span::styled("Reassign equipment slots each cycle", Style::default().fg(t.info))));
            lines.push(Line::from(Span::styled("Adapt loadout to counter the opponent", Style::default().fg(t.muted))));
        }
        _ => {}
    }

    lines
}

fn numbered_tooltip(suit: MinorSuit, value: u8, step: &DraftStep, player: &PlayerState, t: &Theme) -> Vec<Line<'static>> {
    let slot = match step {
        DraftStep::PickWeapon => EquipSlot::Weapon,
        DraftStep::PickApparel => EquipSlot::Apparel,
        DraftStep::PickItem => EquipSlot::Item,
        _ => EquipSlot::Weapon,
    };

    let primary = equipment_primary(slot, value);
    let secondary = equipment_secondary(suit, value);
    let slot_name = match slot {
        EquipSlot::Weapon => "Weapon",
        EquipSlot::Apparel => "Apparel",
        EquipSlot::Item => "Item",
    };
    let primary_desc = match slot {
        EquipSlot::Weapon => format!("+{} ATK", primary.attack),
        EquipSlot::Apparel => format!("+{} DEF", primary.defense),
        EquipSlot::Item => format!("+{} HP", primary.hp),
    };
    let sec_stat = suit.stat_name();
    let sec_val = match suit {
        MinorSuit::Swords => secondary.attack,
        MinorSuit::Wands => secondary.speed,
        MinorSuit::Cups => secondary.hp,
        MinorSuit::Pentacles => secondary.defense,
    };

    let action = match (slot, suit) {
        (EquipSlot::Weapon, MinorSuit::Swords) => "Strike: deal full ATK damage",
        (EquipSlot::Weapon, MinorSuit::Cups) => "Drain: deal 70% ATK, heal for damage dealt",
        (EquipSlot::Weapon, MinorSuit::Wands) => "Quick Strike: deal 70% ATK, always act first",
        (EquipSlot::Weapon, MinorSuit::Pentacles) => "Heavy Blow: deal 120% ATK, halve enemy DEF",
        (EquipSlot::Apparel, MinorSuit::Swords) => "Riposte: +50% DEF this turn, counter 30% ATK",
        (EquipSlot::Apparel, MinorSuit::Cups) => "Restore: heal DEF amount of HP",
        (EquipSlot::Apparel, MinorSuit::Wands) => "Evade: add SPD to DEF this turn",
        (EquipSlot::Apparel, MinorSuit::Pentacles) => "Fortify: triple DEF this turn",
        (EquipSlot::Item, MinorSuit::Swords) => "Backstab: ATK+value damage, pierces half DEF",
        (EquipSlot::Item, MinorSuit::Cups) => "Elixir: heal value x2 HP (1 use)",
        (EquipSlot::Item, MinorSuit::Wands) => "Haste: use weapon + apparel action (1 use)",
        (EquipSlot::Item, MinorSuit::Pentacles) => "Barrier: immune to damage this turn (1 use)",
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("{} of {} — {slot_name}", if value == 1 { "Ace".to_string() } else { value.to_string() }, suit),
            Style::default().fg(t.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Primary: {primary_desc}   Secondary: +{sec_val} {sec_stat}"),
            Style::default().fg(t.text),
        )),
        Line::from(Span::styled(format!("Action: {action}"), Style::default().fg(t.info))),
    ];

    if let Some(hero) = player.hero {
        if let (Some(hero_suit), Some(hero_rank)) = (hero.suit(), hero.court_rank()) {
            lines.push(Line::from(""));
            let matches_hero = suit == hero_suit;
            let synergy = match (hero_rank, matches_hero) {
                (CourtRank::King, true) => Span::styled("Matches hero suit — King bonus!", Style::default().fg(t.positive)),
                (CourtRank::Queen, true) => Span::styled("Matches hero suit — Queen bonus!", Style::default().fg(t.positive)),
                (CourtRank::King, false) => Span::styled("Different suit — no King bonus", Style::default().fg(t.negative)),
                (CourtRank::Queen, false) => Span::styled("Different suit — no Queen bonus", Style::default().fg(t.negative)),
                (CourtRank::Page, false) => Span::styled("Different suit — Page diversity bonus!", Style::default().fg(t.positive)),
                (CourtRank::Knight, false) => Span::styled("Different suit — Knight diversity bonus!", Style::default().fg(t.positive)),
                (CourtRank::Page, true) => Span::styled("Same suit — less Page diversity", Style::default().fg(t.warning)),
                (CourtRank::Knight, true) => Span::styled("Same suit — less Knight diversity", Style::default().fg(t.warning)),
            };
            lines.push(Line::from(synergy));

            let existing: Vec<MinorSuit> = [player.weapon, player.apparel, player.item]
                .iter()
                .filter_map(|o| o.and_then(|c| c.suit()))
                .collect();
            if !existing.is_empty() {
                let mut all_suits = existing.clone();
                all_suits.push(suit);
                all_suits.sort_by_key(|s| *s as u8);
                all_suits.dedup();
                let unique = all_suits.len();
                let total_matching: usize = existing.iter()
                    .chain(std::iter::once(&suit))
                    .filter(|s| **s == hero_suit)
                    .count();
                lines.push(Line::from(Span::styled(
                    format!("With this: {} unique suits, {} matching hero", unique, total_matching),
                    Style::default().fg(t.muted),
                )));
            }
        }
    }

    lines
}
