use std::collections::HashMap;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use crate::ai::{self, AiPersonality};
use crate::card::{TarotCard, MinorSuit, CourtRank};
use crate::combat::{BalanceTweaks, CombatState, Fighter, Side};
use crate::deck::TarotDeck;
use crate::game::{DraftStep, PlayerState};
use crate::stats::derive_stats;

#[derive(Clone, Debug)]
pub struct SimConfig {
    pub num_fights: usize,
    pub seed: Option<u64>,
    pub verbose: bool,
    pub tweaks: BalanceTweaks,
    pub compare: bool,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            num_fights: 100, seed: None, verbose: false,
            tweaks: BalanceTweaks::default(), compare: false,
        }
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

fn ai_draft(deck: &mut TarotDeck, rng: &mut ChaCha8Rng, verbose: bool, label: &str) -> (PlayerState, AiPersonality) {
    let mut state = PlayerState::new();
    let mut personality: Option<AiPersonality> = None;
    let mut trace: Vec<String> = Vec::new();

    let steps = [DraftStep::PickHero, DraftStep::PickWeapon, DraftStep::PickApparel, DraftStep::PickItem];
    for step in &steps {
        let choices = match step {
            DraftStep::PickHero => deck.draw_court(4),
            _ => deck.draw_numbered(4),
        };
        let t = if verbose { Some(&mut trace) } else { None };
        let idx = ai::draft_pick(&choices, step, &state, personality.as_ref(), rng, t);
        let card = choices[idx];
        state.set_slot(step, card);
        if *step == DraftStep::PickHero {
            personality = Some(AiPersonality::from_hero(card));
        }
    }

    if verbose && !trace.is_empty() {
        println!("{label} draft:");
        for line in &trace { println!("{line}"); }
    }

    (state, personality.unwrap())
}

fn run_combat(
    p1: &PlayerState,
    p1_pers: &AiPersonality,
    p2: &PlayerState,
    p2_pers: &AiPersonality,
    rng: &mut ChaCha8Rng,
    tweaks: &BalanceTweaks,
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
    let mut combat = CombatState::new_with_tweaks(f1, f2, combat_rng, *tweaks);

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

        let mut p1_trace: Vec<String> = Vec::new();
        let mut p2_trace: Vec<String> = Vec::new();
        let p1_action = ai::combat_pick_for(&combat, Side::Player, Some(p1_pers), rng,
            if verbose { Some(&mut p1_trace) } else { None });
        let p2_action = ai::combat_pick_for(&combat, Side::Ai, Some(p2_pers), rng,
            if verbose { Some(&mut p2_trace) } else { None });

        if verbose {
            println!("  --- Turn {} ---", turn_count + 1);
            for line in &p1_trace { println!("{line}"); }
            for line in &p2_trace { println!("{line}"); }
        }

        let log_before = combat.log.len();
        combat.resolve_turn(p1_action, p2_action);
        turn_count += 1;

        if verbose {
            for entry in &combat.log[log_before..] {
                println!("  {entry}");
            }
        }
    }

    if verbose {
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

/// Generate draft pairs (shared across all tweak variants for fair comparison)
struct DraftPair {
    p1: PlayerState,
    p1_pers: AiPersonality,
    p2: PlayerState,
    p2_pers: AiPersonality,
}

fn generate_drafts(num: usize, seed: Option<u64>) -> (Vec<DraftPair>, ChaCha8Rng) {
    let mut rng = match seed {
        Some(s) => ChaCha8Rng::seed_from_u64(s),
        None => ChaCha8Rng::from_rng(&mut rand::rng()),
    };

    let mut pairs = Vec::with_capacity(num);
    for _ in 0..num {
        let mut p1_deck = TarotDeck::new();
        let mut p2_deck = TarotDeck::new();
        p1_deck.shuffle_all(&mut rng);
        p2_deck.shuffle_all(&mut rng);

        let (p1, p1_pers) = ai_draft(&mut p1_deck, &mut rng, false, "P1");
        let (p2, p2_pers) = ai_draft(&mut p2_deck, &mut rng, false, "P2");
        pairs.push(DraftPair { p1, p1_pers, p2, p2_pers });
    }

    (pairs, rng)
}

fn run_with_tweaks(pairs: &[DraftPair], tweaks: &BalanceTweaks, base_rng: &ChaCha8Rng, verbose: bool) -> Vec<FightResult> {
    let mut rng = base_rng.clone();
    pairs.iter().map(|pair| {
        run_combat(&pair.p1, &pair.p1_pers, &pair.p2, &pair.p2_pers, &mut rng, tweaks, verbose)
    }).collect()
}

pub fn run_sim(config: &SimConfig) -> Vec<FightResult> {
    if config.verbose {
        // Verbose mode: draft and fight inline so we can trace both
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

            println!("=== Fight {} ===", i + 1);
            let (p1, p1_pers) = ai_draft(&mut p1_deck, &mut rng, true, "P1");
            let (p2, p2_pers) = ai_draft(&mut p2_deck, &mut rng, true, "P2");
            println!("P1: {} | Wpn:{} App:{} Itm:{}",
                p1.hero.unwrap(), p1.weapon.unwrap(), p1.apparel.unwrap(), p1.item.unwrap());
            println!("P2: {} | Wpn:{} App:{} Itm:{}",
                p2.hero.unwrap(), p2.weapon.unwrap(), p2.apparel.unwrap(), p2.item.unwrap());

            let fight = run_combat(&p1, &p1_pers, &p2, &p2_pers, &mut rng, &config.tweaks, true);
            results.push(fight);
        }
        results
    } else {
        let (pairs, base_rng) = generate_drafts(config.num_fights, config.seed);
        run_with_tweaks(&pairs, &config.tweaks, &base_rng, false)
    }
}

