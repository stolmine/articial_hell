use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use crate::card::{TarotCard, MinorSuit, CourtRank};
use crate::combat::{CombatAction, CombatState, Side};
use crate::game::{DraftStep, PlayerState};
use crate::stats;

#[derive(Clone, Debug)]
pub struct AiPersonality {
    pub hero_suit: MinorSuit,
    pub hero_rank: CourtRank,
    pub diversity_weight: f32,
    pub aggression: f32,
    pub default_sequence: [CombatAction; 3],
}

impl AiPersonality {
    pub fn from_hero(card: TarotCard) -> Self {
        let hero_suit = card.suit().unwrap_or(MinorSuit::Swords);
        let hero_rank = card.court_rank().unwrap_or(CourtRank::Page);

        let diversity_weight = match hero_rank {
            CourtRank::Page => 1.0,
            CourtRank::Knight => 0.3,
            CourtRank::Queen => -0.3,
            CourtRank::King => -1.0,
        };

        let aggression = match hero_suit {
            MinorSuit::Swords => 1.0,
            MinorSuit::Wands => 0.5,
            MinorSuit::Cups => -0.5,
            MinorSuit::Pentacles => -1.0,
        };

        let default_sequence = match hero_suit {
            MinorSuit::Swords | MinorSuit::Wands => [
                CombatAction::Weapon,
                CombatAction::Item,
                CombatAction::Apparel,
            ],
            MinorSuit::Cups | MinorSuit::Pentacles => [
                CombatAction::Apparel,
                CombatAction::Weapon,
                CombatAction::Item,
            ],
        };

        Self { hero_suit, hero_rank, diversity_weight, aggression, default_sequence }
    }
}

pub fn draft_pick(
    choices: &[TarotCard],
    step: &DraftStep,
    ai_state: &PlayerState,
    personality: Option<&AiPersonality>,
    rng: &mut ChaCha8Rng,
) -> usize {
    if choices.is_empty() {
        return 0;
    }

    match step {
        DraftStep::PickHero => {
            // Random hero pick — personality doesn't exist yet
            rng.random_range(0..choices.len())
        }
        DraftStep::PickArcana => {
            pick_arcana(choices, personality, rng)
        }
        _ => {
            pick_equipment(choices, step, ai_state, personality, rng)
        }
    }
}

fn slot_with_pick(
    step: &DraftStep,
    card: TarotCard,
    current_weapon: Option<TarotCard>,
    current_apparel: Option<TarotCard>,
    current_item: Option<TarotCard>,
) -> (Option<TarotCard>, Option<TarotCard>, Option<TarotCard>) {
    match step {
        DraftStep::PickWeapon => (Some(card), current_apparel, current_item),
        DraftStep::PickApparel => (current_weapon, Some(card), current_item),
        DraftStep::PickItem => (current_weapon, current_apparel, Some(card)),
        _ => (current_weapon, current_apparel, current_item),
    }
}

fn stat_delta_for_pick(
    hero: Option<TarotCard>,
    step: &DraftStep,
    card: TarotCard,
    current_weapon: Option<TarotCard>,
    current_apparel: Option<TarotCard>,
    current_item: Option<TarotCard>,
) -> i32 {
    let (tw, ta, ti) = slot_with_pick(step, card, current_weapon, current_apparel, current_item);
    let test = stats::partial_derive(hero, tw, ta, ti);
    let base = stats::partial_derive(hero, current_weapon, current_apparel, current_item);
    (test.attack - base.attack) + (test.defense - base.defense)
        + (test.hp - base.hp) + (test.speed - base.speed)
}

