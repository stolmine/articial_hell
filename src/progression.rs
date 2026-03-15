use crate::card::{CourtRank, MinorSuit, TarotCard};
use crate::stats::Stats;

#[derive(Clone, Debug, Default)]
pub struct ProgressionState {
    pub suit_affinity: [u8; 4],     // [Swords, Wands, Cups, Pents] pick counts
    pub diversity_picks: u8,         // times Page/Knight was hero
    pub matching_picks: u8,          // times Queen/King was hero
    pub performance_score: i32,      // adaptive AI scaling accumulator
    pub fights_survived: u8,         // wins so far (flat +1 all stats per fight)
}

pub fn suit_index(suit: MinorSuit) -> usize {
    match suit {
        MinorSuit::Swords => 0,
        MinorSuit::Wands => 1,
        MinorSuit::Cups => 2,
        MinorSuit::Pentacles => 3,
    }
}

pub fn affinity_bonus(suit: MinorSuit, affinity: u8) -> Stats {
    let n = affinity as i32;
    match suit {
        MinorSuit::Swords => Stats { attack: n, ..Default::default() },
        MinorSuit::Wands => Stats { speed: n, ..Default::default() },
        MinorSuit::Cups => Stats { hp: n, ..Default::default() },
        MinorSuit::Pentacles => Stats { defense: n, ..Default::default() },
    }
}

pub fn rank_path_tier(state: &ProgressionState, rank: CourtRank) -> i32 {
    let same_path_picks = match rank {
        CourtRank::Page | CourtRank::Knight => state.diversity_picks,
        CourtRank::Queen | CourtRank::King => state.matching_picks,
    };
    (same_path_picks as i32).min(3)
}

pub fn progression_bonus(state: &ProgressionState, hero: TarotCard, equipment: &[TarotCard]) -> Stats {
    let mut delta = Stats::default();

    // Affinity bonuses per equipment card
    for card in equipment {
        if let Some(suit) = card.suit() {
            let affinity = state.suit_affinity[suit_index(suit)];
            if affinity > 0 {
                delta.add(&affinity_bonus(suit, affinity));
            }
        }
    }

    // Survivor bonus: +1 all stats per 2 fights survived
    let survivor_bonus = state.fights_survived as i32 / 2;
    if survivor_bonus > 0 {
        delta.add_flat(survivor_bonus);
    }

    // Rank path scaling delta
    if let (Some(rank), Some(hero_suit)) = (hero.court_rank(), hero.suit()) {
        let tier = rank_path_tier(state, rank);
        if tier > 0 {
            let unique_suits = crate::stats::count_unique_suits(equipment) as i32;
            let matching_suits = equipment.iter()
                .filter(|c| c.suit() == Some(hero_suit))
                .count() as i32;

            let rank_delta = match rank {
                CourtRank::Page | CourtRank::Knight => unique_suits * tier,
                CourtRank::Queen | CourtRank::King => matching_suits * tier,
            };
            delta.add_flat(rank_delta);
        }
    }

    delta
}

pub const BOSS_FIGHT_BONUS: i32 = 10;

pub fn ai_scaling_bonus(state: &ProgressionState) -> i32 {
    state.performance_score.clamp(0, 4)
}

pub fn ai_boss_bonus(fight_number: usize, total_fights: usize) -> i32 {
    if fight_number == total_fights { BOSS_FIGHT_BONUS } else { 0 }
}

pub fn record_equipment_pick(state: &mut ProgressionState, card: TarotCard) {
    if let Some(suit) = card.suit() {
        state.suit_affinity[suit_index(suit)] += 1;
    }
}

pub fn record_hero_pick(state: &mut ProgressionState, card: TarotCard) {
    match card.court_rank() {
        Some(CourtRank::Page) | Some(CourtRank::Knight) => state.diversity_picks += 1,
        Some(CourtRank::Queen) | Some(CourtRank::King) => state.matching_picks += 1,
        None => {}
    }
}

pub fn record_fight(state: &mut ProgressionState, won: bool, hp_margin_pct: i32) {
    if won {
        state.fights_survived += 1;
        // AI scaling only starts accumulating after fight 3
        if state.fights_survived >= 3 {
            if hp_margin_pct >= 50 {
                state.performance_score += 2;
            } else {
                state.performance_score += 1;
            }
        }
    } else {
        state.performance_score -= 1;
    }
}