// --- Analysis ---

struct WinRecord { wins: usize, total: usize }

impl WinRecord {
    fn new() -> Self { Self { wins: 0, total: 0 } }
    fn add(&mut self, won: bool) { self.total += 1; if won { self.wins += 1; } }
    fn rate(&self) -> f64 { if self.total == 0 { 0.0 } else { self.wins as f64 / self.total as f64 * 100.0 } }
}

fn suit_name(s: MinorSuit) -> &'static str {
    match s {
        MinorSuit::Swords => "Swords", MinorSuit::Wands => "Wands",
        MinorSuit::Cups => "Cups", MinorSuit::Pentacles => "Pents",
    }
}

fn rank_name(r: CourtRank) -> &'static str {
    match r {
        CourtRank::Page => "Page", CourtRank::Knight => "Knight",
        CourtRank::Queen => "Queen", CourtRank::King => "King",
    }
}

struct QuickStats {
    suit_wr: [(MinorSuit, f64); 4],
    rank_wr: [(CourtRank, f64); 4],
    off_vs_def: f64,
    avg_turns: f64,
    dominant_pct: f64,
    stat_delta: [f64; 4], // ATK, DEF, HP, SPD winner-loser deltas
}

fn analyze(results: &[FightResult]) -> QuickStats {
    let n = results.len();

    let mut suit_rec: HashMap<MinorSuit, WinRecord> = HashMap::new();
    let mut rank_rec: HashMap<CourtRank, WinRecord> = HashMap::new();

    let offensive = [MinorSuit::Swords, MinorSuit::Wands];
    let mut off_wins = 0usize;
    let mut mixed_total = 0usize;
    let mut dominant = 0usize;

    let (mut w_atk, mut w_def, mut w_hp, mut w_spd) = (0i64, 0i64, 0i64, 0i64);
    let (mut l_atk, mut l_def, mut l_hp, mut l_spd) = (0i64, 0i64, 0i64, 0i64);

    for f in results {
        let s1 = f.p1_hero.suit().unwrap_or(MinorSuit::Swords);
        let s2 = f.p2_hero.suit().unwrap_or(MinorSuit::Swords);
        let r1 = f.p1_hero.court_rank().unwrap_or(CourtRank::Page);
        let r2 = f.p2_hero.court_rank().unwrap_or(CourtRank::Page);

        suit_rec.entry(s1).or_insert_with(WinRecord::new).add(f.p1_won);
        suit_rec.entry(s2).or_insert_with(WinRecord::new).add(!f.p1_won);
        rank_rec.entry(r1).or_insert_with(WinRecord::new).add(f.p1_won);
        rank_rec.entry(r2).or_insert_with(WinRecord::new).add(!f.p1_won);

        let o1 = offensive.contains(&s1);
        let o2 = offensive.contains(&s2);
        if o1 != o2 {
            mixed_total += 1;
            if (o1 && f.p1_won) || (o2 && !f.p1_won) { off_wins += 1; }
        }

        let (whp, wmax) = if f.p1_won { (f.p1_hp_remaining, f.p1_max_hp) } else { (f.p2_hp_remaining, f.p2_max_hp) };
        if whp * 100 / wmax.max(1) > 50 { dominant += 1; }

        let (ws, ls) = if f.p1_won { (&f.p1_stats, &f.p2_stats) } else { (&f.p2_stats, &f.p1_stats) };
        w_atk += ws.attack as i64; w_def += ws.defense as i64; w_hp += ws.hp as i64; w_spd += ws.speed as i64;
        l_atk += ls.attack as i64; l_def += ls.defense as i64; l_hp += ls.hp as i64; l_spd += ls.speed as i64;
    }

    // Ensure all suits/ranks present (small sample sizes may miss some)
    for s in &MinorSuit::ALL { suit_rec.entry(*s).or_insert_with(WinRecord::new); }
    for r in &CourtRank::ALL { rank_rec.entry(*r).or_insert_with(WinRecord::new); }

    let mut suit_wr: Vec<_> = suit_rec.into_iter().map(|(s, r)| (s, r.rate())).collect();
    suit_wr.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut rank_wr: Vec<_> = rank_rec.into_iter().map(|(r, rec)| (r, rec.rate())).collect();
    rank_wr.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let wn = n as f64;
    QuickStats {
        suit_wr: [suit_wr[0], suit_wr[1], suit_wr[2], suit_wr[3]],
        rank_wr: [rank_wr[0], rank_wr[1], rank_wr[2], rank_wr[3]],
        off_vs_def: if mixed_total > 0 { off_wins as f64 / mixed_total as f64 * 100.0 } else { 50.0 },
        avg_turns: results.iter().map(|f| f.turns as f64).sum::<f64>() / wn,
        dominant_pct: dominant as f64 / wn * 100.0,
        stat_delta: [
            (w_atk - l_atk) as f64 / wn,
            (w_def - l_def) as f64 / wn,
            (w_hp - l_hp) as f64 / wn,
            (w_spd - l_spd) as f64 / wn,
        ],
    }
}

