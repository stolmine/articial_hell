use std::collections::HashMap;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use crate::ai::{self, AiPersonality};
use crate::card::{TarotCard, MinorSuit, CourtRank};
use crate::combat::{CombatState, Fighter};
use crate::deck::TarotDeck;
use crate::game::{DraftStep, PlayerState};
use crate::stats::derive_stats;

#[derive(Clone, Debug)]
pub struct SimConfig {
    pub num_fights: usize,
    pub seed: Option<u64>,
    pub verbose: bool,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self { num_fights: 100, seed: None, verbose: false }
    }
}

#[derive(Clone, Debug)]
pub struct FightResult {
    pub p1_won: bool,
    pub turns: u32,
    pub cycles: u32,
    pub p1_hero: TarotCard,
    pub p2_hero: TarotCard,
    pub p1_hp_remaining: i32,
    pub p2_hp_remaining: i32,
    pub p1_max_hp: i32,
    pub p2_max_hp: i32,
    pub p1_stats: crate::stats::Stats,
    pub p2_stats: crate::stats::Stats,
    pub p1_weapon_suit: MinorSuit,
    pub p2_weapon_suit: MinorSuit,
    pub p1_apparel_suit: MinorSuit,
    pub p2_apparel_suit: MinorSuit,
    pub p1_item_suit: MinorSuit,
    pub p2_item_suit: MinorSuit,
}

fn ai_draft(deck: &mut TarotDeck, rng: &mut ChaCha8Rng) -> (PlayerState, AiPersonality) {
    let mut state = PlayerState::new();
    let mut personality: Option<AiPersonality> = None;

    let steps = [DraftStep::PickHero, DraftStep::PickWeapon, DraftStep::PickApparel, DraftStep::PickItem];
    for step in &steps {
        let choices = match step {
            DraftStep::PickHero => deck.draw_court(4),
            _ => deck.draw_numbered(4),
        };
        let idx = ai::draft_pick(&choices, step, &state, personality.as_ref(), rng);
        let card = choices[idx];
        state.set_slot(step, card);
        if *step == DraftStep::PickHero {
            personality = Some(AiPersonality::from_hero(card));
        }
    }

    (state, personality.unwrap())
}

fn run_combat(
    p1: &PlayerState,
    p1_pers: &AiPersonality,
    p2: &PlayerState,
    p2_pers: &AiPersonality,
    rng: &mut ChaCha8Rng,
    verbose: bool,
) -> FightResult {
    let f1 = Fighter::new(
        p1.hero.unwrap(), p1.weapon.unwrap(),
        p1.apparel.unwrap(), p1.item.unwrap(),
    );
    let f2 = Fighter::new(
        p2.hero.unwrap(), p2.weapon.unwrap(),
        p2.apparel.unwrap(), p2.item.unwrap(),
    );
    let p1_hero = p1.hero.unwrap();
    let p2_hero = p2.hero.unwrap();
    let p1_stats = derive_stats(p1_hero, p1.weapon.unwrap(), p1.apparel.unwrap(), p1.item.unwrap());
    let p2_stats = derive_stats(p2_hero, p2.weapon.unwrap(), p2.apparel.unwrap(), p2.item.unwrap());

    let combat_rng = ChaCha8Rng::from_rng(rng);
    let mut combat = CombatState::new(f1, f2, combat_rng);

    let max_turns = 200;
    let mut turn_count = 0;

    while !combat.combat_over && turn_count < max_turns {
        if combat.awaiting_queen_reassign {
            if combat.player.is_queen() {
                if let Some(cards) = combat.queen_original_cards[0] {
                    let best = ai::queen_reassign(&combat.player, cards);
                    combat.queen_reassign_complete(best.0, best.1, best.2);
                }
            }
            if combat.awaiting_queen_reassign { break; }
        }

        let p1_action = ai::combat_pick(&combat, Some(p1_pers), rng);
        let p2_action = ai::combat_pick(&combat, Some(p2_pers), rng);
        combat.resolve_turn(p1_action, p2_action);
        turn_count += 1;
    }

    if verbose {
        for entry in &combat.log {
            println!("  {entry}");
        }
        let winner = if combat.player_won { "P1" } else { "P2" };
        println!("  => {winner} wins (turn {}, cycle {})\n", combat.turn, combat.cycle);
    }

    FightResult {
        p1_won: combat.player_won,
        turns: combat.turn,
        cycles: combat.cycle,
        p1_hero, p2_hero,
        p1_hp_remaining: combat.player.current_hp,
        p2_hp_remaining: combat.ai.current_hp,
        p1_max_hp: combat.player.max_hp,
        p2_max_hp: combat.ai.max_hp,
        p1_stats, p2_stats,
        p1_weapon_suit: p1.weapon.unwrap().suit().unwrap_or(MinorSuit::Swords),
        p2_weapon_suit: p2.weapon.unwrap().suit().unwrap_or(MinorSuit::Swords),
        p1_apparel_suit: p1.apparel.unwrap().suit().unwrap_or(MinorSuit::Swords),
        p2_apparel_suit: p2.apparel.unwrap().suit().unwrap_or(MinorSuit::Swords),
        p1_item_suit: p1.item.unwrap().suit().unwrap_or(MinorSuit::Swords),
        p2_item_suit: p2.item.unwrap().suit().unwrap_or(MinorSuit::Swords),
    }
}

