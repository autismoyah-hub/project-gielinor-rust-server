//! Player-dependent NPC morph and operation validation from NpcInteractions.
use crate::variables::{Varbit, Variables};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Default)]
pub struct Definition {
    pub operations: [Option<String>; 5],
    pub varp: Option<u16>,
    pub varbit: Option<u16>,
    pub fallback: Option<u16>,
    pub morphs: Vec<Option<u16>>,
}
#[derive(Default)]
pub struct Catalog {
    pub types: BTreeMap<u16, Definition>,
    pub varps: BTreeSet<u16>,
    pub varbits: BTreeMap<u16, Varbit>,
}
impl Catalog {
    pub fn morph(&self, id: u16, vars: &Variables) -> Option<u16> {
        let npc = self.types.get(&id)?;
        if npc.morphs.is_empty() && npc.fallback.unwrap_or(0) == 0 {
            return None;
        }
        // Kotlin prioritizes a positive varp, even when its definition is missing.
        // Missing selectors use zero; they do not fall back to the varbit selector.
        let selector = if let Some(varp) = npc.varp.filter(|v| *v > 0) {
            self.varps.contains(&varp).then(|| vars.get(varp))
        } else if let Some(varbit) = npc.varbit.filter(|v| *v > 0) {
            self.varbits
                .get(&varbit)
                .and_then(|bit| vars.bit(*bit).ok())
        } else {
            None
        }
        .unwrap_or(0);
        let target = usize::try_from(selector)
            .ok()
            .and_then(|index| npc.morphs.get(index))
            .copied()
            .unwrap_or(npc.fallback)?;
        self.types.contains_key(&target).then_some(target)
    }
    pub fn has_operation(&self, id: u16, vars: &Variables, operation: u8) -> bool {
        let id = self.morph(id, vars).unwrap_or(id);
        self.types
            .get(&id)
            .and_then(|npc| {
                operation
                    .checked_sub(1)
                    .and_then(|op| npc.operations.get(op as usize))
            })
            .and_then(|text| text.as_ref())
            .is_some_and(|text| {
                !text.chars().all(kotlin_whitespace) && !text.eq_ignore_ascii_case("hidden")
            })
    }
}
// Kotlin Char.isWhitespace = Character.isWhitespace || Character.isSpaceChar.
fn kotlin_whitespace(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{d}' | '\u{1c}'..='\u{20}' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}')
}
