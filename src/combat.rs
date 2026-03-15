use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use crate::card::{CourtRank, MajorArcana, MinorSuit, TarotCard};
use crate::stats::{Stats, derive_stats};
use crate::fate::{FateEffect, resolve_fate, fate_description};
use rand::seq::SliceRandom;

const DAMAGE_FLOOR: i32 = 2;
const HEAL_FLOOR: i32 = 1;

#[derive(Clone, Copy, Debug)]
pub struct BalanceTweaks {
    /// Passive thorns: when hit, reflect DEF * thorns_pct / 100 back to attacker
    pub thorns_pct: i32,
    /// Cycle escalation: +N damage per cycle to all attacks
    pub cycle_damage_bonus: i32,
    /// HP ratio bonus: fighters deal bonus damage = (current_hp_pct - 50) * hp_ratio_scale / 100
    /// Positive when above 50% HP, negative below (clamped to 0 min)
    pub hp_ratio_scale: i32,
    /// HP bulk bonus: +1 damage per hp_bulk_per HP above hp_bulk_threshold
    /// Rewards high max_hp builds (Cups niche)
    pub hp_bulk_threshold: i32,
    pub hp_bulk_per: i32,
}

impl Default for BalanceTweaks {
    fn default() -> Self {
        Self { thorns_pct: 12, cycle_damage_bonus: 0, hp_ratio_scale: 0,
               hp_bulk_threshold: 20, hp_bulk_per: 6 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side { Player, Ai }

impl Side {
    pub fn opponent(self) -> Side {
        match self { Side::Player => Side::Ai, Side::Ai => Side::Player }
    }
    pub fn name(self) -> &'static str {
        match self { Side::Player => "You", Side::Ai => "Enemy" }
    }
    pub fn target_name(self) -> &'static str {
        match self { Side::Player => "you", Side::Ai => "enemy" }
    }
    pub fn index(self) -> usize {
        match self { Side::Player => 0, Side::Ai => 1 }
    }
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub hero: TarotCard,
    pub weapon: TarotCard,
    pub apparel: TarotCard,
    pub item: TarotCard,
    pub stats: Stats,
    pub current_hp: i32,
    pub max_hp: i32,
    pub heal_count: i32,
    pub last_damage_taken: i32,
}

impl Fighter {
    pub fn new(
        hero: TarotCard,
        weapon: TarotCard,
        apparel: TarotCard,
        item: TarotCard,
    ) -> Fighter {
        let mut stats = derive_stats(hero, weapon, apparel, item);
        stats.hp = stats.hp.max(1);
        let hp = stats.hp;
        Fighter {
            hero, weapon, apparel, item, stats,
            current_hp: hp, max_hp: hp,
            heal_count: 0, last_damage_taken: 0,
        }
    }

    pub fn diminished_heal(&mut self, base_amount: i32) -> i32 {
        let effective = base_amount * 100 / (100 + self.heal_count * 15);
        self.heal_count += 1;
        effective.max(0)
    }

    pub fn is_knight(&self) -> bool {
        self.hero.court_rank() == Some(CourtRank::Knight)
    }

    pub fn is_queen(&self) -> bool {
        self.hero.court_rank() == Some(CourtRank::Queen)
    }

    pub fn reassign_equipment(&mut self, weapon: TarotCard, apparel: TarotCard, item: TarotCard) {
        let hp_ratio = if self.max_hp > 0 { self.current_hp as f64 / self.max_hp as f64 } else { 1.0 };
        self.weapon = weapon;
        self.apparel = apparel;
        self.item = item;
        let mut stats = derive_stats(self.hero, weapon, apparel, item);
        stats.hp = stats.hp.max(1);
        self.stats = stats;
        self.max_hp = stats.hp;
        self.current_hp = ((self.max_hp as f64 * hp_ratio).round() as i32).clamp(1, self.max_hp);
    }

    pub fn weapon_action_name(&self) -> &'static str {
        match self.weapon.suit() {
            Some(MinorSuit::Swords)    => "Strike",
            Some(MinorSuit::Cups)      => "Drain",
            Some(MinorSuit::Wands)     => "Quick Strike",
            Some(MinorSuit::Pentacles) => "Heavy Blow",
            None => "Strike",
        }
    }