fn print_quick(label: &str, s: &QuickStats) {
    println!("  {label}");
    print!("    Suit: ");
    for (suit, wr) in &s.suit_wr { print!(" {}:{:.0}%", suit_name(*suit), wr); }
    println!();
    print!("    Rank: ");
    for (rank, wr) in &s.rank_wr { print!(" {}:{:.0}%", rank_name(*rank), wr); }
    println!();
    println!("    Off vs Def: {:.1}% | Avg turns: {:.1} | Dominant wins: {:.0}%",
        s.off_vs_def, s.avg_turns, s.dominant_pct);
    println!("    Stat delta: ATK:{:+.1} DEF:{:+.1} HP:{:+.1} SPD:{:+.1}",
        s.stat_delta[0], s.stat_delta[1], s.stat_delta[2], s.stat_delta[3]);
    println!();
}

pub fn balance_report(config: &SimConfig) {
    if config.compare {
        compare_tweaks(config);
        return;
    }

    let results = run_sim(config);
    print_full_report(&results, config);
}

fn compare_tweaks(config: &SimConfig) {
    let (pairs, base_rng) = generate_drafts(config.num_fights, config.seed);
    let n = pairs.len();

    println!("=== A/B Comparison: {n} fights, same drafts ===\n");

    let d = BalanceTweaks::default(); // thorns 12%, bulk +1/6hp>20
    let variants: Vec<(&str, BalanceTweaks)> = vec![
        ("Current (no tempo)", d),
        ("Tempo 8",  BalanceTweaks { tempo_threshold: 8, ..d }),
        ("Tempo 10", BalanceTweaks { tempo_threshold: 10, ..d }),
        ("Tempo 12", BalanceTweaks { tempo_threshold: 12, ..d }),
    ];

    for (label, tweaks) in &variants {
        let results = run_with_tweaks(&pairs, tweaks, &base_rng, false);
        let stats = analyze(&results);
        print_quick(label, &stats);

        // Show Wands heroes specifically (the problem children)
        let mut hero_rec: HashMap<String, WinRecord> = HashMap::new();
        for f in &results {
            hero_rec.entry(format!("{}", f.p1_hero)).or_insert_with(WinRecord::new).add(f.p1_won);
            hero_rec.entry(format!("{}", f.p2_hero)).or_insert_with(WinRecord::new).add(!f.p1_won);
        }
        let mut hero_list: Vec<_> = hero_rec.into_iter().collect();
        hero_list.sort_by(|a, b| b.1.rate().partial_cmp(&a.1.rate()).unwrap());
        print!("    Bottom 4: ");
        for (hero, rec) in hero_list.iter().rev().take(4) {
            print!(" {}:{:.0}%", hero, rec.rate());
        }
        println!("\n");
    }
}

