use crate::card::MajorArcana;

#[derive(Clone, Copy, Debug)]
pub struct FateEffect {
    pub reverse_initiative: bool,
    pub damage_bonus: i32,
    pub damage_multiplier_pct: i32,
    pub defense_multiplier_pct: i32,
    pub heal_per_turn: i32,
    pub damage_floor: Option<i32>,
    // New fields
    pub swap_items: bool,
    pub first_action_double: bool,
    pub use_min_rolls: bool,
    pub heal_multiplier_pct: i32,
    pub force_action_order: bool,
    pub reflect_damage_pct: i32,
    pub momentum_bonus: i32,
    pub use_max_rolls: bool,
    pub swap_atk_def: bool,
    pub execute_threshold_pct: i32,
    pub disable_healing: bool,
    pub start_with_barrier: bool,
    pub chaos_rolls: bool,
    pub second_strike_bonus_pct: i32,
    pub flat_stat_bonus: i32,
}

impl Default for FateEffect {
    fn default() -> Self {
        Self {
            reverse_initiative: false,
            damage_bonus: 0,
            damage_multiplier_pct: 100,
            defense_multiplier_pct: 100,
            heal_per_turn: 0,
            damage_floor: None,
            swap_items: false,
            first_action_double: false,
            use_min_rolls: false,
            heal_multiplier_pct: 100,
            force_action_order: false,
            reflect_damage_pct: 0,
            momentum_bonus: 0,
            use_max_rolls: false,
            swap_atk_def: false,
            execute_threshold_pct: 0,
            disable_healing: false,
            start_with_barrier: false,
            chaos_rolls: false,
            second_strike_bonus_pct: 100,
            flat_stat_bonus: 0,
        }
    }
}

pub fn resolve_fate(arcana: MajorArcana) -> FateEffect {
    let d = FateEffect::default();
    match arcana {
        MajorArcana::TheFool => FateEffect { swap_items: true, ..d },
        MajorArcana::TheMagician => FateEffect { first_action_double: true, ..d },
        MajorArcana::TheHighPriestess => FateEffect { use_min_rolls: true, ..d },
        MajorArcana::TheEmpress => FateEffect { heal_multiplier_pct: 200, ..d },
        MajorArcana::TheEmperor => FateEffect { reverse_initiative: true, ..d },
        MajorArcana::TheHierophant => FateEffect { force_action_order: true, ..d },
        MajorArcana::TheLovers => FateEffect { reflect_damage_pct: 50, ..d },
        MajorArcana::TheChariot => FateEffect { momentum_bonus: 3, ..d },
        MajorArcana::Strength => FateEffect { damage_floor: Some(5), ..d },
        MajorArcana::TheHermit => FateEffect { defense_multiplier_pct: 200, ..d },
        MajorArcana::WheelOfFortune => FateEffect { use_max_rolls: true, ..d },
        MajorArcana::Justice => FateEffect { defense_multiplier_pct: 50, ..d },
        MajorArcana::TheHangedMan => FateEffect { swap_atk_def: true, ..d },
        MajorArcana::Death => FateEffect { execute_threshold_pct: 25, ..d },
        MajorArcana::Temperance => FateEffect { heal_per_turn: 3, ..d },
        MajorArcana::TheDevil => FateEffect { disable_healing: true, ..d },
        MajorArcana::TheTower => FateEffect { damage_multiplier_pct: 150, ..d },
        MajorArcana::TheStar => FateEffect { start_with_barrier: true, ..d },
        MajorArcana::TheMoon => FateEffect { chaos_rolls: true, ..d },
        MajorArcana::TheSun => FateEffect { damage_bonus: 4, ..d },
        MajorArcana::Judgement => FateEffect { second_strike_bonus_pct: 150, ..d },
        MajorArcana::TheWorld => FateEffect { flat_stat_bonus: 3, ..d },
    }
}

pub fn fate_description(arcana: MajorArcana) -> &'static str {
    match arcana {
        MajorArcana::TheFool => "Chaos reigns \u{2014} fighters exchange items",
        MajorArcana::TheMagician => "First weapon strike each cycle deals double damage",
        MajorArcana::TheHighPriestess => "Certainty reigns \u{2014} all rolls use minimum values",
        MajorArcana::TheEmpress => "All healing effects are doubled",
        MajorArcana::TheEmperor => "The slow strike first",
        MajorArcana::TheHierophant => "Tradition binds \u{2014} actions must follow Weapon \u{2192} Apparel \u{2192} Item",
        MajorArcana::TheLovers => "Pain is shared \u{2014} 50% of damage dealt reflects to attacker",
        MajorArcana::TheChariot => "Momentum builds \u{2014} each consecutive weapon attack deals +3 more",
        MajorArcana::Strength => "No blow is gentle \u{2014} minimum damage raised to 5",
        MajorArcana::TheHermit => "All defenses doubled",
        MajorArcana::WheelOfFortune => "Fortune smiles \u{2014} all rolls hit maximum value",
        MajorArcana::Justice => "Armor crumbles \u{2014} all defenses halved",
        MajorArcana::TheHangedMan => "The world inverts \u{2014} ATK and DEF are swapped",
        MajorArcana::Death => "The reaper watches \u{2014} fighters below 25% HP take double damage",
        MajorArcana::Temperance => "Both fighters regenerate 3 HP per turn",
        MajorArcana::TheDevil => "No mercy \u{2014} all healing is disabled",
        MajorArcana::TheTower => "All attacks deal 150% damage",
        MajorArcana::TheStar => "Hope shields \u{2014} first hit against each fighter is absorbed",
        MajorArcana::TheMoon => "Madness reigns \u{2014} damage rolls swing from 0 to double",
        MajorArcana::TheSun => "Radiant fury \u{2014} all attacks deal +4 damage",
        MajorArcana::Judgement => "The slow hand strikes harder \u{2014} second actor deals 150% damage",
        MajorArcana::TheWorld => "All boundaries expand \u{2014} both fighters gain +3 to all stats",
    }
}
