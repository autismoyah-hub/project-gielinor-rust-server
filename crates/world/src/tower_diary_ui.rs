//! Existing FloorOneRules journal text, derived from authoritative diary marks.
//! The host must supply persisted marks and current base skill levels.
use crate::tower_diary::{Marks, RowState};

const DONE: &str = "0dc10d";
const NOW: &str = "ffb83f";
const AHEAD: &str = "9f9f9f";
const NAMES: [&str; 11] = [
    "Hides",
    "Bones",
    "Leather vambraces",
    "Bronze scimitar",
    "Grove bows",
    "Shrimp",
    "Mind runes",
    "Attack potion",
    "Bronze arrows",
    "Ash Imp",
    "Ash Warden",
];
fn current(m: Marks, row: usize) -> String {
    let [tans, offers, bows, shrimp, minds, arrows] = m.counts;
    let [crafting, smithing, firemaking] = m.levels;
    let [vambrace, scimitar, cast, guam, newt, potion, hit] = m.actions.map(i32::from);
    match row {
        1 => format!("Hides {tans}/5"),
        2 => format!("Bones {offers}/5"),
        3 if crafting < 11 => format!("Crafting {crafting}/11"),
        3 => format!("Vambraces {vambrace}/1"),
        4 if smithing < 5 => format!("Smithing {smithing}/5"),
        4 => format!("Scimitar {scimitar}/1"),
        5 => format!("Bows {bows}/3 Firemaking {firemaking}/10"),
        6 => format!("Shrimp {shrimp}/5"),
        7 => {
            let core = if minds < 5 {
                format!("Minds {minds}/5")
            } else {
                format!("Cast {cast}/1")
            };
            if m.staff_taken {
                core
            } else {
                format!("{core}. Search the staff crate")
            }
        }
        8 if guam == 0 || newt == 0 => format!("Guam {guam}/1 Newt {newt}/1"),
        8 => format!("Potion {potion}/1"),
        9 if arrows < 15 => format!("Arrows {arrows}/15"),
        9 => format!("Hit {hit}/1"),
        10 => format!("Imp {}/1", i32::from(m.done[9] || m.pending[0])),
        11 => format!("Warden {}/1", i32::from(m.done[10] || m.pending[1])),
        _ => unreachable!("Only the eleven diary rows are rendered"),
    }
}
fn rows(m: Marks) -> Vec<(RowState, String)> {
    let m = m.sync();
    (1..=11)
        .map(|row| {
            let state = m.state(row as i32);
            let detail = if matches!(state, RowState::Now | RowState::Owed) {
                current(m, row)
            } else {
                NAMES[row - 1].to_owned()
            };
            (state, format!("{row}. {detail}"))
        })
        .collect()
}
fn label(state: RowState) -> &'static str {
    match state {
        RowState::Done => "Done",
        RowState::Owed => "Owed",
        RowState::Now => "Now",
        RowState::Ahead => "Ahead",
    }
}
pub fn title(m: Marks) -> String {
    let m = m.sync();
    match (1..=11).find(|&row| !m.is_done(row)) {
        None => "Threshold — 11 of 11 done".into(),
        Some(row) if m.bounty == row => {
            format!("Threshold — row {row} of 11 — bounty at the chest")
        }
        Some(row) => format!("Threshold — row {row} of 11"),
    }
}
pub fn status(m: Marks) -> Vec<String> {
    rows(m)
        .into_iter()
        .map(|(state, text)| format!("{} {text}", label(state)))
        .collect()
}
pub fn lines(m: Marks) -> String {
    rows(m)
        .into_iter()
        .map(|(state, text)| {
            let colour = match state {
                RowState::Done => DONE,
                RowState::Now | RowState::Owed => NOW,
                RowState::Ahead => AHEAD,
            };
            format!("<col={colour}>{} {text}</col>", label(state))
        })
        .collect::<Vec<_>>()
        .join("<br>")
}
pub fn body(m: Marks) -> String {
    let colour = if m.sync().done.iter().all(|&d| d) {
        DONE
    } else {
        NOW
    };
    format!("<col={colour}>{}</col><br>{}", title(m), lines(m))
}
