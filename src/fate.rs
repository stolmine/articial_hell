use crate::card::MajorArcana;

#[derive(Clone, Copy, Debug)]
pub struct FateEffect {
    pub reverse_initiative: bool,
    pub damage_bonus: i32,
    pub damage_multiplier_pct: i32,
    pub defense_multiplier_pct: i32,
    pub heal_per_turn: i32,
    pub damage_floor: Option<i32>,
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
        }
    }
}

pub fn resolve_fate(arcana: MajorArcana) -> FateEffect {
    match arcana {
        MajorArcana::TheEmperor => FateEffect {
            reverse_initiative: true,
            ..FateEffect::default()
        },
        MajorArcana::TheHermit => FateEffect {
            defense_multiplier_pct: 200,
            ..FateEffect::default()
        },
        MajorArcana::Temperance => FateEffect {
            heal_per_turn: 3,
            ..FateEffect::default()
        },
        MajorArcana::TheTower => FateEffect {
            damage_multiplier_pct: 150,
            ..FateEffect::default()
        },
        _ => FateEffect::default(),
    }
}

pub fn fate_description(arcana: MajorArcana) -> &'static str {
    match arcana {
        MajorArcana::TheEmperor => "The slow strike first",
        MajorArcana::TheHermit => "All defenses doubled",
        MajorArcana::Temperance => "Both fighters regenerate 3 HP per turn",
        MajorArcana::TheTower => "All attacks deal 150% damage",
        _ => "Fate is silent.",
    }
}
