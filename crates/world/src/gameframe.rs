//! GameframeScript selection and intermediate layout rules.
use crate::{
    bank_actions::Binding,
    ui::{Component, State},
    variables::{Update, Variables},
};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct Frame {
    pub top: u16,
    pub overlays: Vec<(Component, u16)>,
    pub mappings: BTreeMap<Component, Component>,
    pub client_mode: i32,
    pub resizable: bool,
    pub default: bool,
    pub stones: bool,
}
pub struct Frames {
    pub ordered: Vec<Frame>,
    pub top: Binding,
    pub stones: Binding,
}
pub struct Login {
    pub frame: Frame,
    pub variables: Vec<Update>,
}
#[derive(Clone, Debug)]
pub struct Movement {
    pub from: Frame,
    pub into: Frame,
    pub intermediate: Option<Frame>,
}
pub struct QueuedMove {
    pub movement: Movement,
    pub delay: i32,
    pub client_mode: Option<i32>,
    pub variables: Vec<Update>,
}
impl Frames {
    pub fn prepare_move(
        &self,
        top: u16,
        vars: &mut Variables,
        ui: &mut State,
        logged_in: bool,
    ) -> Result<QueuedMove, String> {
        let into = self
            .ordered
            .iter()
            .find(|f| f.top == top)
            .ok_or("Unknown destination gameframe")?;
        let previous = self.top.get(vars).map_err(|e| format!("{e:?}"))?;
        let from = self
            .ordered
            .iter()
            .find(|f| f.top as i32 == previous)
            .ok_or("Unknown previous gameframe")?;
        let stones = self.stones.get(vars).map_err(|e| format!("{e:?}"))? == 1;
        let intermediate = self.intermediate(from, into, stones)?.cloned();
        let mode_change = ui.frame_resizable != into.resizable;
        let update = vars
            .set_bit(self.top.base, self.top.bit, into.top as i32, logged_in)
            .map_err(|e| format!("{e:?}"))?;
        ui.frame_resizable = into.resizable;
        // Actual toplevel and mappings remain unchanged until the soft queue fires.
        Ok(QueuedMove {
            movement: Movement {
                from: from.clone(),
                into: into.clone(),
                intermediate,
            },
            delay: if mode_change { 2 } else { 1 },
            client_mode: mode_change.then_some(into.client_mode),
            variables: update.into_iter().collect(),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.ordered.iter().filter(|f| f.default).count() != 1 {
            return Err("Expected exactly one default gameframe".into());
        }
        let mut seen = std::collections::HashSet::new();
        if self.ordered.iter().any(|f| !seen.insert(f.top)) {
            return Err("Duplicate gameframe toplevel".into());
        }
        Ok(())
    }
    pub fn login(
        &self,
        vars: &mut Variables,
        ui: &mut State,
        logged_in: bool,
    ) -> Result<Login, String> {
        self.validate()?;
        let saved = self.top.get(vars).map_err(|e| format!("{e:?}"))?;
        let stones = self.stones.get(vars).map_err(|e| format!("{e:?}"))? != 0;
        let existing = self
            .ordered
            .iter()
            .find(|f| f.top as i32 == saved && f.resizable == ui.frame_resizable);
        let mut updates = Vec::new();
        let frame = if let Some(frame) = existing {
            frame
        } else {
            let frame = self
                .ordered
                .iter()
                .find(|f| f.resizable == ui.frame_resizable && f.stones == stones)
                .or_else(|| {
                    self.ordered
                        .iter()
                        .find(|f| f.resizable == ui.frame_resizable)
                })
                .or_else(|| self.ordered.iter().find(|f| f.default))
                .unwrap();
            ui.frame_resizable = frame.resizable;
            for (binding, value) in [
                (self.top, frame.top as i32),
                (self.stones, frame.stones as i32),
            ] {
                if let Some(update) = vars
                    .set_bit(binding.base, binding.bit, value, logged_in)
                    .map_err(|e| format!("{e:?}"))?
                {
                    updates.push(update);
                }
            }
            frame
        };
        ui.top_level = Some(frame.top);
        ui.gameframe = frame.mappings.clone();
        for &(parent, interface) in &frame.overlays {
            ui.open_sub(parent, interface, true)
                .map_err(str::to_owned)?;
        }
        Ok(Login {
            frame: frame.clone(),
            variables: updates,
        })
    }
    pub fn intermediate(
        &self,
        from: &Frame,
        into: &Frame,
        stones: bool,
    ) -> Result<Option<&Frame>, String> {
        if !into.resizable || from.resizable || stones == into.stones {
            return Ok(None);
        }
        self.ordered
            .iter()
            .find(|f| f.resizable && f.stones == stones)
            .map(Some)
            .ok_or_else(|| "Missing intermediate gameframe".into())
    }
}