    pub fn apparel_action_name(&self) -> &'static str {
        match self.apparel.suit() {
            Some(MinorSuit::Swords)    => "Riposte",
            Some(MinorSuit::Cups)      => "Restore",
            Some(MinorSuit::Wands)     => "Evade",
            Some(MinorSuit::Pentacles) => "Fortify",
            None => "Riposte",
        }
    }

    pub fn item_action_name(&self) -> &'static str {
        match self.item.suit() {
            Some(MinorSuit::Swords)    => "Backstab",
            Some(MinorSuit::Cups)      => "Elixir",
            Some(MinorSuit::Wands)     => "Haste",
            Some(MinorSuit::Pentacles) => "Barrier",
            None => "Backstab",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatAction {
    Weapon,
    Apparel,
    Item,
}

impl CombatAction {
    pub const ALL: [CombatAction; 3] = [CombatAction::Weapon, CombatAction::Apparel, CombatAction::Item];

    fn index(self) -> usize {
        match self {
            CombatAction::Weapon => 0,
            CombatAction::Apparel => 1,
            CombatAction::Item => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CombatAction::Weapon => "Weapon",
            CombatAction::Apparel => "Apparel",
            CombatAction::Item => "Item",
        }
    }
}

struct Roll {
    value: i32,
    low: i32,
    high: i32,
}

impl Roll {
    fn fmt(&self) -> String {
        if self.low == self.high {
            format!("{}", self.value)
        } else {
            format!("{} ({}~{})", self.value, self.low, self.high)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct FighterFlags {
    evade: bool,
    fortify: bool,
    riposte: bool,
    barrier: bool,
}

struct TurnFlags {
    flags: [FighterFlags; 2],
}

impl TurnFlags {
    fn none() -> Self {
        Self { flags: [FighterFlags::default(); 2] }
    }
    fn get(&self, side: Side) -> &FighterFlags {
        &self.flags[side.index()]
    }
    fn get_mut(&mut self, side: Side) -> &mut FighterFlags {
        &mut self.flags[side.index()]
    }
}

#[derive(Clone, Debug)]
pub struct CombatState {
    pub player: Fighter,
    pub ai: Fighter,
    pub log: Vec<String>,
    pub turn: u32,
    pub cycle: u32,
    exhausted: [[bool; 3]; 2],
    pub awaiting_action: bool,
    pub combat_over: bool,
    pub player_won: bool,
    pub rng: ChaCha8Rng,
    pub fate_pool: Vec<MajorArcana>,
    pub current_fate: Option<MajorArcana>,
    pub fate_effect: FateEffect,
    // Knight: which action is doubled this cycle (per side), and use counts
    pub knight_doubled: [Option<CombatAction>; 2],
    knight_uses: [[u8; 3]; 2],
    // Queen: reassignment state
    pub awaiting_queen_reassign: bool,
    pub queen_original_cards: [Option<[TarotCard; 3]>; 2],
    // Fate tracking
    star_barriers: [bool; 2],
    momentum_count: [i32; 2],
    first_weapon_this_cycle: [bool; 2],
    second_strike_active: bool,
    applied_flat_bonus: i32,
    items_swapped: bool,
    pub tweaks: BalanceTweaks,
}

impl CombatState {
    pub fn new(player: Fighter, ai: Fighter, rng: ChaCha8Rng) -> CombatState {
        Self::new_with_tweaks(player, ai, rng, BalanceTweaks::default())
    }

    pub fn new_with_tweaks(player: Fighter, ai: Fighter, mut rng: ChaCha8Rng, tweaks: BalanceTweaks) -> CombatState {
        let mut log = Vec::new();

        // Initialize fate pool with all 22 major arcana, shuffled
        let mut fate_pool: Vec<MajorArcana> = MajorArcana::ALL.to_vec();
        fate_pool.shuffle(&mut rng);

        // Draw first fate card
        let current_fate = fate_pool.pop();
        let fate_effect = current_fate.map(|a| resolve_fate(a)).unwrap_or_default();
        if let Some(arcana) = current_fate {
            log.push(format!("Fate: {} — {}", arcana, fate_description(arcana)));
        }
        log.push("--- Cycle 1 ---".to_string());

        // Store original cards for Queen reassignment
        let queen_original_cards = [
            if player.is_queen() { Some([player.weapon, player.apparel, player.item]) } else { None },
            if ai.is_queen() { Some([ai.weapon, ai.apparel, ai.item]) } else { None },
        ];

        // Roll Knight doubles for cycle 1
        let mut knight_doubled = [None; 2];
        if player.is_knight() {
            let action = CombatAction::ALL[rng.random_range(0..3)];
            knight_doubled[0] = Some(action);
            log.push(format!("Your Knight doubles: {} x2!", action.name()));
        }
        if ai.is_knight() {
            let action = CombatAction::ALL[rng.random_range(0..3)];
            knight_doubled[1] = Some(action);
            log.push(format!("Enemy Knight doubles: {} x2!", action.name()));
        }

        // Star barrier init
        let star_barriers = [fate_effect.start_with_barrier, fate_effect.start_with_barrier];

        // World flat stat bonus
        let applied_flat_bonus = fate_effect.flat_stat_bonus;

        let mut state = CombatState {
            player, ai, log, turn: 1, cycle: 1,
            exhausted: [[false; 3]; 2],
            awaiting_action: true, combat_over: false, player_won: false,
            rng,
            fate_pool, current_fate, fate_effect,
            knight_doubled,
            knight_uses: [[0; 3]; 2],
            awaiting_queen_reassign: false,
            queen_original_cards,
            star_barriers,
            momentum_count: [0; 2],
            first_weapon_this_cycle: [false; 2],
            second_strike_active: false,
            applied_flat_bonus: 0,
            items_swapped: false,
            tweaks,
        };

        // Apply World bonus
        if applied_flat_bonus != 0 {
            state.apply_flat_bonus(applied_flat_bonus);
            state.applied_flat_bonus = applied_flat_bonus;
        }

        // Fool: swap items
        if fate_effect.swap_items {
            state.swap_items();
        }

        state
    }

    fn swap_items(&mut self) {
        let p_item = self.player.item;
        let a_item = self.ai.item;
        self.player.reassign_equipment(self.player.weapon, self.player.apparel, a_item);
        self.ai.reassign_equipment(self.ai.weapon, self.ai.apparel, p_item);
        self.items_swapped = !self.items_swapped;
        if self.items_swapped {
            self.log.push("The Fool swaps items between fighters!".to_string());
        } else {
            self.log.push("Items return to their original owners.".to_string());
        }
    }

    fn apply_flat_bonus(&mut self, bonus: i32) {
        for side in [Side::Player, Side::Ai] {
            let f = self.fighter_mut(side);
            f.stats.attack += bonus;
            f.stats.defense += bonus;
            f.stats.speed += bonus;
            f.stats.hp += bonus;
            f.max_hp += bonus;
            f.current_hp = (f.current_hp + bonus).max(1);
        }
    }

    fn draw_fate(&mut self) {
        // Undo previous Fool swap
        if self.items_swapped {
            self.swap_items();
        }

        // Undo previous World bonus
        if self.applied_flat_bonus != 0 {
            self.apply_flat_bonus(-self.applied_flat_bonus);
            self.applied_flat_bonus = 0;
        }

        self.current_fate = self.fate_pool.pop();
        self.fate_effect = self.current_fate.map(|a| resolve_fate(a)).unwrap_or_default();
        if let Some(arcana) = self.current_fate {
            self.log.push(format!("Fate: {} — {}", arcana, fate_description(arcana)));
        }

        // Star barrier reset
        self.star_barriers = [self.fate_effect.start_with_barrier, self.fate_effect.start_with_barrier];

        // Reset per-cycle tracking
        self.first_weapon_this_cycle = [false; 2];
        self.momentum_count = [0; 2];

        // Apply new World bonus
        if self.fate_effect.flat_stat_bonus != 0 {
            self.apply_flat_bonus(self.fate_effect.flat_stat_bonus);
            self.applied_flat_bonus = self.fate_effect.flat_stat_bonus;
        }

        // Fool: swap items
        if self.fate_effect.swap_items {
            self.swap_items();
        }
    }

    pub fn fighter(&self, side: Side) -> &Fighter {
        match side { Side::Player => &self.player, Side::Ai => &self.ai }
    }

    fn fighter_mut(&mut self, side: Side) -> &mut Fighter {
        match side { Side::Player => &mut self.player, Side::Ai => &mut self.ai }
    }

    fn exhausted(&self, side: Side) -> &[bool; 3] {
        &self.exhausted[side.index()]
    }

    fn exhausted_mut(&mut self, side: Side) -> &mut [bool; 3] {
        &mut self.exhausted[side.index()]
    }

    pub fn effective_stats(&self, side: Side) -> Stats {
        let f = self.fighter(side);
        if self.fate_effect.swap_atk_def {
            Stats { attack: f.stats.defense, defense: f.stats.attack, ..f.stats }
        } else {
            f.stats
        }
    }

    fn effective_atk(&self, side: Side) -> i32 {
        let f = self.fighter(side);
        if self.fate_effect.swap_atk_def { f.stats.defense } else { f.stats.attack }
    }

    fn effective_def(&self, side: Side) -> i32 {
        let f = self.fighter(side);
        if self.fate_effect.swap_atk_def { f.stats.attack } else { f.stats.defense }
    }

    fn roll_effect(&mut self, base: i32, floor: i32) -> Roll {
        if base <= floor {
            return Roll { value: base, low: base, high: base };
        }
        let fe = &self.fate_effect;
        if fe.use_max_rolls {
            return Roll { value: base, low: base, high: base };
        }
        if fe.use_min_rolls {
            let low = floor.max(base * 2 / 3);
            return Roll { value: low, low, high: base };
        }
        let (low, high) = if fe.chaos_rolls {
            (0, base * 2)
        } else {
            (floor.max(base * 2 / 3), base)
        };
        if low >= high {
            return Roll { value: high, low: high, high };
        }
        let value = self.rng.random_range(low..=high);
        Roll { value, low, high }
    }

    fn apply_heal(&mut self, side: Side, base_heal: i32) -> i32 {
        if self.fate_effect.disable_healing {
            return 0;
        }
        let adjusted = base_heal * self.fate_effect.heal_multiplier_pct / 100;
        let f = self.fighter_mut(side);
        let heal = f.diminished_heal(adjusted);
        f.current_hp = (f.current_hp + heal).min(f.max_hp);
        heal
    }

    pub fn action_available(&self, side: Side, action: CombatAction) -> bool {
        if self.exhausted(side)[action.index()] {
            return false;
        }
        // Knight doubled action: available if uses < 2
        if self.knight_doubled[side.index()] == Some(action) {
            if self.knight_uses[side.index()][action.index()] >= 2 {
                return false;
            }
        }
        // Hierophant forced order
        if self.fate_effect.force_action_order {
            let order = [CombatAction::Weapon, CombatAction::Apparel, CombatAction::Item];
            for &required in &order {
                if required == action {
                    return true;
                }
                // If this earlier action is still available, block later ones
                let ri = required.index();
                let exhausted = self.exhausted(side)[ri];
                if !exhausted {
                    // Knight: check if doubled action still has uses
                    if self.knight_doubled[side.index()] == Some(required) {
                        if self.knight_uses[side.index()][ri] < 2 {
                            return false; // Must use this doubled action first
                        }
                    } else {
                        return false; // Must use this action first
                    }
                }
            }
        }
        true
    }

    pub fn available_actions(&self, side: Side) -> Vec<CombatAction> {
        CombatAction::ALL.iter().copied()
            .filter(|a| self.action_available(side, *a))
            .collect()
    }

    fn exhaust(&mut self, side: Side, action: CombatAction) {
        let si = side.index();
        let ai = action.index();
        if self.knight_doubled[si] == Some(action) {
            self.knight_uses[si][ai] += 1;
            if self.knight_uses[si][ai] >= 2 {
                self.exhausted_mut(side)[ai] = true;
            }
        } else {
            self.exhausted_mut(side)[ai] = true;
        }
    }

    fn side_cycle_complete(&self, side: Side) -> bool {
        let si = side.index();
        if self.knight_doubled[si].is_none() {
            return self.exhausted(side).iter().all(|e| *e);
        }
        // Knight: count total turns used this cycle
        let mut turns_used: u8 = 0;
        for action in CombatAction::ALL {
            let ai = action.index();
            if self.knight_doubled[si] == Some(action) {
                turns_used += self.knight_uses[si][ai];
            } else if self.exhausted(side)[ai] {
                turns_used += 1;
            }
        }
        turns_used >= 3
    }

    pub fn knight_action_uses(&self, side: Side, action: CombatAction) -> u8 {
        self.knight_uses[side.index()][action.index()]
    }

    fn check_both_cycles(&mut self) {
        let player_done = self.side_cycle_complete(Side::Player);
        let ai_done = self.side_cycle_complete(Side::Ai);

        if player_done {
            for a in CombatAction::ALL {
                self.exhausted_mut(Side::Player)[a.index()] = true;
            }
            *self.exhausted_mut(Side::Player) = [false; 3];
            self.knight_uses[0] = [0; 3];
            self.first_weapon_this_cycle[0] = false;
            self.momentum_count[0] = 0;
        }
        if ai_done {
            for a in CombatAction::ALL {
                self.exhausted_mut(Side::Ai)[a.index()] = true;
            }
            *self.exhausted_mut(Side::Ai) = [false; 3];
            self.knight_uses[1] = [0; 3];
            self.first_weapon_this_cycle[1] = false;
            self.momentum_count[1] = 0;
        }
        if player_done && ai_done {
            self.cycle += 1;
            self.log.push(format!("--- Cycle {} ---", self.cycle));
            self.draw_fate();

            // Roll new Knight doubles
            for (si, side) in [(0, Side::Player), (1, Side::Ai)] {
                if self.fighter(side).is_knight() {
                    let action = CombatAction::ALL[self.rng.random_range(0..3)];
                    self.knight_doubled[si] = Some(action);
                    let name = side.name();
                    self.log.push(format!("{name} Knight doubles: {} x2!", action.name()));
                }
            }

            // Queen reassignment (cycle 2+)
            self.trigger_queen_reassign();
        }
    }

    fn trigger_queen_reassign(&mut self) {
        // AI Queen: reassign immediately
        if self.ai.is_queen() {
            if let Some(cards) = self.queen_original_cards[1] {
                let best = crate::ai::queen_reassign(&self.ai, cards);
                self.ai.reassign_equipment(best.0, best.1, best.2);
                self.log.push(format!("Enemy Queen reshapes: Wpn={} App={} Itm={}", best.0, best.1, best.2));
            }
        }
        // Player Queen: pause for input
        if self.player.is_queen() {
            self.awaiting_queen_reassign = true;
            self.awaiting_action = false;
            self.log.push("Your Queen can reassign equipment! [Left/Right] to cycle, [Enter] to confirm.".to_string());
        }
    }

    pub fn queen_reassign_complete(&mut self, weapon: TarotCard, apparel: TarotCard, item: TarotCard) {
        self.player.reassign_equipment(weapon, apparel, item);
        self.awaiting_queen_reassign = false;
        self.awaiting_action = true;
        self.log.push(format!("Your Queen reshapes: Wpn={weapon} App={apparel} Itm={item}"));
    }

    pub fn resolve_turn(&mut self, player_action: CombatAction, ai_action: CombatAction) {
        self.player.last_damage_taken = 0;
        self.ai.last_damage_taken = 0;

        let mut flags = TurnFlags::none();

        let player_wands_weapon = self.player.weapon.suit() == Some(MinorSuit::Wands);
        let ai_wands_weapon = self.ai.weapon.suit() == Some(MinorSuit::Wands);

        let player_first = {
            let reverse = self.fate_effect.reverse_initiative;
            let ps = self.player.stats.speed + if player_wands_weapon && player_action == CombatAction::Weapon { 999 } else { 0 };
            let as_ = self.ai.stats.speed + if ai_wands_weapon && ai_action == CombatAction::Weapon { 999 } else { 0 };
            if reverse { ps <= as_ } else { ps >= as_ }
        };

        self.exhaust(Side::Player, player_action);
        self.exhaust(Side::Ai, ai_action);

        self.apply_apparel(player_action, Side::Player, &mut flags);
        self.apply_apparel(ai_action, Side::Ai, &mut flags);

        let (first_side, first_action, second_side, second_action) = if player_first {
            (Side::Player, player_action, Side::Ai, ai_action)
        } else {
            (Side::Ai, ai_action, Side::Player, player_action)
        };

        self.apply_action(first_action, first_side, &mut flags);
        if self.fighter(second_side).current_hp > 0 {
            self.second_strike_active = true;
            self.apply_action(second_action, second_side, &mut flags);
            self.second_strike_active = false;
        }

        // Momentum reset: if action was not Weapon, reset momentum
        if player_action != CombatAction::Weapon {
            self.momentum_count[0] = 0;
        }
        if ai_action != CombatAction::Weapon {
            self.momentum_count[1] = 0;
        }

        if self.fate_effect.heal_per_turn > 0 {
            for side in [Side::Player, Side::Ai] {
                let f = self.fighter(side);
                if f.current_hp > 0 {
                    let base = self.fate_effect.heal_per_turn;
                    let floor = self.fate_effect.damage_floor.unwrap_or(HEAL_FLOOR);
                    let roll = self.roll_effect(base, floor);
                    let name = side.name();
                    let verb = match side { Side::Player => "regenerate", Side::Ai => "regenerates" };
                    if self.fate_effect.disable_healing {
                        self.log.push(format!("{name} — healing disabled (Fate)"));
                    } else {
                        let adjusted = roll.value * self.fate_effect.heal_multiplier_pct / 100;
                        let f = self.fighter_mut(side);
                        let h = f.diminished_heal(adjusted);
                        f.current_hp = (f.current_hp + h).min(f.max_hp);
                        self.log.push(format!("{name} {verb} {h} HP (Fate) [base {base}, rolled {}]", roll.fmt()));
                    }
                }
            }
        }

        if self.player.current_hp <= 0 {
            self.combat_over = true;
            self.player_won = false;
        } else if self.ai.current_hp <= 0 {
            self.combat_over = true;
            self.player_won = true;
        }

        if !self.combat_over {
            self.check_both_cycles();
        }

        self.turn += 1;
        self.awaiting_action = true;
    }

    fn def_breakdown(&self, side: Side, flags: &TurnFlags) -> (i32, String) {
        let ff = flags.get(side);
        let raw = self.effective_def(side) * self.fate_effect.defense_multiplier_pct / 100;
        let spd = self.fighter(side).stats.speed;

        if ff.fortify {
            let eff = raw * 3;
            (eff, format!("{raw} DEF x3 Fortify = {eff}"))
        } else if ff.riposte {
            let eff = raw * 3 / 2;
            let evade_part = if ff.evade { spd } else { 0 };
            let total = eff + evade_part;
            if evade_part > 0 {
                (total, format!("{raw} DEF x1.5 Riposte + {spd} SPD Evade = {total}"))
            } else {
                (total, format!("{raw} DEF x1.5 Riposte = {total}"))
            }
        } else if ff.evade {
            let total = raw + spd;
            (total, format!("{raw} DEF + {spd} SPD Evade = {total}"))
        } else {
            (raw, format!("{raw} DEF"))
        }
    }

    fn apply_apparel(&mut self, action: CombatAction, side: Side, flags: &mut TurnFlags) {
        if action != CombatAction::Apparel { return; }
        let f = self.fighter(side);
        let suit = f.apparel.suit();
        let raw_def = self.effective_def(side);
        let spd = f.stats.speed;
        let name = side.name();

        match suit {
            Some(MinorSuit::Swords) => {
                flags.get_mut(side).riposte = true;
                self.log.push(format!("{name} readies Riposte [{raw_def} DEF x1.5 = {} effective]", raw_def * 3 / 2));
            }
            Some(MinorSuit::Wands) => {
                flags.get_mut(side).evade = true;
                self.log.push(format!("{name} uses Evade [{raw_def} DEF + {spd} SPD = {} effective]", raw_def + spd));
            }
            Some(MinorSuit::Pentacles) => {
                flags.get_mut(side).fortify = true;
                self.log.push(format!("{name} uses Fortify [{raw_def} DEF x3 = {} effective]", raw_def * 3));
            }
            Some(MinorSuit::Cups) => {
                let last_dmg = self.fighter(side).last_damage_taken;
                let card_val = self.fighter(side).apparel.numbered_value().unwrap_or(1) as i32;
                let base_heal = if last_dmg > 0 {
                    last_dmg * 60 / 100
                } else {
                    card_val / 2
                };
                let base = base_heal.max(1);
                let roll = self.roll_effect(base, HEAL_FLOOR);
                if self.fate_effect.disable_healing {
                    self.log.push(format!("{name} uses Restore — healing disabled (Fate)"));
                } else {
                    let heal = self.apply_heal(side, roll.value);
                    if last_dmg > 0 {
                        self.log.push(format!("{name} uses Restore — {heal} HP [{last_dmg} dmg taken x60% = {base}, rolled {}]", roll.fmt()));
                    } else {
                        self.log.push(format!("{name} uses Restore — {heal} HP (minor) [rolled {}]", roll.fmt()));
                    }
                }
            }
            None => {}
        }
    }

    fn apply_action(&mut self, action: CombatAction, side: Side, flags: &mut TurnFlags) {
        match action {
            CombatAction::Weapon  => self.apply_weapon(side, flags),
            CombatAction::Apparel => {}
            CombatAction::Item    => self.apply_item(side, flags),
        }
    }

    fn apply_weapon(&mut self, side: Side, flags: &mut TurnFlags) {
        let atk = self.effective_atk(side);
        let suit = self.fighter(side).weapon.suit();
        let target = side.opponent();
        let (def, def_str) = self.def_breakdown(target, flags);
        let barrier = flags.get(target).barrier;
        let name = side.name();
        let tgt = target.target_name();
        let dmg_floor = self.fate_effect.damage_floor.unwrap_or(DAMAGE_FLOOR);
        let dmg_bonus = self.fate_effect.damage_bonus;
        let dmg_mult = self.fate_effect.damage_multiplier_pct;
        let momentum = self.fate_effect.momentum_bonus * self.momentum_count[side.index()];

        if barrier {
            self.log.push(format!("{name} attacks but the Barrier blocks all damage!"));
            return;
        }

        let is_first_weapon = self.first_weapon_this_cycle[side.index()];
        let magician = self.fate_effect.first_action_double;

        let calc_final_dmg = |raw_dmg: i32| -> i32 {
            let mut dmg = raw_dmg * dmg_mult / 100;
            dmg += momentum;
            if magician && !is_first_weapon {
                dmg *= 2;
            }
            dmg
        };

        // Build fate modifier suffix for readout
        let mut mods = Vec::new();
        if dmg_bonus != 0 { mods.push(format!("+{dmg_bonus} Sun")); }
        if dmg_mult != 100 { mods.push(format!("x{}%", dmg_mult)); }
        if momentum > 0 { mods.push(format!("+{momentum} momentum")); }
        if magician && !is_first_weapon { mods.push("x2 Magician".to_string()); }
        let fate_suffix = if mods.is_empty() { String::new() } else { format!(", {}", mods.join(", ")) };

        let bonus_str = if dmg_bonus != 0 { format!(" + {dmg_bonus}") } else { String::new() };

        match suit {
            Some(MinorSuit::Swords) | None => {
                let base = (atk + dmg_bonus - def).max(dmg_floor);
                let roll = self.roll_effect(base, dmg_floor);
                let dmg = calc_final_dmg(roll.value);
                let detail = format!("{atk} ATK{bonus_str} - {def_str} = {base}, rolled {}{fate_suffix}", roll.fmt());
                self.deal_damage(Some(side), target, dmg);
                self.riposte_check(side, flags);
                self.log.push(format!("{name} uses Strike on {tgt} for {dmg} damage [{detail}]"));
            }
            Some(MinorSuit::Cups) => {
                let raw = atk * 70 / 100;
                let base = (raw + dmg_bonus - def).max(dmg_floor);
                let roll = self.roll_effect(base, dmg_floor);
                let dmg = calc_final_dmg(roll.value);
                let detail = format!("{atk} ATK x70% = {raw}{bonus_str} - {def_str} = {base}, rolled {}{fate_suffix}", roll.fmt());
                self.deal_damage(Some(side), target, dmg);
                self.riposte_check(side, flags);
                if self.fate_effect.disable_healing {
                    self.log.push(format!("{name} uses Drain on {tgt} for {dmg} damage — healing disabled (Fate) [{detail}]"));
                } else {
                    let heal_roll = self.roll_effect(dmg, HEAL_FLOOR);
                    let heal = self.apply_heal(side, heal_roll.value);
                    self.log.push(format!("{name} uses Drain on {tgt} for {dmg} damage, heals {heal} HP [{detail}; heal rolled {}]", heal_roll.fmt()));
                }
            }
            Some(MinorSuit::Wands) => {
                let raw = atk * 70 / 100;
                let base = (raw + dmg_bonus - def).max(dmg_floor);
                let roll = self.roll_effect(base, dmg_floor);
                let dmg = calc_final_dmg(roll.value);
                let detail = format!("{atk} ATK x70% = {raw}{bonus_str} - {def_str} = {base}, rolled {}{fate_suffix}", roll.fmt());
                self.deal_damage(Some(side), target, dmg);
                self.riposte_check(side, flags);
                self.log.push(format!("{name} uses Quick Strike on {tgt} for {dmg} damage [{detail}]"));
            }
            Some(MinorSuit::Pentacles) => {
                let raw_atk = atk * 120 / 100;
                let half_def = def / 2;
                let base = (raw_atk + dmg_bonus - half_def).max(dmg_floor);
                let roll = self.roll_effect(base, dmg_floor);
                let dmg = calc_final_dmg(roll.value);
                let detail = format!("{atk} ATK x120% = {raw_atk}{bonus_str} - {def_str}/2 = {base}, rolled {}{fate_suffix}", roll.fmt());
                self.deal_damage(Some(side), target, dmg);
                self.riposte_check(side, flags);
                self.log.push(format!("{name} uses Heavy Blow on {tgt} for {dmg} damage [{detail}]"));
            }
        }

        // Magician: mark first weapon used
        if !self.first_weapon_this_cycle[side.index()] {
            self.first_weapon_this_cycle[side.index()] = true;
        }
        // Chariot: increment momentum
        self.momentum_count[side.index()] += 1;
    }

    fn riposte_check(&mut self, attacker: Side, flags: &TurnFlags) {
        let defender = attacker.opponent();
        if !flags.get(defender).riposte { return; }
        let def_atk = self.effective_atk(defender);
        let base = (def_atk * 30 / 100).max(1);
        let roll = self.roll_effect(base, 1);
        let counter = roll.value;
        self.deal_damage(Some(defender), attacker, counter);
        let name = defender.name();
        let tgt = attacker.target_name();
        self.log.push(format!("{name} Ripostes {tgt} for {counter} damage [{def_atk} ATK x30% = {base}, rolled {}]", roll.fmt()));
    }

    fn apply_item(&mut self, side: Side, flags: &mut TurnFlags) {
        let f = self.fighter(side);
        let suit = f.item.suit();
        let val = f.item.numbered_value().unwrap_or(1) as i32;
        let name = side.name();
        let target = side.opponent();
        let tgt = target.target_name();

        match suit {
            Some(MinorSuit::Swords) | None => {
                let atk = self.effective_atk(side);
                let raw_def = self.effective_def(target);
                let def_applied = raw_def * 3 / 4;
                let dmg_floor = self.fate_effect.damage_floor.unwrap_or(DAMAGE_FLOOR);
                let base = (atk + val - def_applied).max(dmg_floor);
                let roll = self.roll_effect(base, dmg_floor);
                let dmg = roll.value;
                self.deal_damage(Some(side), target, dmg);
                self.log.push(format!("{name} uses Backstab on {tgt} for {dmg} damage [{atk} ATK + {val} card - {raw_def} DEF x3/4 = {base}, rolled {}]", roll.fmt()));
            }
            Some(MinorSuit::Cups) => {
                let base = val * 2;
                let roll = self.roll_effect(base, HEAL_FLOOR);
                if self.fate_effect.disable_healing {
                    self.log.push(format!("{name} uses Elixir — healing disabled (Fate)"));
                } else {
                    let heal = self.apply_heal(side, roll.value);
                    self.log.push(format!("{name} uses Elixir — heals {heal} HP [{val} card x2 = {base}, rolled {}]", roll.fmt()));
                }
            }
            Some(MinorSuit::Wands) => {
                self.log.push(format!("{name} uses Haste — bonus weapon + apparel!"));
                self.apply_weapon(side, flags);
                self.apply_apparel(CombatAction::Apparel, side, flags);
            }
            Some(MinorSuit::Pentacles) => {
                flags.get_mut(side).barrier = true;
                self.log.push(format!("{name} uses Barrier — immune to damage this turn!"));
            }
        }
    }

    fn deal_damage(&mut self, attacker: Option<Side>, target: Side, dmg: i32) {
        // Star barrier absorbs first hit
        if self.star_barriers[target.index()] {
            self.star_barriers[target.index()] = false;
            let tgt_name = target.name();
            self.log.push(format!("{tgt_name}'s Star barrier absorbs the hit!"));
            return;
        }

        let mut final_dmg = dmg;

        // Cycle escalation tweak
        if self.tweaks.cycle_damage_bonus > 0 {
            final_dmg += self.tweaks.cycle_damage_bonus * (self.cycle as i32 - 1);
        }

        // HP bulk bonus tweak: attackers with high max HP deal bonus damage
        if let Some(atk_side) = attacker {
            if self.tweaks.hp_bulk_per > 0 && self.tweaks.hp_bulk_threshold > 0 {
                let f = self.fighter(atk_side);
                let excess = f.max_hp - self.tweaks.hp_bulk_threshold;
                if excess > 0 {
                    final_dmg += excess / self.tweaks.hp_bulk_per;
                }
            }
        }

        // HP ratio bonus tweak: attacker above 50% HP deals more
        if let Some(atk_side) = attacker {
            if self.tweaks.hp_ratio_scale > 0 {
                let f = self.fighter(atk_side);
                let hp_pct = f.current_hp * 100 / f.max_hp.max(1);
                let bonus = (hp_pct - 50) * self.tweaks.hp_ratio_scale / 100;
                if bonus > 0 {
                    final_dmg += bonus;
                }
            }
        }

        // Death execute: target below threshold takes double
        if self.fate_effect.execute_threshold_pct > 0 {
            let f = self.fighter(target);
            let hp_pct = f.current_hp * 100 / f.max_hp.max(1);
            if hp_pct <= self.fate_effect.execute_threshold_pct {
                final_dmg *= 2;
            }
        }

        // Judgement: second strike bonus
        if self.second_strike_active && self.fate_effect.second_strike_bonus_pct != 100 {
            final_dmg = final_dmg * self.fate_effect.second_strike_bonus_pct / 100;
        }

        // Apply damage
        let f = self.fighter_mut(target);
        f.current_hp -= final_dmg;
        f.last_damage_taken += final_dmg;

        // Passive thorns tweak: target reflects DEF% back
        if let Some(atk_side) = attacker {
            let thorns_pct = self.tweaks.thorns_pct;
            let target_def = self.effective_def(target);
            if thorns_pct > 0 && target_def > 0 {
                let thorns = (target_def * thorns_pct / 100).max(1);
                let f = self.fighter_mut(atk_side);
                f.current_hp -= thorns;
                f.last_damage_taken += thorns;
            }
        }

        // Lovers reflect (no recursion)
        if let Some(atk_side) = attacker {
            if self.fate_effect.reflect_damage_pct > 0 {
                let reflected = final_dmg * self.fate_effect.reflect_damage_pct / 100;
                if reflected > 0 {
                    let f = self.fighter_mut(atk_side);
                    f.current_hp -= reflected;
                    f.last_damage_taken += reflected;
                    let name = atk_side.name();
                    self.log.push(format!("{name} takes {reflected} reflected damage (Lovers)"));
                }
            }
        }
    }
}
