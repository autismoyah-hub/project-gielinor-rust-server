//! Shared observations and atomic reward commit for ordinary gathering.
use crate::locations::Location;
pub enum Activity {
    Woodcutting,
    Mining,
}
#[derive(Clone, Debug)]
pub struct Trace {
    pub tick: i32,
    pub location: Location,
    pub revision: u64,
    pub action_delay: i32,
    pub result: &'static str,
    pub roll: Option<u32>,
    pub threshold: Option<f64>,
    pub product: i32,
    pub product_delta: i32,
    pub fine_xp_delta: i32,
    pub level: i32,
    pub required_level: i32,
    pub tool: Option<i32>,
    pub free_slots: usize,
    pub product_before: i64,
    pub fine_xp_before: i32,
    pub base_xp: f64,
    pub xp_rate_hundreds: i32,
    pub depletion_roll: Option<u32>,
    pub active_ticks: Option<i32>,
    pub respawn_delay: Option<i32>,
}
impl Trace {
    /// Numeric and closed-vocabulary gameplay data only; no account/session secrets.
    pub fn json(&self, actor: crate::simulation::ActorId, activity: Activity) -> String {
        let activity = match activity {
            Activity::Woodcutting => "woodcutting",
            Activity::Mining => "mining",
        };
        let optional = |value: Option<String>| value.unwrap_or_else(|| "null".into());
        format!(
            concat!(
            "{{\"schema\":1,\"activity\":\"{}\",\"actor_slot\":{},\"actor_generation\":{},\"tick\":{},",
                "\"location\":[{},{},{},{},{},{}],\"revision\":{},\"scheduled_tick\":{},",
                "\"result\":\"{}\",\"roll\":{},\"threshold\":{},\"product\":{},",
                "\"product_delta\":{},\"fine_xp_delta\":{},\"level\":{},\"required_level\":{},",
                "\"tool\":{},\"free_slots\":{},\"product_before\":{},\"fine_xp_before\":{},",
                "\"base_xp\":{},\"xp_rate_hundreds\":{},\"depletion_roll\":{},\"active_ticks\":{},\"respawn_delay\":{},\"consumed_delta\":0}}"
            ),
            activity,
            actor.slot,
            actor.generation,
            self.tick,
            self.location.coords.x(),
            self.location.coords.z(),
            self.location.coords.level(),
            self.location.id,
            self.location.shape,
            self.location.angle,
            self.revision,
            self.action_delay,
            self.result,
            optional(self.roll.map(|v| v.to_string())),
            optional(self.threshold.map(|v| v.to_string())),
            self.product,
            self.product_delta,
            self.fine_xp_delta,
            self.level,
            self.required_level,
            optional(self.tool.map(|v| v.to_string())),
            self.free_slots,
            self.product_before,
            self.fine_xp_before,
            self.base_xp,
            self.xp_rate_hundreds,
            optional(self.depletion_roll.map(|v|v.to_string())),
            optional(self.active_ticks.map(|v|v.to_string())),
            optional(self.respawn_delay.map(|v|v.to_string()))
        )
    }
}

pub(crate) struct Reward {
    inventory: crate::inventory::Inventory,
    stats: crate::stats::Stats,
    advancement: crate::skills::Advancement,
    delta: i32,
}
impl Reward {
    pub(crate) fn commit(
        self,
        data: &mut crate::simulation::PlayerData,
    ) -> (crate::skills::Advancement, i32) {
        data.inventory = self.inventory;
        data.stats = self.stats;
        (self.advancement, self.delta)
    }
}

/// Prepare both mutations; the caller commits after any location replacement.
pub(crate) fn reward(
    data: &crate::simulation::PlayerData,
    catalog: &crate::inventory::Catalog,
    skill: u8,
    product: i32,
    xp: f64,
) -> Result<Reward, crate::simulation::ActionError> {
    let mut inventory = data.inventory.clone();
    let mut stats = data.stats.clone();
    let before = stats.fine_xp(skill);
    inventory
        .insert(catalog, crate::inventory::Insert::new(product, 1))
        .map_err(|_| crate::simulation::ActionError::InvalidItem)?;
    let advancement = crate::skills::advance(
        &mut stats,
        skill,
        xp,
        data.xp_rate_hundreds as f64 / 100.0,
        99,
    )
    .map_err(|_| crate::simulation::ActionError::InvalidItem)?;
    let delta = stats.fine_xp(skill) - before;
    Ok(Reward {
        inventory,
        stats,
        advancement,
        delta,
    })
}

/// Highest eligible requirement; first carried tie, then equipped tie wins.
pub(crate) fn best_tool(
    inv: &crate::inventory::Inventory,
    worn: &crate::inventory::Inventory,
    level: i32,
    requirement: impl Fn(i32) -> Option<i32>,
) -> Option<i32> {
    best_tool_by(inv, worn, level, |id| requirement(id).map(|req| (req, req)))
}
/// Eligibility and quality are separate: equal-level tools may have different rates.
pub(crate) fn best_tool_by(
    inv: &crate::inventory::Inventory,
    worn: &crate::inventory::Inventory,
    level: i32,
    candidate: impl Fn(i32) -> Option<(i32, i32)>,
) -> Option<i32> {
    let mut best: Option<(i32, i32)> = None;
    for item in inv.slots().iter().flatten() {
        if let Some((_, quality)) = candidate(item.id).filter(|&(req, _)| req <= level) {
            if best.is_none_or(|(_, current)| quality > current) {
                best = Some((item.id, quality));
            }
        }
    }
    if let Some(item) = worn.get(3) {
        if let Some((_, quality)) = candidate(item.id).filter(|&(req, _)| req <= level) {
            if best.is_none_or(|(_, current)| quality >= current) {
                best = Some((item.id, quality));
            }
        }
    }
    best.map(|(id, _)| id)
}

/// Apply an optional worn modifier before the existing single reward transaction.
pub(crate) fn reward_with_outfit(
    data:&crate::simulation::PlayerData,catalog:&crate::inventory::Catalog,
    skill:u8,product:i32,xp:f64,member:bool,
    outfit:Option<&crate::gathering_outfits::Outfit>,
)->Result<Reward,crate::simulation::ActionError>{
    let bonus=outfit.map(|o|o.bonus_basis_points(&data.equipment,catalog,member)).transpose()
        .map_err(|_|crate::simulation::ActionError::InvalidItem)?.unwrap_or(0);
    reward(data,catalog,skill,product,xp*(1.0+f64::from(bonus)/10000.0))
}
