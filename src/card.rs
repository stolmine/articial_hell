use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MinorSuit {
    Swords,
    Wands,
    Cups,
    Pentacles,
}

impl MinorSuit {
    pub const ALL: [MinorSuit; 4] = [
        MinorSuit::Swords,
        MinorSuit::Wands,
        MinorSuit::Cups,
        MinorSuit::Pentacles,
    ];

    pub fn stat_name(self) -> &'static str {
        match self {
            MinorSuit::Swords => "ATK",
            MinorSuit::Wands => "SPD",
            MinorSuit::Cups => "HP",
            MinorSuit::Pentacles => "DEF",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CourtRank {
    Page,
    Knight,
    Queen,
    King,
}

impl CourtRank {
    pub const ALL: [CourtRank; 4] = [
        CourtRank::Page,
        CourtRank::Knight,
        CourtRank::Queen,
        CourtRank::King,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MajorArcana {
    TheFool,
    TheMagician,
    TheHighPriestess,
    TheEmpress,
    TheEmperor,
    TheHierophant,
    TheLovers,
    TheChariot,
    Strength,
    TheHermit,
    WheelOfFortune,
    Justice,
    TheHangedMan,
    Death,
    Temperance,
    TheDevil,
    TheTower,
    TheStar,
    TheMoon,
    TheSun,
    Judgement,
    TheWorld,
}

impl MajorArcana {
    pub const ALL: [MajorArcana; 22] = [
        MajorArcana::TheFool,
        MajorArcana::TheMagician,
        MajorArcana::TheHighPriestess,
        MajorArcana::TheEmpress,
        MajorArcana::TheEmperor,
        MajorArcana::TheHierophant,
        MajorArcana::TheLovers,
        MajorArcana::TheChariot,
        MajorArcana::Strength,
        MajorArcana::TheHermit,
        MajorArcana::WheelOfFortune,
        MajorArcana::Justice,
        MajorArcana::TheHangedMan,
        MajorArcana::Death,
        MajorArcana::Temperance,
        MajorArcana::TheDevil,
        MajorArcana::TheTower,
        MajorArcana::TheStar,
        MajorArcana::TheMoon,
        MajorArcana::TheSun,
        MajorArcana::Judgement,
        MajorArcana::TheWorld,
    ];

    pub fn numeral(self) -> &'static str {
        match self {
            MajorArcana::TheFool => "0",
            MajorArcana::TheMagician => "I",
            MajorArcana::TheHighPriestess => "II",
            MajorArcana::TheEmpress => "III",
            MajorArcana::TheEmperor => "IV",
            MajorArcana::TheHierophant => "V",
            MajorArcana::TheLovers => "VI",
            MajorArcana::TheChariot => "VII",
            MajorArcana::Strength => "VIII",
            MajorArcana::TheHermit => "IX",
            MajorArcana::WheelOfFortune => "X",
            MajorArcana::Justice => "XI",
            MajorArcana::TheHangedMan => "XII",
            MajorArcana::Death => "XIII",
            MajorArcana::Temperance => "XIV",
            MajorArcana::TheDevil => "XV",
            MajorArcana::TheTower => "XVI",
            MajorArcana::TheStar => "XVII",
            MajorArcana::TheMoon => "XVIII",
            MajorArcana::TheSun => "XIX",
            MajorArcana::Judgement => "XX",
            MajorArcana::TheWorld => "XXI",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TarotCard {
    Court { suit: MinorSuit, rank: CourtRank },
    Numbered { suit: MinorSuit, value: u8 },
    Major(MajorArcana),
}

impl TarotCard {
    pub fn suit(&self) -> Option<MinorSuit> {
        match self {
            TarotCard::Court { suit, .. } | TarotCard::Numbered { suit, .. } => Some(*suit),
            TarotCard::Major(_) => None,
        }
    }

    pub fn court_rank(&self) -> Option<CourtRank> {
        match self {
            TarotCard::Court { rank, .. } => Some(*rank),
            _ => None,
        }
    }

    pub fn numbered_value(&self) -> Option<u8> {
        match self {
            TarotCard::Numbered { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub fn arcana(&self) -> Option<MajorArcana> {
        match self {
            TarotCard::Major(a) => Some(*a),
            _ => None,
        }
    }
}

impl fmt::Display for MinorSuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MinorSuit::Swords => "󰞇 Swords",
            MinorSuit::Wands => "󱡃 Wands",
            MinorSuit::Cups => "󰆫 Cups",
            MinorSuit::Pentacles => "󰣏 Pentacles",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for CourtRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CourtRank::Page => "Page",
            CourtRank::Knight => "Knight",
            CourtRank::Queen => "Queen",
            CourtRank::King => "King",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for MajorArcana {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            MajorArcana::TheFool => "The Fool",
            MajorArcana::TheMagician => "The Magician",
            MajorArcana::TheHighPriestess => "The High Priestess",
            MajorArcana::TheEmpress => "The Empress",
            MajorArcana::TheEmperor => "The Emperor",
            MajorArcana::TheHierophant => "The Hierophant",
            MajorArcana::TheLovers => "The Lovers",
            MajorArcana::TheChariot => "The Chariot",
            MajorArcana::Strength => "Strength",
            MajorArcana::TheHermit => "The Hermit",
            MajorArcana::WheelOfFortune => "Wheel of Fortune",
            MajorArcana::Justice => "Justice",
            MajorArcana::TheHangedMan => "The Hanged Man",
            MajorArcana::Death => "Death",
            MajorArcana::Temperance => "Temperance",
            MajorArcana::TheDevil => "The Devil",
            MajorArcana::TheTower => "The Tower",
            MajorArcana::TheStar => "The Star",
            MajorArcana::TheMoon => "The Moon",
            MajorArcana::TheSun => "The Sun",
            MajorArcana::Judgement => "Judgement",
            MajorArcana::TheWorld => "The World",
        };
        write!(f, "{} {name}", self.numeral())
    }
}

impl fmt::Display for TarotCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TarotCard::Court { suit, rank } => write!(f, "{rank} of {suit}"),
            TarotCard::Numbered { suit, value } => {
                let name = match value {
                    1 => "Ace".to_string(),
                    v => v.to_string(),
                };
                write!(f, "{name} of {suit}")
            }
            TarotCard::Major(a) => write!(f, "{a}"),
        }
    }
}