pub fn run_sim(config: &SimConfig) -> Vec<FightResult> {
    let mut rng = match config.seed {
        Some(s) => ChaCha8Rng::seed_from_u64(s),
        None => ChaCha8Rng::from_rng(&mut rand::rng()),
    };

    let mut results = Vec::with_capacity(config.num_fights);

    for i in 0..config.num_fights {
        let mut p1_deck = TarotDeck::new();
        let mut p2_deck = TarotDeck::new();
        p1_deck.shuffle_all(&mut rng);
        p2_deck.shuffle_all(&mut rng);

        let (p1, p1_pers) = ai_draft(&mut p1_deck, &mut rng);
        let (p2, p2_pers) = ai_draft(&mut p2_deck, &mut rng);

        if config.verbose {
            println!("Fight {}: {} vs {}", i + 1, p1.hero.unwrap(), p2.hero.unwrap());
        }

        let fight = run_combat(&p1, &p1_pers, &p2, &p2_pers, &mut rng, config.verbose);
        results.push(fight);
    }

    results
}

// --- Analysis helpers ---

struct WinRecord {
    wins: usize,
    total: usize,
}

impl WinRecord {
    fn new() -> Self { Self { wins: 0, total: 0 } }
    fn add(&mut self, won: bool) { self.total += 1; if won { self.wins += 1; } }
    fn rate(&self) -> f64 { if self.total == 0 { 0.0 } else { self.wins as f64 / self.total as f64 * 100.0 } }
}

fn suit_name(s: MinorSuit) -> &'static str {
    match s {
        MinorSuit::Swords => "Swords",
        MinorSuit::Wands => "Wands",
        MinorSuit::Cups => "Cups",
        MinorSuit::Pentacles => "Pents",
    }
}

fn rank_name(r: CourtRank) -> &'static str {
    match r {
        CourtRank::Page => "Page",
        CourtRank::Knight => "Knight",
        CourtRank::Queen => "Queen",
        CourtRank::King => "King",
    }
}

