use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use crate::card::{CourtRank, MinorSuit, TarotCard};
use crate::stats::{Stats, derive_stats};
use crate::arcana::{ArcanaEffect, resolve_arcana};

const DAMAGE_FLOOR: i32 = 2;
const HEAL_FLOOR: i32 = 1;

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
    pub arcana: TarotCard,
    pub stats: Stats,
    pub current_hp: i32,
    pub max_hp: i32,
    pub arcana_effect: ArcanaEffect,
    pub heal_count: i32,
    pub last_damage_taken: i32,
}

impl Fighter {
    pub fn new(
        hero: TarotCard,
        weapon: TarotCard,
        apparel: TarotCard,
        item: TarotCard,
        arcana_card: TarotCard,
    ) -> Fighter {
        let mut stats = derive_stats(hero, weapon, apparel, item);
        let arcana = arcana_card.arcana().unwrap();
        let arcana_effect = resolve_arcana(arcana, hero.suit(), &[weapon, apparel, item]);
        stats.add(&arcana_effect.stat_bonus);
        stats.hp = stats.hp.max(1);
        let hp = stats.hp;
        Fighter {
            hero, weapon, apparel, item, arcana: arcana_card, stats,
            current_hp: hp, max_hp: hp, arcana_effect,
            heal_count: 0, last_damage_taken: 0,
        }
    }

