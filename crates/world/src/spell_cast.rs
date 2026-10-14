//! MagicRuneManager's access, validation and consumption boundary.
//! Successful results authorize spell effects; this layer does not award XP.
use crate::{
    autocast::Spell,
    inventory::{Catalog, Inventory},
    magic,
    rune_consumption::{self, ResultKind},
    runes::Validation,
    variables::{Update, Variables},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultKindCast {
    IncorrectSpellbook,
    MissingLevel,
    MissingRequirements,
    Consumption(ResultKind),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub kind: ResultKindCast,
    pub message: Option<String>,
    pub updates: Vec<Update>,
}
impl Outcome {
    pub fn success(&self) -> bool {
        matches!(
            self.kind,
            ResultKindCast::Consumption(ResultKind::AllUnlimited | ResultKind::Consumed { .. })
        )
    }
    pub fn used_sunfire(&self) -> bool {
        matches!(
            self.kind,
            ResultKindCast::Consumption(ResultKind::Consumed { sunfire: true })
        )
    }
}
#[derive(Clone, Copy)]
pub struct Request<'a> {
    pub spell: &'a Spell,
    /// None represents a spell with no configured spellbook; it cannot match.
    pub required_book: Option<i32>,
    pub active_book: i32,
    pub required_level: i32,
    pub visible_level: i32,
    pub sunfire_item: i32,
    pub logged: bool,
}
/// Trusted spell metadata, resolved from the cache and existing content hooks.
/// Elemental handlers may substitute their Kotlin/Kronos level and rune costs.
#[derive(Clone)]
pub struct Definition {
    pub spell: Spell,
    pub book: Option<i32>,
    pub level: i32,
    pub cast_xp: f64,
}
/// MagicRuneManager.canCastSpell, used before combat eligibility/cooldown.
/// Returns the first failure without consuming items, changing vars or XP.
pub fn check(
    config: &magic::Config,
    request: Request<'_>,
    inventory: &Inventory,
    worn: &Inventory,
    vars: &Variables,
) -> Result<Option<Outcome>, String> {
    let failure = |kind, message| Outcome {
        kind,
        message,
        updates: vec![],
    };
    if request.required_book != Some(request.active_book) {
        return Ok(Some(failure(ResultKindCast::IncorrectSpellbook, None)));
    }
    if request.visible_level < request.required_level {
        return Ok(Some(failure(
            ResultKindCast::MissingLevel,
            Some("Your Magic level is not high enough for this spell.".into()),
        )));
    }
    let validations = config.validate(vars, inventory, worn, request.spell)?;
    if let Some(invalid) = validations.iter().find(|v| !v.valid()) {
        let message = match invalid {
            Validation::NotEnough(id) => config.names.missing.get(id),
            Validation::NotWearing(id) => config.names.not_wearing.get(id),
            _ => None,
        }
        .cloned()
        .unwrap_or_else(|| "You do not have enough runes to cast this spell.".into());
        return Ok(Some(failure(
            ResultKindCast::MissingRequirements,
            Some(message),
        )));
    }
    Ok(None)
}
pub fn attempt(
    config: &magic::Config,
    request: Request<'_>,
    inventory: &mut Inventory,
    worn: &Inventory,
    vars: &mut Variables,
    items: &Catalog,
) -> Result<Outcome, String> {
    if let Some(failure) = check(config, request, inventory, worn, vars)? {
        return Ok(failure);
    }
    let validations = config.validate(vars, inventory, worn, request.spell)?;
    let bits = config.pouch_counts.into_iter().collect();
    let result = rune_consumption::consume(
        inventory,
        vars,
        items,
        &bits,
        &validations,
        request.sunfire_item,
        request.logged,
    )?;
    Ok(Outcome {
        kind: ResultKindCast::Consumption(result.kind),
        message: None,
        updates: result.updates,
    })
}
