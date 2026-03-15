use crate::card::MinorSuit;

pub fn suit_art(suit: MinorSuit) -> [&'static str; 5] {
    match suit {
        MinorSuit::Swords => [
            " /|\\ ",
            "/ | \\",
            "  |  ",
            " /|\\ ",
            "/ | \\",
        ],
        MinorSuit::Wands => [
            "  |  ",
            " )|(  ",
            " )|(  ",
            " )|(  ",
            "  |  ",
        ],
        MinorSuit::Cups => [
            " ___ ",
            "|   |",
            "|   |",
            " \\_/ ",
            "  |  ",
        ],
        MinorSuit::Pentacles => [
            "  *  ",
            " * * ",
            "*   *",
            " * * ",
            "  *  ",
        ],
    }
}

pub fn arcana_art() -> [&'static str; 5] {
    [
        " *** ",
        "*   *",
        "* o *",
        "*   *",
        " *** ",
    ]
}