    pub fn diminished_heal(&mut self, base_amount: i32) -> i32 {
        let effective = base_amount * 100 / (100 + self.heal_count * 25);
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
        let arcana = self.arcana.arcana().unwrap();
        let effect = resolve_arcana(arcana, self.hero.suit(), &[weapon, apparel, item]);
        stats.add(&effect.stat_bonus);
        stats.hp = stats.hp.max(1);
        self.stats = stats;
        self.arcana_effect = effect;
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
    // Knight: which action is doubled this cycle (per side), and use counts
    pub knight_doubled: [Option<CombatAction>; 2],
    knight_uses: [[u8; 3]; 2],
    // Queen: reassignment state
    pub awaiting_queen_reassign: bool,
    pub queen_original_cards: [Option<[TarotCard; 3]>; 2],
}

impl CombatState {
    pub fn new(player: Fighter, ai: Fighter, mut rng: ChaCha8Rng) -> CombatState {
        let mut log = Vec::new();
        log.push(format!("Your arcana: {}", player.arcana));
        log.push(format!("Enemy arcana: {}", ai.arcana));
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

        CombatState {
            player, ai, log, turn: 1, cycle: 1,
            exhausted: [[false; 3]; 2],
            awaiting_action: true, combat_over: false, player_won: false,
            rng,
            knight_doubled,
            knight_uses: [[0; 3]; 2],
            awaiting_queen_reassign: false,
            queen_original_cards,
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

    fn roll_effect(&mut self, base: i32, floor: i32) -> Roll {
        if base <= floor {
            return Roll { value: base, low: base, high: base };
        }
        let low = floor.max(base * 2 / 3);
        if low >= base {
            return Roll { value: base, low: base, high: base };
        }
        let value = self.rng.random_range(low..=base);
        Roll { value, low, high: base }
    }

    pub fn action_available(&self, side: Side, action: CombatAction) -> bool {
        if self.exhausted(side)[action.index()] {
            return false;
        }
        // Knight doubled action: available if uses < 2
        if self.knight_doubled[side.index()] == Some(action) {
            return self.knight_uses[side.index()][action.index()] < 2;
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
            // Auto-exhaust remaining actions (Knight's skipped action)
            for a in CombatAction::ALL {
                self.exhausted_mut(Side::Player)[a.index()] = true;
            }
            *self.exhausted_mut(Side::Player) = [false; 3];
            self.knight_uses[0] = [0; 3];
        }
        if ai_done {
            for a in CombatAction::ALL {
                self.exhausted_mut(Side::Ai)[a.index()] = true;
            }
            *self.exhausted_mut(Side::Ai) = [false; 3];
            self.knight_uses[1] = [0; 3];
        }
        if player_done && ai_done {
            self.cycle += 1;
            self.log.push(format!("--- Cycle {} ---", self.cycle));

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
            let pa = self.player.arcana_effect.always_first;
            let aa = self.ai.arcana_effect.always_first;
            if pa && !aa { true }
            else if aa && !pa { false }
            else {
                let ps = self.player.stats.speed + if player_wands_weapon && player_action == CombatAction::Weapon { 999 } else { 0 };
                let as_ = self.ai.stats.speed + if ai_wands_weapon && ai_action == CombatAction::Weapon { 999 } else { 0 };
                ps >= as_
            }
        };

        self.exhaust(Side::Player, player_action);
        self.exhaust(Side::Ai, ai_action);

        self.apply_apparel(player_action, Side::Player, &mut flags);
        self.apply_apparel(ai_action, Side::Ai, &mut flags);

        if player_first {
            self.apply_action(player_action, Side::Player, &mut flags);
            if self.ai.current_hp > 0 {
                self.apply_action(ai_action, Side::Ai, &mut flags);
            }
        } else {
            self.apply_action(ai_action, Side::Ai, &mut flags);
            if self.player.current_hp > 0 {
                self.apply_action(player_action, Side::Player, &mut flags);
            }
        }

        for side in [Side::Player, Side::Ai] {
            let f = self.fighter(side);
            if f.arcana_effect.heal_per_turn > 0 && f.current_hp > 0 {
                let base = f.arcana_effect.heal_per_turn;
                let roll = self.roll_effect(base, HEAL_FLOOR);
                let f = self.fighter_mut(side);
                let h = f.diminished_heal(roll.value);
                f.current_hp = (f.current_hp + h).min(f.max_hp);
                let name = side.name();
                let verb = match side { Side::Player => "regenerate", Side::Ai => "regenerates" };
                self.log.push(format!("{name} {verb} {h} HP (Temperance) [base {base}, rolled {}]", roll.fmt()));
            }
        }

        if self.player.current_hp <= 0 {
            self.combat_over = true;
            self.player_won = false;
            if self.ai.arcana_effect.death_curse {
                self.ai.current_hp = 1;
                self.log.push("Death curse: enemy carries 1 HP to next round.".to_string());
            }
        } else if self.ai.current_hp <= 0 {
            self.combat_over = true;
            self.player_won = true;
            if self.player.arcana_effect.death_curse {
                self.player.current_hp = 1;
                self.log.push("Death curse: you carry 1 HP to next round.".to_string());
            }
        }

        if !self.combat_over {
            self.check_both_cycles();
        }

        self.turn += 1;
        self.awaiting_action = true;
    }

    fn def_breakdown(&self, side: Side, flags: &TurnFlags) -> (i32, String) {
        let f = self.fighter(side);
        let ff = flags.get(side);
        let raw = f.stats.defense;
        let spd = f.stats.speed;

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
        let raw_def = f.stats.defense;
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
                let f = self.fighter_mut(side);
                let heal = f.diminished_heal(roll.value);
                f.current_hp = (f.current_hp + heal).min(f.max_hp);
                if last_dmg > 0 {
                    self.log.push(format!("{name} uses Restore — {heal} HP [{last_dmg} dmg taken x60% = {base}, rolled {}]", roll.fmt()));
                } else {
                    self.log.push(format!("{name} uses Restore — {heal} HP (minor) [rolled {}]", roll.fmt()));
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
        let f = self.fighter(side);
        let atk = f.stats.attack;
        let suit = f.weapon.suit();
        let chariot = f.arcana_effect.first_hit_double;
        let target = side.opponent();
        let (def, def_str) = self.def_breakdown(target, flags);
        let barrier = flags.get(target).barrier;
        let name = side.name();
        let tgt = target.target_name();

        if barrier {
            self.log.push(format!("{name} attacks but the Barrier blocks all damage!"));
            return;
        }

        let first_turn = self.turn == 1;

        match suit {
            Some(MinorSuit::Swords) | None => {
                let base = (atk - def).max(DAMAGE_FLOOR);
                let roll = self.roll_effect(base, DAMAGE_FLOOR);
                let mut dmg = roll.value;
                let mut detail = format!("{atk} ATK - {def_str} = {base}, rolled {}", roll.fmt());
                if chariot && first_turn { dmg *= 2; detail = format!("{detail}, x2 Chariot"); }
                self.deal_damage(target, dmg);
                self.riposte_check(side, flags);
                self.log.push(format!("{name} uses Strike on {tgt} for {dmg} damage [{detail}]"));
            }
            Some(MinorSuit::Cups) => {
                let raw = atk * 60 / 100;
                let base = (raw - def).max(DAMAGE_FLOOR);
                let roll = self.roll_effect(base, DAMAGE_FLOOR);
                let mut dmg = roll.value;
                let mut detail = format!("{atk} ATK x60% = {raw} - {def_str} = {base}, rolled {}", roll.fmt());
                if chariot && first_turn { dmg *= 2; detail = format!("{detail}, x2 Chariot"); }
                self.deal_damage(target, dmg);
                self.riposte_check(side, flags);
                let heal_roll = self.roll_effect(dmg, HEAL_FLOOR);
                let f = self.fighter_mut(side);
                let heal = f.diminished_heal(heal_roll.value);
                f.current_hp = (f.current_hp + heal).min(f.max_hp);
                self.log.push(format!("{name} uses Drain on {tgt} for {dmg} damage, heals {heal} HP [{detail}; heal rolled {}]", heal_roll.fmt()));
            }
            Some(MinorSuit::Wands) => {
                let raw = atk * 70 / 100;
                let base = (raw - def).max(DAMAGE_FLOOR);
                let roll = self.roll_effect(base, DAMAGE_FLOOR);
                let mut dmg = roll.value;
                let mut detail = format!("{atk} ATK x70% = {raw} - {def_str} = {base}, rolled {}", roll.fmt());
                if chariot && first_turn { dmg *= 2; detail = format!("{detail}, x2 Chariot"); }
                self.deal_damage(target, dmg);
                self.riposte_check(side, flags);
                self.log.push(format!("{name} uses Quick Strike on {tgt} for {dmg} damage [{detail}]"));
            }
            Some(MinorSuit::Pentacles) => {
                let raw_atk = atk * 120 / 100;
                let half_def = def / 2;
                let base = (raw_atk - half_def).max(DAMAGE_FLOOR);
                let roll = self.roll_effect(base, DAMAGE_FLOOR);
                let mut dmg = roll.value;
                let mut detail = format!("{atk} ATK x120% = {raw_atk} - {def_str}/2 = {base}, rolled {}", roll.fmt());
                if chariot && first_turn { dmg *= 2; detail = format!("{detail}, x2 Chariot"); }
                self.deal_damage(target, dmg);
                self.riposte_check(side, flags);
                self.log.push(format!("{name} uses Heavy Blow on {tgt} for {dmg} damage [{detail}]"));
            }
        }
    }

    fn riposte_check(&mut self, attacker: Side, flags: &TurnFlags) {
        let defender = attacker.opponent();
        if !flags.get(defender).riposte { return; }
        let def_atk = self.fighter(defender).stats.attack;
        let base = (def_atk * 30 / 100).max(1);
        let roll = self.roll_effect(base, 1);
        let counter = roll.value;
        self.deal_damage(attacker, counter);
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
                let atk = self.fighter(side).stats.attack;
                let raw_def = self.fighter(target).stats.defense;
                let def_applied = raw_def * 3 / 4;
                let base = (atk + val - def_applied).max(DAMAGE_FLOOR);
                let roll = self.roll_effect(base, DAMAGE_FLOOR);
                let dmg = roll.value;
                self.deal_damage(target, dmg);
                self.log.push(format!("{name} uses Backstab on {tgt} for {dmg} damage [{atk} ATK + {val} card - {raw_def} DEF x3/4 = {base}, rolled {}]", roll.fmt()));
            }
            Some(MinorSuit::Cups) => {
                let base = val * 2;
                let roll = self.roll_effect(base, HEAL_FLOOR);
                let f = self.fighter_mut(side);
                let heal = f.diminished_heal(roll.value);
                f.current_hp = (f.current_hp + heal).min(f.max_hp);
                self.log.push(format!("{name} uses Elixir — heals {heal} HP [{val} card x2 = {base}, rolled {}]", roll.fmt()));
            }
            Some(MinorSuit::Wands) => {
                self.log.push(format!("{name} uses Haste — all actions this turn!"));
                self.exhaust(side, CombatAction::Weapon);
                self.exhaust(side, CombatAction::Apparel);
                self.apply_weapon(side, flags);
                self.apply_apparel(CombatAction::Apparel, side, flags);
            }
            Some(MinorSuit::Pentacles) => {
                flags.get_mut(side).barrier = true;
                self.log.push(format!("{name} uses Barrier — immune to damage this turn!"));
            }
        }
    }

    fn deal_damage(&mut self, target: Side, dmg: i32) {
        let f = self.fighter_mut(target);
        f.current_hp -= dmg;
        f.last_damage_taken += dmg;
    }
}