fn pick_equipment(
    choices: &[TarotCard],
    step: &DraftStep,
    ai_state: &PlayerState,
    personality: Option<&AiPersonality>,
    rng: &mut ChaCha8Rng,
) -> usize {
    let pers = match personality {
        Some(p) => p,
        None => {
            return choices.iter()
                .enumerate()
                .max_by_key(|(_, c)| c.numbered_value().unwrap_or(0))
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    };

    let hero = ai_state.hero;
    let (current_weapon, current_apparel, current_item) = match step {
        DraftStep::PickWeapon => (None, None, None),
        DraftStep::PickApparel => (ai_state.weapon, None, None),
        DraftStep::PickItem => (ai_state.weapon, ai_state.apparel, None),
        _ => (ai_state.weapon, ai_state.apparel, ai_state.item),
    };

    let existing_suits: Vec<MinorSuit> = [current_weapon, current_apparel, current_item]
        .iter()
        .filter_map(|o| o.and_then(|c| c.suit()))
        .collect();

    let scores: Vec<f32> = choices.iter().map(|card| {
        let base_value = card.numbered_value().unwrap_or(1) as f32;
        let card_suit = card.suit();

        let synergy = if pers.diversity_weight > 0.0 {
            let is_new = card_suit.map(|s| !existing_suits.contains(&s)).unwrap_or(false);
            if is_new { 3.0 * pers.diversity_weight } else { -1.0 * pers.diversity_weight }
        } else {
            let matches_hero = card_suit == Some(pers.hero_suit);
            if matches_hero { 3.0 * pers.diversity_weight.abs() } else { -1.0 * pers.diversity_weight.abs() }
        };

        let slot_weight = match step {
            DraftStep::PickWeapon => pers.aggression * 1.5,
            DraftStep::PickApparel => -pers.aggression * 1.5,
            _ => 0.0,
        };

        let suit_align = match card_suit {
            Some(MinorSuit::Swords) => pers.aggression * 1.0,
            Some(MinorSuit::Wands) => pers.aggression * 0.5,
            Some(MinorSuit::Cups) => -pers.aggression * 0.5,
            Some(MinorSuit::Pentacles) => -pers.aggression * 1.0,
            None => 0.0,
        };

        let delta = stat_delta_for_pick(hero, step, *card, current_weapon, current_apparel, current_item);
        base_value + synergy + slot_weight + suit_align + delta as f32 * 0.3
    }).collect();

    let personality_pick = scores.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let raw_best = choices.iter()
        .enumerate()
        .max_by_key(|(_, card)| stat_delta_for_pick(hero, step, **card, current_weapon, current_apparel, current_item))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let delta_best = stat_delta_for_pick(hero, step, choices[raw_best], current_weapon, current_apparel, current_item);
    let delta_pers = stat_delta_for_pick(hero, step, choices[personality_pick], current_weapon, current_apparel, current_item);

    if delta_best - delta_pers >= 5 { raw_best } else { personality_pick }
}

fn pick_arcana(
    choices: &[TarotCard],
    personality: Option<&AiPersonality>,
    _rng: &mut ChaCha8Rng,
) -> usize {
    let pers = match personality {
        Some(p) => p,
        None => return 0,
    };

    let scores: Vec<f32> = choices.iter().map(|card| {
        let arcana = match card.arcana() {
            Some(a) => a,
            None => return 0.0,
        };

        // Use a dummy hero suit and empty equipment for evaluation
        let effect = crate::arcana::resolve_arcana(arcana, Some(pers.hero_suit), &[
            TarotCard::Numbered { suit: pers.hero_suit, value: 5 },
            TarotCard::Numbered { suit: pers.hero_suit, value: 5 },
            TarotCard::Numbered { suit: pers.hero_suit, value: 5 },
        ]);
        let s = &effect.stat_bonus;

        // Weighted stat value based on aggression
        let atk_weight = 1.0 + pers.aggression * 0.5;
        let def_weight = 1.0 - pers.aggression * 0.5;
        let hp_weight = 1.0 - pers.aggression * 0.3;
        let spd_weight = 1.0 + pers.aggression * 0.3;

        let stat_score = s.attack as f32 * atk_weight
            + s.defense as f32 * def_weight
            + s.hp as f32 * hp_weight
            + s.speed as f32 * spd_weight;

        // Special ability bonuses
        let special = if effect.always_first { 5.0 } else { 0.0 }
            + if effect.first_hit_double { 4.0 * (1.0 + pers.aggression * 0.3) } else { 0.0 }
            + if effect.heal_per_turn > 0 { effect.heal_per_turn as f32 * 2.0 * (1.0 - pers.aggression * 0.3) } else { 0.0 }
            + if effect.death_curse { 3.0 } else { 0.0 };

        stat_score + special
    }).collect();

    scores.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

pub fn combat_pick(
    combat: &CombatState,
    personality: Option<&AiPersonality>,
    rng: &mut ChaCha8Rng,
) -> CombatAction {
    let available = combat.available_actions(Side::Ai);
    if available.is_empty() {
        return CombatAction::Weapon;
    }
    if available.len() == 1 {
        return available[0];
    }

    let pers = match personality {
        Some(p) => p,
        None => {
            for pref in [CombatAction::Weapon, CombatAction::Item, CombatAction::Apparel] {
                if available.contains(&pref) {
                    return pref;
                }
            }
            return available[0];
        }
    };

    let fighter = combat.fighter(Side::Ai);
    let opponent = combat.fighter(Side::Player);
    let hp_pct = if fighter.max_hp > 0 { fighter.current_hp * 100 / fighter.max_hp } else { 100 };
    let opp_hp_pct = if opponent.max_hp > 0 { opponent.current_hp * 100 / opponent.max_hp } else { 100 };

    let opp_available = combat.available_actions(Side::Player);
    let opp_forced = if opp_available.len() == 1 { Some(opp_available[0]) } else { None };

    let scores: Vec<(CombatAction, f32)> = available.iter().map(|&action| {
        // Personality weight: position in default_sequence
        let seq_weight = match pers.default_sequence.iter().position(|&a| a == action) {
            Some(0) => 4.0,
            Some(1) => 2.0,
            Some(2) => 0.0,
            _ => 0.0,
        };

        // Situational weight based on HP
        let situational = match action {
            CombatAction::Apparel => {
                // More valuable when low HP
                if hp_pct < 40 { 3.0 } else if hp_pct < 70 { 1.0 } else { 0.0 }
            }
            CombatAction::Weapon => {
                // More valuable when opponent is low
                if opp_hp_pct < 30 { 3.0 } else if opp_hp_pct < 60 { 1.0 } else { 0.0 }
            }
            CombatAction::Item => {
                // Cups item (heal) more valuable when low HP
                let item_suit = fighter.item.suit();
                if item_suit == Some(MinorSuit::Cups) && hp_pct < 50 {
                    2.5
                } else if item_suit == Some(MinorSuit::Swords) && opp_hp_pct < 40 {
                    2.0
                } else {
                    0.5
                }
            }
        };

        // Opponent read: respond to forced action
        let opponent_read = match opp_forced {
            Some(CombatAction::Weapon) => {
                // Opponent must attack — use defense
                if action == CombatAction::Apparel { 4.0 } else { 0.0 }
            }
            Some(CombatAction::Apparel) => {
                // Opponent must defend — attack harder
                if action == CombatAction::Weapon { 3.0 }
                else if action == CombatAction::Item { 2.0 }
                else { 0.0 }
            }
            Some(CombatAction::Item) => {
                // Opponent uses item — weapon is good
                if action == CombatAction::Weapon { 2.0 } else { 0.0 }
            }
            None => 0.0,
        };

        // Random noise for unpredictability
        let noise: f32 = rng.random_range(-1.5..=1.5);

        let total = seq_weight + situational + opponent_read + noise;
        (action, total)
    }).collect();

    scores.iter()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(action, _)| *action)
        .unwrap_or(available[0])
}
