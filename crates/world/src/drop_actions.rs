//! HeldDropOp's confirmation stages. Pending operations retain the original item
//! and slot across dialogue, while an ordinary drop reads the slot again on commit.
use crate::inventory::{Catalog, Delete, Inventory, Item};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Drop,
    Destroy,
    Release,
}
#[derive(Clone, Debug)]
pub struct Rule {
    pub kind: Kind,
    pub name: String,
    pub market_price: i32,
    pub destroy_title: String,
    pub destroy_description: String,
    pub release_title: Option<String>,
    pub release_message: Option<String>,
}
pub struct Config {
    pub rules: BTreeMap<i32, Rule>,
    pub warning_enabled: crate::bank_actions::Binding,
    pub warning_threshold: crate::bank_actions::Binding,
    pub sound: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Ready,
    WarningBox,
    WarningChoice,
    DestroyChoice,
    ReleaseChoice,
    Cancelled,
}
#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub slot: usize,
    pub item: Item,
    pub kind: Kind,
    pub stage: Stage,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prompt {
    Object {
        id: i32,
        text: String,
    },
    Choice {
        title: String,
        yes: String,
        no: String,
    },
    Destroy {
        id: i32,
        count: i32,
        title: String,
        text: String,
    },
}
#[derive(Clone, Debug)]
pub enum Event {
    Prompt { token: u64, prompt: Prompt },
    Closed,
    Sound(u16),
    Message(String),
    Committed { item: Item, kind: Kind },
}
impl Pending {
    pub fn begin(
        slot: usize,
        item: Item,
        rule: &Rule,
        warning_enabled: bool,
        threshold: i32,
    ) -> Self {
        let stage = match rule.kind {
            Kind::Destroy => Stage::DestroyChoice,
            Kind::Release if item.count > 1 => Stage::ReleaseChoice,
            Kind::Drop
                if warning_enabled && rule.market_price.wrapping_mul(item.count) >= threshold =>
            {
                Stage::WarningBox
            }
            _ => Stage::Ready,
        };
        Self {
            slot,
            item,
            kind: rule.kind,
            stage,
        }
    }
    pub fn prompt(self, rule: &Rule) -> Option<Prompt> {
        match self.stage {
            Stage::WarningBox=>Some(Prompt::Object{id:self.item.id,text:"The item you are trying to put down is considered <col=7f0000>valuable</col>. Are you absolutely sure you want to do that?".into()}),
            Stage::WarningChoice=>Some(Prompt::Choice{title:format!("{}: Really put it down?",rule.name),yes:"Put it down.".into(),no:"No, don't put it down.".into()}),
            Stage::DestroyChoice=>Some(Prompt::Destroy{id:self.item.id,count:self.item.count,title:rule.destroy_title.clone(),text:rule.destroy_description.clone()}),
            Stage::ReleaseChoice=>Some(Prompt::Choice{title:rule.release_title.clone().unwrap_or_else(||format!("Drop all of your {}?",rule.name.to_lowercase())),yes:"Yes".into(),no:"No".into()}),
            _=>None,
        }
    }
    pub fn resume(&mut self, choice: i32) -> Result<(), &'static str> {
        self.stage = match (self.stage, choice) {
            (Stage::WarningBox, 0 | 1) => Stage::WarningChoice,
            (Stage::WarningChoice | Stage::ReleaseChoice, 1) => Stage::Ready,
            (Stage::WarningChoice | Stage::ReleaseChoice, 2) => Stage::Cancelled,
            (Stage::DestroyChoice, 1) => Stage::Ready,
            (Stage::DestroyChoice, 0) => Stage::Cancelled,
            _ => return Err("Invalid drop dialogue response"),
        };
        Ok(())
    }
    /// Player.invDel supplies strictSlot when a slot is specified: the captured
    /// type must still occupy that slot before any transaction can commit.
    pub fn delete(
        self,
        inventory: &mut Inventory,
        catalog: &Catalog,
    ) -> Result<i32, crate::inventory::Error> {
        let mut request = Delete::new(self.item.id, self.item.count);
        request.slot = self.slot;
        request.strict_slot = Some(self.slot);
        inventory
            .delete(catalog, request)
            .map(|result| result.completed)
    }
}
