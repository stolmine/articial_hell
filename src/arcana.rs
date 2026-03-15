use crate::card::MajorArcana;
use crate::stats::Stats;

#[derive(Clone, Copy, Debug)]
pub struct ArcanaEffect {
    pub stat_bonus: Stats,
    pub always_first: bool,
    pub first_hit_double: bool,
    pub heal_per_turn: i32,
    pub death_curse: bool,
}

impl Default for ArcanaEffect {
    fn default() -> Self {
        Self {
            stat_bonus: Stats::default(),
            always_first: false,
            first_hit_double: false,
            heal_per_turn: 0,
            death_curse: false,
        }
    }
}

pub fn arcana_description(arcana: MajorArcana) -> &'static str {
    match arcana {
        MajorArcana::TheFool          => "+2 all stats",
        MajorArcana::TheMagician      => "+6 ATK",
        MajorArcana::TheHighPriestess => "+6 SPD",
        MajorArcana::TheEmpress       => "+5 all stats",
        MajorArcana::TheEmperor       => "Always acts first, +2 DEF",
        MajorArcana::TheHierophant    => "+3 per equipment card matching hero suit",
        MajorArcana::TheLovers        => "+8 all stats if exactly 2 equipment suits match, else +2",
        MajorArcana::TheChariot       => "First hit deals double damage, +2 ATK",
        MajorArcana::Strength         => "+8 ATK",
        MajorArcana::TheHermit        => "+8 DEF",
        MajorArcana::WheelOfFortune   => "+3 all stats",
        MajorArcana::Justice          => "+4 HP, +4 DEF",
        MajorArcana::TheHangedMan     => "+5 DEF, -2 ATK",
        MajorArcana::Death            => "If you lose, opponent carries 1 HP to next round",
        MajorArcana::Temperance       => "Heal 3 HP per turn",
        MajorArcana::TheDevil         => "+7 ATK, -3 DEF",
        MajorArcana::TheTower         => "+10 ATK, -5 HP",
        MajorArcana::TheStar          => "+10 HP",
        MajorArcana::TheMoon          => "+4 SPD, +4 DEF",
        MajorArcana::TheSun           => "+5 ATK, +5 SPD",
        MajorArcana::Judgement        => "+6 HP, +3 DEF",
        MajorArcana::TheWorld         => "+2 all stats per unique suit among equipment",
    }
}

pub fn resolve_arcana(
    arcana: MajorArcana,
    hero_suit: Option<crate::card::MinorSuit>,
    equipment: &[crate::card::TarotCard; 3],
) -> ArcanaEffect {
    let mut effect = ArcanaEffect::default();
    let s = &mut effect.stat_bonus;

    match arcana {
        MajorArcana::TheFool          => s.add_flat(2),
        MajorArcana::TheMagician      => s.attack += 6,
        MajorArcana::TheHighPriestess => s.speed += 6,
        MajorArcana::TheEmpress       => s.add_flat(5),
        MajorArcana::TheEmperor       => { effect.always_first = true; effect.stat_bonus.defense += 2; }
        MajorArcana::TheHierophant    => {
            let count = hero_suit.map(|suit| equipment.iter().filter(|c| c.suit() == Some(suit)).count()).unwrap_or(0) as i32;
            s.add_flat(count * 3);
        }
        MajorArcana::TheLovers        => {
            let matches = hero_suit.map(|suit| equipment.iter().filter(|c| c.suit() == Some(suit)).count()).unwrap_or(0);
            s.add_flat(if matches == 2 { 8 } else { 2 });
        }
        MajorArcana::TheChariot       => { effect.first_hit_double = true; effect.stat_bonus.attack += 2; }
        MajorArcana::Strength         => s.attack += 8,
        MajorArcana::TheHermit        => s.defense += 8,
        MajorArcana::WheelOfFortune   => s.add_flat(3),
        MajorArcana::Justice          => { s.hp += 4; s.defense += 4; }
        MajorArcana::TheHangedMan     => { s.defense += 5; s.attack -= 2; }
        MajorArcana::Death            => effect.death_curse = true,
        MajorArcana::Temperance       => effect.heal_per_turn = 3,
        MajorArcana::TheDevil         => { s.attack += 7; s.defense -= 3; }
        MajorArcana::TheTower         => { s.attack += 10; s.hp -= 5; }
        MajorArcana::TheStar          => s.hp += 10,
        MajorArcana::TheMoon          => { s.speed += 4; s.defense += 4; }
        MajorArcana::TheSun           => { s.attack += 5; s.speed += 5; }
        MajorArcana::Judgement        => { s.hp += 6; s.defense += 3; }
        MajorArcana::TheWorld         => {
            s.add_flat(crate::stats::count_unique_suits(equipment) as i32 * 2);
        }
    }

    effect
}
