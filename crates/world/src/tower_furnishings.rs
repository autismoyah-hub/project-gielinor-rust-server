//! Deliberately authored Cinderworks furnishings; no prototype room filler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Furnishing {
    pub x: u8,
    pub z: u8,
    pub symbol: &'static str,
}
pub const FURNISHINGS: &[Furnishing] = &[
    Furnishing {
        x: 38,
        z: 52,
        symbol: "table",
    },
    Furnishing {
        x: 39,
        z: 51,
        symbol: "stool",
    },
    Furnishing {
        x: 38,
        z: 51,
        symbol: "chair",
    },
    Furnishing {
        x: 33,
        z: 62,
        symbol: "bookcase",
    },
    Furnishing {
        x: 35,
        z: 62,
        symbol: "bookcase",
    },
    Furnishing {
        x: 50,
        z: 61,
        symbol: "bookcase",
    },
    Furnishing {
        x: 51,
        z: 61,
        symbol: "bookcase",
    },
    Furnishing {
        x: 62,
        z: 61,
        symbol: "bookcase",
    },
    Furnishing {
        x: 49,
        z: 60,
        symbol: "barrel",
    },
    Furnishing {
        x: 49,
        z: 58,
        symbol: "sacks",
    },
    Furnishing {
        x: 62,
        z: 58,
        symbol: "barrel",
    },
    Furnishing {
        x: 52,
        z: 51,
        symbol: "candles",
    },
    Furnishing {
        x: 61,
        z: 51,
        symbol: "candles",
    },
    Furnishing {
        x: 49,
        z: 62,
        symbol: "torch",
    },
    Furnishing {
        x: 62,
        z: 62,
        symbol: "torch",
    },
    Furnishing {
        x: 33,
        z: 67,
        symbol: "bush1",
    },
    Furnishing {
        x: 46,
        z: 67,
        symbol: "bush1",
    },
    Furnishing {
        x: 33,
        z: 77,
        symbol: "bush1",
    },
    Furnishing {
        x: 46,
        z: 77,
        symbol: "bush1",
    },
    Furnishing {
        x: 38,
        z: 66,
        symbol: "evergreen",
    },
    Furnishing {
        x: 44,
        z: 74,
        symbol: "fern",
    },
    Furnishing {
        x: 36,
        z: 72,
        symbol: "fern",
    },
    Furnishing {
        x: 38,
        z: 74,
        symbol: "deadtree1_stump",
    },
    Furnishing {
        x: 45,
        z: 71,
        symbol: "bush1",
    },
    Furnishing {
        x: 42,
        z: 51,
        symbol: "cart",
    },
    Furnishing {
        x: 36,
        z: 51,
        symbol: "barrel",
    },
    Furnishing {
        x: 43,
        z: 51,
        symbol: "sacks",
    },
    Furnishing {
        x: 38,
        z: 61,
        symbol: "statue_dwarfminer",
    },
    Furnishing {
        x: 33,
        z: 56,
        symbol: "torch",
    },
    Furnishing {
        x: 46,
        z: 59,
        symbol: "torch",
    },
    Furnishing {
        x: 35,
        z: 61,
        symbol: "tinrock1",
    },
    Furnishing {
        x: 45,
        z: 61,
        symbol: "copperrock1",
    },
    Furnishing {
        x: 66,
        z: 58,
        symbol: "barrel_fish",
    },
    Furnishing {
        x: 68,
        z: 58,
        symbol: "barrel_fish",
    },
    Furnishing {
        x: 66,
        z: 50,
        symbol: "sacks",
    },
    Furnishing {
        x: 68,
        z: 50,
        symbol: "barrel",
    },
    Furnishing {
        x: 65,
        z: 54,
        symbol: "barrel",
    },
    Furnishing {
        x: 40,
        z: 35,
        symbol: "hay_bale",
    },
    Furnishing {
        x: 35,
        z: 40,
        symbol: "hay_bale",
    },
    Furnishing {
        x: 45,
        z: 40,
        symbol: "hay_bale",
    },
    Furnishing {
        x: 42,
        z: 45,
        symbol: "cart",
    },
    Furnishing {
        x: 33,
        z: 35,
        symbol: "bush1",
    },
    Furnishing {
        x: 46,
        z: 35,
        symbol: "bush1",
    },
    Furnishing {
        x: 33,
        z: 46,
        symbol: "bush1",
    },
    Furnishing {
        x: 46,
        z: 46,
        symbol: "bush1",
    },
    Furnishing {
        x: 68,
        z: 35,
        symbol: "hay_bale",
    },
    Furnishing {
        x: 75,
        z: 35,
        symbol: "hay_bale",
    },
    Furnishing {
        x: 68,
        z: 45,
        symbol: "bush1",
    },
    Furnishing {
        x: 75,
        z: 45,
        symbol: "bush1",
    },
    Furnishing {
        x: 72,
        z: 35,
        symbol: "sacks",
    },
    Furnishing {
        x: 65,
        z: 66,
        symbol: "fire",
    },
    Furnishing {
        x: 78,
        z: 66,
        symbol: "fire",
    },
    Furnishing {
        x: 51,
        z: 45,
        symbol: "enakh_brazier_coal_lit",
    },
    Furnishing {
        x: 61,
        z: 45,
        symbol: "enakh_brazier_coal_lit",
    },
    Furnishing {
        x: 51,
        z: 35,
        symbol: "sacks",
    },
    Furnishing {
        x: 61,
        z: 35,
        symbol: "barrel",
    },
    Furnishing {
        x: 65,
        z: 76,
        symbol: "fire",
    },
    Furnishing {
        x: 78,
        z: 76,
        symbol: "fire",
    },
    Furnishing {
        x: 52,
        z: 49,
        symbol: "enakh_brazier_coal_lit",
    },
    Furnishing {
        x: 52,
        z: 65,
        symbol: "enakh_brazier_coal_lit",
    },
    Furnishing {
        x: 59,
        z: 65,
        symbol: "enakh_brazier_coal_lit",
    },
];