pub fn balance_report(config: &SimConfig) {
    let results = run_sim(config);
    let n = results.len();

    let p1_wins = results.iter().filter(|f| f.p1_won).count();
    let avg_turns: f64 = results.iter().map(|f| f.turns as f64).sum::<f64>() / n as f64;
    let avg_cycles: f64 = results.iter().map(|f| f.cycles as f64).sum::<f64>() / n as f64;

    println!("=== Balance Report: {n} fights (seed: {}) ===",
        config.seed.map(|s| s.to_string()).unwrap_or("random".into()));
    println!("{n} fights | P1 {p1_wins}/{n} ({:.1}%) | avg {avg_turns:.1} turns, {avg_cycles:.1} cycles",
        p1_wins as f64 / n as f64 * 100.0);

    // --- 1. Suit winrate ---
    println!("\n--- SUIT WINRATE ---");
    let mut suit_rec: HashMap<MinorSuit, WinRecord> = HashMap::new();
    for f in &results {
        let s1 = f.p1_hero.suit().unwrap_or(MinorSuit::Swords);
        let s2 = f.p2_hero.suit().unwrap_or(MinorSuit::Swords);
        suit_rec.entry(s1).or_insert_with(WinRecord::new).add(f.p1_won);
        suit_rec.entry(s2).or_insert_with(WinRecord::new).add(!f.p1_won);
    }
    let mut suits: Vec<_> = suit_rec.into_iter().collect();
    suits.sort_by(|a, b| b.1.rate().partial_cmp(&a.1.rate()).unwrap());
    for (suit, rec) in &suits {
        println!("  {:<10} {:>4}/{:<4} ({:>5.1}%)", suit_name(*suit), rec.wins, rec.total, rec.rate());
    }

    // --- 2. Rank winrate ---
    println!("\n--- RANK WINRATE ---");
    let mut rank_rec: HashMap<CourtRank, WinRecord> = HashMap::new();
    for f in &results {
        let r1 = f.p1_hero.court_rank().unwrap_or(CourtRank::Page);
        let r2 = f.p2_hero.court_rank().unwrap_or(CourtRank::Page);
        rank_rec.entry(r1).or_insert_with(WinRecord::new).add(f.p1_won);
        rank_rec.entry(r2).or_insert_with(WinRecord::new).add(!f.p1_won);
    }
    let mut ranks: Vec<_> = rank_rec.into_iter().collect();
    ranks.sort_by(|a, b| b.1.rate().partial_cmp(&a.1.rate()).unwrap());
    for (rank, rec) in &ranks {
        println!("  {:<10} {:>4}/{:<4} ({:>5.1}%)", rank_name(*rank), rec.wins, rec.total, rec.rate());
    }

    // --- 3. Suit vs Suit matchup matrix ---
    println!("\n--- SUIT vs SUIT MATCHUP (row = hero, col = opponent) ---");
    let mut matchup: HashMap<(MinorSuit, MinorSuit), WinRecord> = HashMap::new();
    for f in &results {
        let s1 = f.p1_hero.suit().unwrap_or(MinorSuit::Swords);
        let s2 = f.p2_hero.suit().unwrap_or(MinorSuit::Swords);
        matchup.entry((s1, s2)).or_insert_with(WinRecord::new).add(f.p1_won);
        matchup.entry((s2, s1)).or_insert_with(WinRecord::new).add(!f.p1_won);
    }
    print!("  {:>10}", "");
    for opp in &MinorSuit::ALL { print!("  {:>8}", suit_name(*opp)); }
    println!();
    for hero in &MinorSuit::ALL {
        print!("  {:<10}", suit_name(*hero));
        for opp in &MinorSuit::ALL {
            if let Some(rec) = matchup.get(&(*hero, *opp)) {
                print!("  {:>5.1}%/{}", rec.rate(), rec.total);
            } else {
                print!("  {:>8}", "-");
            }
        }
        println!();
    }

    // --- 4. Stat averages: winners vs losers ---
    println!("\n--- STAT PROFILE: WINNERS vs LOSERS ---");
    let (mut w_atk, mut w_def, mut w_hp, mut w_spd, mut w_n) = (0i64, 0i64, 0i64, 0i64, 0usize);
    let (mut l_atk, mut l_def, mut l_hp, mut l_spd, mut l_n) = (0i64, 0i64, 0i64, 0i64, 0usize);
    for f in &results {
        let (ws, ls) = if f.p1_won { (&f.p1_stats, &f.p2_stats) } else { (&f.p2_stats, &f.p1_stats) };
        w_atk += ws.attack as i64; w_def += ws.defense as i64; w_hp += ws.hp as i64; w_spd += ws.speed as i64; w_n += 1;
        l_atk += ls.attack as i64; l_def += ls.defense as i64; l_hp += ls.hp as i64; l_spd += ls.speed as i64; l_n += 1;
    }
    println!("  Winners  avg ATK:{:.1} DEF:{:.1} HP:{:.1} SPD:{:.1}",
        w_atk as f64 / w_n as f64, w_def as f64 / w_n as f64, w_hp as f64 / w_n as f64, w_spd as f64 / w_n as f64);
    println!("  Losers   avg ATK:{:.1} DEF:{:.1} HP:{:.1} SPD:{:.1}",
        l_atk as f64 / l_n as f64, l_def as f64 / l_n as f64, l_hp as f64 / l_n as f64, l_spd as f64 / l_n as f64);
    println!("  Delta        ATK:{:+.1} DEF:{:+.1} HP:{:+.1} SPD:{:+.1}",
        w_atk as f64 / w_n as f64 - l_atk as f64 / l_n as f64,
        w_def as f64 / w_n as f64 - l_def as f64 / l_n as f64,
        w_hp as f64 / w_n as f64 - l_hp as f64 / l_n as f64,
        w_spd as f64 / w_n as f64 - l_spd as f64 / l_n as f64);

    // --- 5. Equipment suit winrate by slot ---
    println!("\n--- EQUIPMENT SUIT WINRATE ---");
    for slot_name_str in ["Weapon", "Apparel", "Item"] {
        let mut eq_rec: HashMap<MinorSuit, WinRecord> = HashMap::new();
        for f in &results {
            let (s1, s2) = match slot_name_str {
                "Weapon" => (f.p1_weapon_suit, f.p2_weapon_suit),
                "Apparel" => (f.p1_apparel_suit, f.p2_apparel_suit),
                _ => (f.p1_item_suit, f.p2_item_suit),
            };
            eq_rec.entry(s1).or_insert_with(WinRecord::new).add(f.p1_won);
            eq_rec.entry(s2).or_insert_with(WinRecord::new).add(!f.p1_won);
        }
        let mut eqs: Vec<_> = eq_rec.into_iter().collect();
        eqs.sort_by(|a, b| b.1.rate().partial_cmp(&a.1.rate()).unwrap());
        print!("  {slot_name_str:<8}");
        for (suit, rec) in &eqs {
            print!("  {}:{:.0}%/{}", suit_name(*suit), rec.rate(), rec.total);
        }
        println!();
    }

    // --- 6. Overkill analysis (how decisive are wins) ---
    println!("\n--- WIN DECISIVENESS ---");
    let mut close_wins = 0usize; // winner had <25% HP
    let mut medium_wins = 0usize; // 25-50%
    let mut dominant_wins = 0usize; // >50%
    for f in &results {
        let (winner_hp, winner_max) = if f.p1_won {
            (f.p1_hp_remaining, f.p1_max_hp)
        } else {
            (f.p2_hp_remaining, f.p2_max_hp)
        };
        let pct = winner_hp * 100 / winner_max.max(1);
        if pct < 25 { close_wins += 1; }
        else if pct < 50 { medium_wins += 1; }
        else { dominant_wins += 1; }
    }
    println!("  Close (<25% HP left):    {close_wins} ({:.0}%)", close_wins as f64 / n as f64 * 100.0);
    println!("  Medium (25-50% HP left): {medium_wins} ({:.0}%)", medium_wins as f64 / n as f64 * 100.0);
    println!("  Dominant (>50% HP left):  {dominant_wins} ({:.0}%)", dominant_wins as f64 / n as f64 * 100.0);

    // --- 7. Fight length by matchup type ---
    println!("\n--- FIGHT LENGTH BY MATCHUP ---");
    let offensive = [MinorSuit::Swords, MinorSuit::Wands];
    let defensive = [MinorSuit::Cups, MinorSuit::Pentacles];
    let mut off_off = Vec::new();
    let mut off_def = Vec::new();
    let mut def_def = Vec::new();
    for f in &results {
        let s1 = f.p1_hero.suit().unwrap_or(MinorSuit::Swords);
        let s2 = f.p2_hero.suit().unwrap_or(MinorSuit::Swords);
        let o1 = offensive.contains(&s1);
        let o2 = offensive.contains(&s2);
        let bucket = if o1 && o2 { &mut off_off }
            else if !o1 && !o2 { &mut def_def }
            else { &mut off_def };
        bucket.push(f.turns as f64);
    }
    let avg = |v: &[f64]| if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 };
    println!("  Offensive vs Offensive: {:.1} avg turns ({} fights)", avg(&off_off), off_off.len());
    println!("  Offensive vs Defensive: {:.1} avg turns ({} fights)", avg(&off_def), off_def.len());
    println!("  Defensive vs Defensive: {:.1} avg turns ({} fights)", avg(&def_def), def_def.len());

    // --- 8. Offensive vs Defensive suit winrate in mixed matchups ---
    println!("\n--- OFFENSIVE vs DEFENSIVE HEAD-TO-HEAD ---");
    let mut off_wins = 0usize;
    let mut mixed_total = 0usize;
    for f in &results {
        let s1 = f.p1_hero.suit().unwrap_or(MinorSuit::Swords);
        let s2 = f.p2_hero.suit().unwrap_or(MinorSuit::Swords);
        let o1 = offensive.contains(&s1);
        let o2 = offensive.contains(&s2);
        if o1 == o2 { continue; } // same archetype, skip
        mixed_total += 1;
        if (o1 && f.p1_won) || (o2 && !f.p1_won) { off_wins += 1; }
    }
    if mixed_total > 0 {
        println!("  Offensive wins: {off_wins}/{mixed_total} ({:.1}%)", off_wins as f64 / mixed_total as f64 * 100.0);
        println!("  Defensive wins: {}/{mixed_total} ({:.1}%)", mixed_total - off_wins,
            (mixed_total - off_wins) as f64 / mixed_total as f64 * 100.0);
    }

    // --- 9. Per-hero detailed breakdown ---
    println!("\n--- FULL HERO BREAKDOWN ---");
    let mut hero_data: HashMap<String, (WinRecord, Vec<f64>, Vec<i32>)> = HashMap::new();
    for f in &results {
        let k1 = format!("{}", f.p1_hero);
        let e1 = hero_data.entry(k1).or_insert_with(|| (WinRecord::new(), Vec::new(), Vec::new()));
        e1.0.add(f.p1_won);
        e1.1.push(f.turns as f64);
        e1.2.push(f.p1_stats.attack + f.p1_stats.defense + f.p1_stats.hp + f.p1_stats.speed);

        let k2 = format!("{}", f.p2_hero);
        let e2 = hero_data.entry(k2).or_insert_with(|| (WinRecord::new(), Vec::new(), Vec::new()));
        e2.0.add(!f.p1_won);
        e2.1.push(f.turns as f64);
        e2.2.push(f.p2_stats.attack + f.p2_stats.defense + f.p2_stats.hp + f.p2_stats.speed);
    }
    let mut hero_list: Vec<_> = hero_data.into_iter().collect();
    hero_list.sort_by(|a, b| b.1.0.rate().partial_cmp(&a.1.0.rate()).unwrap());
    println!("  {:<22} {:>7}  {:>7}  {:>8}  {:>8}", "Hero", "WR%", "W/Total", "AvgTurn", "AvgStat");
    for (hero, (rec, turns, stats)) in &hero_list {
        let avg_t = turns.iter().sum::<f64>() / turns.len().max(1) as f64;
        let avg_s = stats.iter().sum::<i32>() as f64 / stats.len().max(1) as f64;
        println!("  {:<22} {:>5.1}%  {:>3}/{:<3}  {:>7.1}  {:>7.1}",
            hero, rec.rate(), rec.wins, rec.total, avg_t, avg_s);
    }
}
