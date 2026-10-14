//! Original RSProt revision-239 outgoing categories. FIFO is retained within each category.
//! Reserved UNKNOWN_STRING has no encoder in the pinned artifact and is unsupported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    High,
    Low,
}
pub fn category(opcode: u16) -> Option<Category> {
    match opcode {
        2 | 5 | 9 | 10 | 11 | 12 | 13 | 18 | 21 | 22 | 28 | 32 | 33 | 34 | 41 | 44 | 45 | 47
        | 49 | 52 | 54 | 57 | 58 | 60 | 62 | 66 | 67 | 69 | 70 | 71 | 73 | 74 | 75 | 76 | 78
        | 79 | 81 | 85 | 86 | 88 | 91 | 93 | 95 | 97 | 104 | 105 | 109 | 110 | 116 | 122 | 124
        | 125 | 127 | 143 => Some(Category::High),
        0 | 1 | 3 | 4 | 6 | 7 | 8 | 14 | 15 | 16 | 17 | 19 | 20 | 23 | 25 | 26 | 27 | 29 | 30
        | 31 | 35 | 36 | 37 | 38 | 39 | 40 | 42 | 43 | 46 | 48 | 50 | 51 | 53 | 55 | 56 | 59
        | 61 | 63 | 64 | 65 | 68 | 72 | 77 | 80 | 82 | 83 | 84 | 87 | 89 | 90 | 92 | 94 | 96
        | 98 | 99 | 100 | 101 | 102 | 103 | 106 | 107 | 108 | 111 | 112 | 113 | 114 | 115 | 117
        | 118 | 119 | 120 | 121 | 123 | 126 | 128 | 129 | 130 | 131 | 132 | 133 | 134 | 135
        | 136 | 137 | 138 | 139 | 140 | 141 | 142 | 144 | 145 | 146 | 147 | 148 => {
            Some(Category::Low)
        }
        _ => None,
    }
}