fn print_full_report(results: &[FightResult], config: &SimConfig) {
    let n = results.len();
    let p1_wins = results.iter().filter(|f| f.p1_won).count();
    let avg_turns: f64 = results.iter().map(|f| f.turns as f64).sum::<f64>() / n as f64;
    let avg_cycles: f64 = results.iter().map(|f| f.cycles as f64).sum::<f64>() / n as f64;

    println!("=== Balance Report: {n} fights (seed: {}) ===",
        config.seed.map(|s| s.to_string()).unwrap_or("random".into()));
    println!("{n} fights | P1 {p1_wins}/{n} ({:.1}%) | avg {avg_turns:.1} turns, {avg_cycles:.1} cycles",
        p1_wins as f64 / n as f64 * 100.0);

    let stats = analyze(results);

    println!("\n--- SUIT WINRATE ---");
    for (suit, wr) in &stats.suit_wr {
        println!("  {:<10} {:>5.1}%", suit_name(*suit), wr);
    }

    println!("\n--- RANK WINRATE ---");
    for (rank, wr) in &stats.rank_wr {
        println!("  {:<10} {:>5.1}%", rank_name(*rank), wr);
    }

    println!("\n--- SUIT vs SUIT MATCHUP ---");
    let mut matchup: HashMap<(MinorSuit, MinorSuit), WinRecord> = HashMap::new();
    for f in results {
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

    println!("\n--- STAT PROFILE: WINNERS vs LOSERS ---");
    let (mut w_atk, mut w_def, mut w_hp, mut w_spd) = (0i64, 0i64, 0i64, 0i64);
    let (mut l_atk, mut l_def, mut l_hp, mut l_spd) = (0i64, 0i64, 0i64, 0i64);
    for f in results {
        let (ws, ls) = if f.p1_won { (&f.p1_stats, &f.p2_stats) } else { (&f.p2_stats, &f.p1_stats) };
        w_atk += ws.attack as i64; w_def += ws.defense as i64; w_hp += ws.hp as i64; w_spd += ws.speed as i64;
        l_atk += ls.attack as i64; l_def += ls.defense as i64; l_hp += ls.hp as i64; l_spd += ls.speed as i64;
    }
    let wn = n as f64;
    println!("  Winners  avg ATK:{:.1} DEF:{:.1} HP:{:.1} SPD:{:.1}",
        w_atk as f64/wn, w_def as f64/wn, w_hp as f64/wn, w_spd as f64/wn);
    println!("  Losers   avg ATK:{:.1} DEF:{:.1} HP:{:.1} SPD:{:.1}",
        l_atk as f64/wn, l_def as f64/wn, l_hp as f64/wn, l_spd as f64/wn);
    println!("  Delta        ATK:{:+.1} DEF:{:+.1} HP:{:+.1} SPD:{:+.1}",
        (w_atk-l_atk) as f64/wn, (w_def-l_def) as f64/wn, (w_hp-l_hp) as f64/wn, (w_spd-l_spd) as f64/wn);

    println!("\n--- OFFENSIVE vs DEFENSIVE ---");
    println!("  Off wins: {:.1}% | Avg turns: {:.1} | Dominant wins: {:.0}%",
        stats.off_vs_def, stats.avg_turns, stats.dominant_pct);

    println!("\n--- FULL HERO BREAKDOWN ---");
    let mut hero_data: HashMap<String, WinRecord> = HashMap::new();
    for f in results {
        hero_data.entry(format!("{}", f.p1_hero)).or_insert_with(WinRecord::new).add(f.p1_won);
        hero_data.entry(format!("{}", f.p2_hero)).or_insert_with(WinRecord::new).add(!f.p1_won);
    }
    let mut hero_list: Vec<_> = hero_data.into_iter().collect();
    hero_list.sort_by(|a, b| b.1.rate().partial_cmp(&a.1.rate()).unwrap());
    println!("  {:<22} {:>7}  {:>7}", "Hero", "WR%", "W/Total");
    for (hero, rec) in &hero_list {
        println!("  {:<22} {:>5.1}%  {:>3}/{:<3}", hero, rec.rate(), rec.wins, rec.total);
    }
}
