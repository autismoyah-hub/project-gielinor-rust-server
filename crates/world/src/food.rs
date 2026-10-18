//! EdibleFoods and Eating.bite, including this server's cake/pizza consumption behavior.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    stat_ops::{self, Operation, Update},
    stats::Stats,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Food {
    pub symbol: &'static str,
    pub heal: i32,
    pub delay: i32,
    pub replacement: Option<&'static str>,
    pub message: Option<String>,
}
pub fn catalog() -> Vec<Food> {
    let bite = |symbol, heal, delay, replacement, message| Food {
        symbol,
        heal,
        delay,
        replacement,
        message,
    };
    let mut foods = vec![
        bite(
            "potato",
            1,
            3,
            None,
            Some("You eat the potato. Yuck!".into()),
        ),
        bite(
            "cabbage",
            1,
            3,
            None,
            Some("You eat the cabbage. Yuck!".into()),
        ),
    ];
    for (symbol, heal) in [
        ("banana", 2),
        ("king_worm", 2),
        ("peach", 8),
        ("ugthanki_kebab_bad", 19),
        ("bread", 5),
    ] {
        foods.push(bite(symbol, heal, 3, None, None));
    }
    for (whole, part, slice, heal, name) in [
        ("cake", "partial_cake", "cake_slice", 4, "cake"),
        (
            "chocolate_cake",
            "partial_chocolate_cake",
            "chocolate_slice",
            5,
            "chocolate cake",
        ),
    ] {
        // Kotlin's helpers supply chat/delays, but no replacement item for these entries.
        foods.push(bite(
            whole,
            heal,
            2,
            None,
            Some(format!("You eat part of the {name}.")),
        ));
        foods.push(bite(
            part,
            heal,
            2,
            None,
            Some(format!("You eat some more of the {name}.")),
        ));
        foods.push(bite(
            slice,
            heal,
            3,
            None,
            Some(format!("You eat the slice of {name}.")),
        ));
    }
    for (whole, half, heal, name) in [
        ("plain_pizza", "half_plain_pizza", 7, "pizza"),
        ("meat_pizza", "half_meat_pizza", 8, "meat pizza"),
        ("anchovie_pizza", "half_anchovie_pizza", 9, "anchovy pizza"),
        (
            "pineapple_pizza",
            "half_pineapple_pizza",
            11,
            "pineapple pizza",
        ),
        ("redberry_pie", "half_a_redberry_pie", 5, "redberry pie"),
        ("meat_pie", "half_a_meat_pie", 6, "meat pie"),
        ("apple_pie", "half_an_apple_pie", 7, "apple pie"),
    ] {
        foods.push(bite(
            whole,
            heal,
            1,
            None,
            Some(format!("You eat half of the {name}.")),
        ));
        foods.push(bite(
            half,
            heal,
            2,
            None,
            Some(format!("You eat the remaining {name}.")),
        ));
    }
    foods.push(bite(
        "bowl_mushroom_fried",
        5,
        3,
        Some("bowl_empty"),
        Some("You eat the fried mushrooms.".into()),
    ));
    foods.push(bite(
        "curry",
        19,
        3,
        Some("bowl_empty"),
        Some("You eat the curry.".into()),
    ));
    for (symbol, heal) in [
        ("potato_chilli+carne", 14),
        ("potato_mushroom+onion", 20),
        ("potato_cheese", 16),
        ("potato_tuna+sweetcorn", 22),
        ("cooked_chicken", 3),
        ("cooked_meat", 3),
        ("shrimp", 3),
        ("sardine", 4),
        ("herring", 5),
        ("mackerel", 6),
        ("trout", 7),
        ("cod", 7),
        ("pike", 8),
        ("salmon", 9),
        ("tuna", 10),
        ("lobster", 12),
        ("bass", 13),
        ("swordfish", 14),
        ("monkfish", 16),
        ("shark", 20),
        ("seaturtle", 21),
        ("mantaray", 22),
        ("dark_crab", 22),
        ("edible_seaweed", 4),
    ] {
        foods.push(bite(symbol, heal, 3, None, None));
    }
    foods
}
#[derive(Clone, Debug)]
pub struct Edible {
    pub id: i32,
    pub heal: i32,
    pub delay: i32,
    pub replacement: Option<i32>,
    pub message: String,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Animation(u16),
    Stat(Update),
    Message(String),
    Delay(i32),
    Energy(i32),
}
/// Invoke from a protected inventory-op handler. The original consumes even at full HP,
/// ignores transaction failure, and awards no XP. Events preserve the script's ordering.
pub fn eat(
    food: &Edible,
    inventory: &mut Inventory,
    stats: &mut Stats,
    items: &Catalog,
    hitpoints: u8,
    animation: u16,
    mut emit: impl FnMut(Event),
) -> Result<bool, stat_ops::Error> {
    if !inventory
        .slots()
        .iter()
        .flatten()
        .any(|i| i.id == food.id && i.count > 0)
    {
        return Ok(false);
    }
    emit(Event::Animation(animation));
    let mut transaction = inventory.clone();
    let result = transaction
        .delete(items, Delete::new(food.id, 1))
        .and_then(|_| {
            if let Some(replacement) = food.replacement {
                transaction.insert(items, Insert::new(replacement, 1))
            } else {
                Ok(crate::inventory::Success {
                    requested: 1,
                    completed: 1,
                })
            }
        });
    if result.is_ok() {
        *inventory = transaction;
    }
    let update = stats.apply(hitpoints, Operation::Heal, food.heal, 0, true)?;
    emit(Event::Stat(update));
    emit(Event::Message(food.message.clone()));
    emit(Event::Delay(food.delay));
    Ok(true)
}
/// Ordinary food secondary boosts use the existing stat operation, not a
/// separate progression engine. Definitions resolve skills before registration.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct StatBoost {pub skill:u8,pub amount:i32}
/// Repeated flat subtraction (alcohol), distinct from a base-capped Drain.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct StatSubtraction {pub skill:u8,pub amount:i32}
/// Positive capped non-HP healing; preserves an existing level above base.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct StatHealing {pub skill:u8,pub amount:i32}
/// Boosts use live base levels; drains use live current levels at consumption.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct StatFormula {pub skill:u8,pub percent:u8,pub constant:u8}
impl StatFormula {
    pub fn amount(self,stats:&Stats,hitpoints:u8)->Result<i32,stat_ops::Error>{self.amount_at(stats.base(self.skill)as u8,hitpoints)}
    pub fn subtraction_amount(self,stats:&Stats,hitpoints:u8)->Result<i32,stat_ops::Error>{self.amount_at(stats.current(self.skill)as u8,hitpoints)}
    fn amount_at(self,level:u8,hitpoints:u8)->Result<i32,stat_ops::Error>{
        if self.skill>=24 || self.skill==hitpoints || self.percent>100{return Err(stat_ops::Error::InvalidPercent);}
        let amount=i32::from(level)*i32::from(self.percent)/100+i32::from(self.constant);
        if amount>255{return Err(stat_ops::Error::InvalidPercent);}Ok(amount)
    }
}
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct StatEffects {pub boosts:Vec<StatFormula>,pub subtractions:Vec<StatFormula>}
impl StatEffects {
    pub fn resolve(&self,stats:&Stats,hitpoints:u8)->Result<(Vec<StatBoost>,Vec<StatSubtraction>),stat_ops::Error>{
        let boosts=self.boosts.iter().map(|f|Ok(StatBoost{skill:f.skill,amount:f.amount(stats,hitpoints)?})).collect::<Result<Vec<_>,stat_ops::Error>>()?;
        let subtractions=self.subtractions.iter().map(|f|Ok(StatSubtraction{skill:f.skill,amount:f.subtraction_amount(stats,hitpoints)?})).collect::<Result<Vec<_>,stat_ops::Error>>()?;
        Ok((boosts,subtractions))
    }
}
/// Base-HP dependent ordinary healing, evaluated at the actual Eat action.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Healing {pub percent:u8,pub constant:i32,pub round_up:bool}
impl Healing {
    pub fn amount(self,base:u8)->Result<i32,stat_ops::Error>{
        if self.constant<0{return Err(stat_ops::Error::NegativeConstant);}
        if self.percent>100{return Err(stat_ops::Error::InvalidPercent);}
        let product=i32::from(base)*i32::from(self.percent);
        Ok(((product+if self.round_up{99}else{0})/100).saturating_add(self.constant))
    }
}

/// Selected nonstackable Wiki profile: consume the clicked slot and retain its
/// remainder there. Inventory and healing are staged before any visible effect.
/// `eat` remains the independently tested original-reference policy.
pub fn eat_slot(
    food: &Edible,
    slot: usize,
    inventory: &mut Inventory,
    stats: &mut Stats,
    items: &Catalog,
    hitpoints: u8,
    animation: u16,
    emit: impl FnMut(Event),
) -> Result<bool, stat_ops::Error> {
    eat_slot_with_boosts(food,slot,inventory,stats,items,hitpoints,animation,&[],emit)
}
/// Stage secondary skill boosts with the same selected food/HP transaction.
pub fn eat_slot_with_boosts(
    food:&Edible,slot:usize,inventory:&mut Inventory,stats:&mut Stats,
    items:&Catalog,hitpoints:u8,animation:u16,boosts:&[StatBoost],emit:impl FnMut(Event),
)->Result<bool,stat_ops::Error>{
    eat_slot_transaction(food,slot,inventory,stats,items,hitpoints,animation,boosts,&[],&[],false,||Ok(food.heal),emit)
}
/// Inclusive healing range; resolved only after the selected item, remainder
/// and secondary-effect policy have been validated in the transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomHealing {
    pub minimum: u8,
    pub maximum: u8,
}
impl RandomHealing {
    fn roll(self, random: &mut impl crate::npc::RandomSource) -> Result<i32, stat_ops::Error> {
        if self.minimum > self.maximum { return Err(stat_ops::Error::InvalidPercent); }
        let width = u32::from(self.maximum) - u32::from(self.minimum) + 1;
        Ok(i32::from(self.minimum) + if width == 1 { 0 } else { random.below(width) as i32 })
    }
}

/// Random ordinary food uses the same staged inventory/HP/boost transaction.
/// Stackable remainders use the ordinary inventory stack merge (e.g. skewers).
pub fn eat_slot_with_random_healing(
    food: &Edible, slot: usize, inventory: &mut Inventory, stats: &mut Stats,
    items: &Catalog, hitpoints: u8, animation: u16, boosts: &[StatBoost],
    healing: RandomHealing, random: &mut impl crate::npc::RandomSource,
    emit: impl FnMut(Event),
) -> Result<bool, stat_ops::Error> {
    eat_slot_transaction(food, slot, inventory, stats, items, hitpoints, animation,
        boosts, &[], &[], true, || healing.roll(random), emit)
}


/// Shared staged food transaction with positive boosts and stacking flat drains.
/// Invalid subtraction policy is rejected before resolving random healing.
pub fn eat_slot_with_secondary_effects(
    food:&Edible,slot:usize,inventory:&mut Inventory,stats:&mut Stats,
    items:&Catalog,hitpoints:u8,animation:u16,boosts:&[StatBoost],
    subtractions:&[StatSubtraction],healing:Option<RandomHealing>,
    random:&mut impl crate::npc::RandomSource,emit:impl FnMut(Event),
)->Result<bool,stat_ops::Error>{
    eat_slot_transaction(food,slot,inventory,stats,items,hitpoints,animation,boosts,subtractions,&[],
        healing.is_some(),||match healing{Some(h)=>h.roll(random),None=>Ok(food.heal)},emit)
}

/// Capped stat healing joins the same staged HP/boost/subtraction transaction.
/// Old food APIs retain their behavior and delegate with no healing metadata.
pub fn eat_slot_with_stat_healing(
    food:&Edible,slot:usize,inventory:&mut Inventory,stats:&mut Stats,
    items:&Catalog,hitpoints:u8,animation:u16,boosts:&[StatBoost],
    subtractions:&[StatSubtraction],restores:&[StatHealing],healing:Option<RandomHealing>,
    random:&mut impl crate::npc::RandomSource,emit:impl FnMut(Event),
)->Result<bool,stat_ops::Error>{
    eat_slot_transaction(food,slot,inventory,stats,items,hitpoints,animation,boosts,subtractions,restores,
        healing.is_some(),||match healing{Some(h)=>h.roll(random),None=>Ok(food.heal)},emit)
}

fn eat_slot_transaction(
    food:&Edible,slot:usize,inventory:&mut Inventory,stats:&mut Stats,
    items:&Catalog,hitpoints:u8,animation:u16,boosts:&[StatBoost],
    subtractions:&[StatSubtraction],restores:&[StatHealing],allow_stack_remainder:bool,resolve_heal:impl FnOnce()->Result<i32,stat_ops::Error>,mut emit:impl FnMut(Event),
)->Result<bool,stat_ops::Error>{
    if restores.iter().any(|s|s.amount<0){return Err(stat_ops::Error::NegativeConstant);}
    if restores.iter().any(|s|s.skill>=24 || s.skill==hitpoints || s.amount>255){return Err(stat_ops::Error::InvalidPercent);}
    if !subtractions.is_empty() || !restores.is_empty() {
        if inventory.stack!=crate::inventory::StackType::Normal || inventory.placeholders {return Ok(false);}
        if subtractions.iter().any(|s|s.amount<0){return Err(stat_ops::Error::NegativeConstant);}
        if subtractions.iter().any(|s|s.skill>=24 || s.skill==hitpoints || s.amount>255){return Err(stat_ops::Error::InvalidPercent);}
        if inventory.get(slot).is_some_and(|i|i.vars!=0){return Ok(false);}
    }
    let ordinary=|id| id>=0
        && !items.dummy.contains(&id)
        && !items.notes.get(&id).is_some_and(|v|v.template!=0)
        && !items.placeholders.get(&id).is_some_and(|v|v.template!=0)
        && !items.transforms.get(&id).is_some_and(|v|v.template!=0);
    if inventory
        .get(slot)
        .is_none_or(|item| item.id != food.id || item.count != 1)
        || !ordinary(food.id) || items.stackable.contains(&food.id)
        || food.replacement.is_some_and(|id|id==food.id || !ordinary(id) || (!allow_stack_remainder && items.stackable.contains(&id)))
    {
        return Ok(false);
    }
    let mut next_inventory = inventory.clone();
    let mut debit = Delete::new(food.id, 1);
    debit.slot = slot;
    debit.strict_slot = Some(slot);
    if next_inventory.delete(items, debit).is_err() {
        return Ok(false);
    }
    if let Some(id) = food.replacement {
        let mut credit = Insert::new(id, 1);
        credit.slot = slot;
        let stack_remainder=allow_stack_remainder && items.stackable.contains(&id);
        if stack_remainder && (inventory.stack!=crate::inventory::StackType::Normal || inventory.placeholders) {return Ok(false);}
        if stack_remainder && inventory.slots().iter().flatten().any(|i|i.id==id && (i.vars!=0 || i.count<=0)){return Ok(false);}
        credit.strict_slot = (!stack_remainder).then_some(slot);
        if next_inventory.insert(items, credit).is_err()
            || (!stack_remainder && next_inventory.get(slot).is_none_or(|item|item.id!=id || item.count!=1 || item.vars!=0)) {
            return Ok(false);
        }
    }
    if food.heal<0 || boosts.iter().any(|boost|boost.amount<0) {return Err(stat_ops::Error::NegativeConstant);}
    let heal=resolve_heal()?;
    if heal<0 {return Err(stat_ops::Error::NegativeConstant);}
    let mut next_stats = stats.clone();
    let current=stats.current(hitpoints) as u8 as i32;
    let base=stats.base(hitpoints) as u8 as i32;
    let gain=heal.min((base-current).max(0));
    let update = if gain>0 {Some(next_stats.apply(hitpoints,Operation::Add,gain,0,true)?)} else {None};
    let mut secondary=Vec::new();
    for boost in boosts {secondary.push(next_stats.apply(boost.skill,Operation::Boost,boost.amount,0,false)?);}
    for restore in restores {
        if restore.amount>0 && (next_stats.current(restore.skill)as u8)<(next_stats.base(restore.skill)as u8){secondary.push(next_stats.apply(restore.skill,Operation::Heal,restore.amount,0,false)?);}
    }
    for subtraction in subtractions {secondary.push(next_stats.apply(subtraction.skill,Operation::Subtract,subtraction.amount,0,false)?);}
    *inventory = next_inventory;
    *stats = next_stats;
    emit(Event::Animation(animation));
    if let Some(update)=update {emit(Event::Stat(update));}
    emit(Event::Message(food.message.clone()));
    emit(Event::Delay(food.delay));
    for update in secondary {emit(Event::Stat(update));}
    Ok(true)
}
/// Successful selected-slot transaction, captured by the authoritative world.
/// Configured food delay is not an observation of the combat attack clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition {
    pub tick: i32,
    pub slot: usize,
    pub before: crate::inventory::Item,
    pub after: Option<crate::inventory::Item>,
    pub hp_before: u8,
    pub hp_after: u8,
    pub fine_xp_before: i32,
    pub fine_xp_after: i32,
    pub configured_delay: i32,
    pub attack_delay: Option<i32>,
}
impl Transition {
    pub fn json(&self, actor: crate::simulation::ActorId) -> String {
        let item = |v: crate::inventory::Item| format!("[{},{},{}]", v.id, v.count, v.vars);
        format!(
            concat!("{{\"schema\":1,\"actor_slot\":{},\"actor_generation\":{},\"tick\":{},",
                "\"policy\":\"selected_slot\",\"result\":\"committed\",\"slot\":{},",
                "\"before\":{},\"after\":{},\"hp_before\":{},\"hp_after\":{},",
                "\"fine_xp_before\":{},\"fine_xp_after\":{},\"configured_delay\":{},\"attack_delay\":{}}}"),
            actor.slot, actor.generation, self.tick, self.slot, item(self.before),
            self.after.map(item).unwrap_or_else(|| "null".into()), self.hp_before,
            self.hp_after, self.fine_xp_before, self.fine_xp_after, self.configured_delay,
            self.attack_delay.map(|n| n.to_string()).unwrap_or_else(|| "null".into())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn dynamic_food_healing_matches_explicit_wiki_table_boundaries(){
        let strawberry=Healing{percent:6,constant:0,round_up:true};for (base,heal) in [(10,1),(16,1),(17,2),(33,2),(34,3),(50,3),(51,4),(66,4),(67,5),(83,5),(84,6),(99,6)]{assert_eq!(strawberry.amount(base),Ok(heal));}
        let melon=Healing{percent:5,constant:1,round_up:false};for (base,heal) in [(10,1),(19,1),(20,2),(39,2),(40,3),(59,3),(60,4),(79,4),(80,5),(99,5)]{assert_eq!(melon.amount(base),Ok(heal));}
        let corn=Healing{percent:10,constant:1,round_up:false};for (base,heal) in [(10,2),(19,2),(20,3),(99,10)]{assert_eq!(corn.amount(base),Ok(heal));}
        assert_eq!(Healing{percent:101,..corn}.amount(99),Err(stat_ops::Error::InvalidPercent));assert_eq!(Healing{constant:-1,..corn}.amount(99),Err(stat_ops::Error::NegativeConstant));
    }
    #[test]
    fn secondary_food_boosts_cap_and_rollback_with_selected_item_and_healing(){
        let food=Edible{id:1,heal:8,delay:1,replacement:Some(2),message:"Pie".into()};
        for current in [50,99,105,110]{let mut inv=Inventory::new(1,StackType::Normal,false);inv.restore_slot(0,Some(Item{id:1,count:1,vars:0})).unwrap();let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,115);stats.set_base(10,99);stats.set_current(10,current);let xp=stats.fine_xp(10);eat_slot_with_boosts(&food,0,&mut inv,&mut stats,&Default::default(),3,829,&[StatBoost{skill:10,amount:5}],|_|{}).unwrap();assert_eq!(stats.current(3),115);assert_eq!(stats.current(10),match current{50=>55,99=>104,_=>current});assert_eq!(stats.fine_xp(10),xp);assert_eq!(inv.get(0).unwrap().id,2);}
        let mut inv=Inventory::new(1,StackType::Normal,false);inv.restore_slot(0,Some(Item{id:1,count:1,vars:0})).unwrap();let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,50);stats.set_base(10,99);stats.set_current(10,99);let before=(inv.clone(),stats.clone());let mut events=Vec::new();assert!(eat_slot_with_boosts(&food,0,&mut inv,&mut stats,&Default::default(),3,829,&[StatBoost{skill:10,amount:5},StatBoost{skill:18,amount:-1}],|e|events.push(e)).is_err());assert_eq!((inv,stats),before);assert!(events.is_empty());
    }
    #[test]
    fn ordinary_food_preserves_overhealed_health_and_rejects_malformed_exchange() {
        for hp in [50,99,115] {
            let mut inv=Inventory::new(2,StackType::Normal,false);for slot in 0..2{inv.restore_slot(slot,Some(Item{id:1,count:1,vars:0})).unwrap();}
            let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,hp);let xp=stats.fine_xp(3);let mut events=Vec::new();
            let food=Edible{id:1,heal:20,delay:3,replacement:None,message:"Shark".into()};
            assert!(eat_slot(&food,1,&mut inv,&mut stats,&Catalog::default(),3,829,|e|events.push(e)).unwrap());
            assert_eq!(stats.current(3),if hp==50{70}else{hp});assert_eq!(stats.fine_xp(3),xp);assert!(inv.get(1).is_none());assert_eq!(inv.get(0).unwrap().id,1);assert_eq!(events.len(),if hp==50{4}else{3});
        }
        for mode in 0..8 {
            let mut inv=Inventory::new(2,if mode==7{StackType::Always}else{StackType::Normal},false);inv.restore_slot(1,Some(Item{id:1,count:1,vars:0})).unwrap();if mode==7{inv.restore_slot(0,Some(Item{id:2,count:1,vars:0})).unwrap();}
            let mut items=Catalog::default();match mode{0=>{items.dummy.insert(1);},1=>{items.notes.insert(1,crate::inventory::Template{link:8,template:799});},2=>{items.placeholders.insert(1,crate::inventory::Template{link:8,template:14401});},3=>{items.transforms.insert(2,crate::inventory::Template{link:8,template:1});},4=>{items.stackable.insert(2);},_=>{}}
            let food=Edible{id:1,heal:if mode==6{-1}else{20},delay:3,replacement:Some(if mode==5{1}else{2}),message:"Food".into()};let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,115);let before=(inv.clone(),stats.clone());let mut events=Vec::new();
            let result=eat_slot(&food,1,&mut inv,&mut stats,&items,3,829,|e|events.push(e));if mode==6{assert_eq!(result,Err(stat_ops::Error::NegativeConstant));}else{assert_eq!(result,Ok(false));}assert_eq!((inv,stats),before);assert!(events.is_empty());
        }
    }
    #[test]
    fn selected_bite_keeps_earlier_identical_food_and_empty_slot_untouched() {
        let mut inv = Inventory::new(4, StackType::Normal, false);
        for slot in [1, 3] {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 1,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let mut stats = Stats::default();
        stats.set_base(3, 20);
        stats.set_current(3, 1);
        let mut food = Edible {
            id: 1,
            heal: 4,
            delay: 2,
            replacement: Some(2),
            message: "Cake".into(),
        };
        for (id, replacement, hp) in [(1, Some(2), 5), (2, Some(3), 9), (3, None, 13)] {
            food.id = id;
            food.replacement = replacement;
            assert_eq!(
                eat_slot(
                    &food,
                    3,
                    &mut inv,
                    &mut stats,
                    &Catalog::default(),
                    3,
                    829,
                    |_| {}
                ),
                Ok(true)
            );
            assert_eq!(inv.get(3).map(|i| i.id), replacement);
            assert!(inv.get(0).is_none());
            assert_eq!(inv.get(1).unwrap().id, 1);
            assert_eq!(stats.current(3), hp);
            assert_eq!(stats.fine_xp(3), 0);
        }
        assert_eq!(
            eat_slot(
                &food,
                3,
                &mut inv,
                &mut stats,
                &Catalog::default(),
                3,
                829,
                |_| panic!()
            ),
            Ok(false)
        );
    }
    #[test]
    fn selected_bite_rejects_remainder_or_stat_failure_before_any_commit() {
        for bad_stat in [false, true] {
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: 1,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            let mut catalog = Catalog::default();
            if !bad_stat {
                catalog.dummy.insert(2);
            }
            let food = Edible {
                id: 1,
                heal: if bad_stat { -1 } else { 4 },
                delay: 2,
                replacement: Some(2),
                message: "Cake".into(),
            };
            let before = (inv.clone(), stats.clone());
            let result = eat_slot(
                &food,
                0,
                &mut inv,
                &mut stats,
                &catalog,
                3,
                829,
                |_| panic!(),
            );
            if bad_stat {
                assert_eq!(result, Err(stat_ops::Error::NegativeConstant));
            } else {
                assert_eq!(result, Ok(false));
            }
            assert_eq!((inv, stats), before);
        }
    }
    #[test]
    fn catalog_preserves_diary_heals_bowls_and_source_cake_quirk() {
        let foods = catalog();
        assert_eq!(foods.len(), 53);
        let by_name: std::collections::BTreeMap<_, _> =
            foods.iter().map(|f| (f.symbol, f)).collect();
        assert_eq!(by_name.len(), 53);
        assert_eq!(by_name["cooked_chicken"].heal, 3);
        assert_eq!(by_name["curry"].replacement, Some("bowl_empty"));
        assert_eq!(by_name["cake"].replacement, None);
        assert_eq!(by_name["plain_pizza"].delay, 1);
        assert_eq!(by_name["shark"].heal, 20);
    }
    #[test]
    fn full_health_consumes_and_replacement_uses_first_free_slot() {
        let mut inventory = Inventory::new(3, StackType::Normal, false);
        inventory
            .restore_slot(
                2,
                Some(Item {
                    id: 1,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let food = Edible {
            id: 1,
            heal: 19,
            delay: 3,
            replacement: Some(2),
            message: "You eat the curry.".into(),
        };
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        let xp = stats.fine_xp(3);
        let mut events = vec![];
        assert_eq!(
            eat(
                &food,
                &mut inventory,
                &mut stats,
                &Catalog::default(),
                3,
                829,
                |e| events.push(e)
            ),
            Ok(true)
        );
        assert_eq!(inventory.get(0).unwrap().id, 2);
        assert!(inventory.get(2).is_none());
        assert_eq!(stats.current(3), 10);
        assert_eq!(stats.fine_xp(3), xp);
        assert_eq!(events.first(), Some(&Event::Animation(829)));
        assert_eq!(events.last(), Some(&Event::Delay(3)));
        assert_eq!(
            eat(
                &food,
                &mut inventory,
                &mut stats,
                &Catalog::default(),
                3,
                829,
                |_| panic!()
            ),
            Ok(false)
        );
    }
}

#[cfg(test)]
mod random_healing_tests {
    use super::*;
    use crate::{inventory::{Item, StackType}, npc::RandomSource};
    struct Draw { high: bool, calls: u32 }
    impl RandomSource for Draw { fn below(&mut self, n:u32)->u32 {self.calls+=1;if self.high{n-1}else{0}} }
    #[test]
    fn random_food_transaction_draws_after_validation_and_merges_stackable_return_atomically() {
        let food=Edible{id:1,heal:7,delay:3,replacement:Some(2),message:"Spider".into()};
        let mut catalog=Catalog::default();catalog.stackable.insert(2);
        for (count,current,expected) in [(0,1,11),(3,99,99),(3,115,115)] {
            let mut inv=Inventory::new(2,StackType::Normal,false);inv.restore_slot(1,Some(Item{id:1,count:1,vars:0})).unwrap();if count>0{inv.restore_slot(0,Some(Item{id:2,count,vars:0})).unwrap();}
            let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,current);let mut draw=Draw{high:true,calls:0};
            assert_eq!(eat_slot_with_random_healing(&food,1,&mut inv,&mut stats,&catalog,3,829,&[],RandomHealing{minimum:7,maximum:10},&mut draw,|_|{}),Ok(true));
            assert_eq!(draw.calls,1);assert_eq!(stats.current(3),expected);assert_eq!(stats.fine_xp(3),0);
            assert_eq!(inv.slots().iter().flatten().filter(|i|i.id==2).map(|i|i.count).sum::<i32>(),count+1);
            if count>0{assert!(inv.get(1).is_none());assert_eq!(inv.get(0).unwrap().count,count+1);}else{assert_eq!(inv.get(1).unwrap().id,2);}
        }
        for bad in 0..7 {
            let mut inv=Inventory::new(2,StackType::Normal,false);inv.restore_slot(1,Some(Item{id:1,count:1,vars:0})).unwrap();
            let mut catalog=Catalog::default();catalog.stackable.insert(2);let mut stats=Stats::default();let mut draw=Draw{high:false,calls:0};
            let mut rule=RandomHealing{minimum:7,maximum:10};let mut boosts=Vec::new();
            match bad {0=>inv.restore_slot(1,Some(Item{id:9,count:1,vars:0})).unwrap(),1=>{catalog.dummy.insert(2);},2=>{rule.minimum=11;},3=>boosts.push(StatBoost{skill:4,amount:-1}),4=>inv.restore_slot(0,Some(Item{id:2,count:i32::MAX,vars:0})).unwrap(),5=>inv.restore_slot(0,Some(Item{id:2,count:1,vars:1})).unwrap(),_=>{inv.stack=StackType::Always;}}
            let before=(inv.clone(),stats.clone());let mut events=Vec::new();let result=eat_slot_with_random_healing(&food,1,&mut inv,&mut stats,&catalog,3,829,&boosts,rule,&mut draw,|e|events.push(e));
            assert!(result!=Ok(true));assert_eq!(draw.calls,0);assert_eq!((inv,stats),before);assert!(events.is_empty());
        }
    }

    struct NoFoodDraw;
    impl crate::npc::RandomSource for NoFoodDraw {fn below(&mut self,_:u32)->u32{panic!("fixed wine or invalid policy drew RNG")}}
    #[test]
    fn wine_secondary_subtraction_stacks_floors_and_keeps_selected_remainder_with_healing(){
        let f=Edible{id:1993,heal:11,delay:3,replacement:Some(1935),message:"Wine".into()};
        for hp in [1,98,99,115]{for attack in [0,1,2,5,99,105]{let mut inv=Inventory::new(2,StackType::Normal,false);for slot in 0..2{inv.restore_slot(slot,Some(Item{id:1993,count:1,vars:0})).unwrap();}let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,hp);stats.set_base(0,99);stats.set_current(0,attack);stats.set_fine_xp(0,12345).unwrap();let mut events=Vec::new();
            for slot in [1,0]{assert!(eat_slot_with_secondary_effects(&f,slot,&mut inv,&mut stats,&Catalog::default(),3,829,&[],&[StatSubtraction{skill:0,amount:2}],None,&mut NoFoodDraw,|e|events.push(e)).unwrap());assert_eq!(inv.get(slot),Some(Item{id:1935,count:1,vars:0}));}
            assert_eq!(stats.current(0),(attack-4).max(0));assert_eq!(stats.current(3),if hp>99{hp}else{(hp+22).min(99)});assert_eq!(stats.fine_xp(0),12345);assert_eq!(stats.fine_xp(3),0);assert_eq!(events.iter().filter(|e|matches!(e,Event::Stat(u)if u.skill==0)).count(),2);
        }}
    }
    #[test]
    fn wine_subtraction_policy_and_malformed_exchange_refuse_before_rng_or_publication(){
        let f=Edible{id:1993,heal:11,delay:3,replacement:Some(1935),message:"Wine".into()};
        for mode in 0..9{let mut inv=Inventory::new(1,if mode==7{StackType::Always}else{StackType::Normal},mode==8);inv.restore_slot(0,Some(Item{id:1993,count:1,vars:if mode==5{1}else{0}})).unwrap();let mut stats=Stats::default();stats.set_current(0,50);stats.set_current(3,1);let mut catalog=Catalog::default();let effect=match mode{0=>StatSubtraction{skill:0,amount:-1},1=>StatSubtraction{skill:0,amount:256},2=>StatSubtraction{skill:24,amount:2},3=>StatSubtraction{skill:3,amount:2},_=>StatSubtraction{skill:0,amount:2}};if mode==4{catalog.stackable.insert(1935);}if mode==6{catalog.dummy.insert(1935);}
            let before=(inv.clone(),stats.clone());let mut events=Vec::new();let result=eat_slot_with_secondary_effects(&f,0,&mut inv,&mut stats,&catalog,3,829,&[],&[effect],None,&mut NoFoodDraw,|e|events.push(e));assert!(!matches!(result,Ok(true)));assert_eq!((inv,stats),before);assert!(events.is_empty());
        }
        // Invalid secondary policy is rejected before even random-healing rolls.
        let mut inv=Inventory::new(1,StackType::Normal,false);inv.restore_slot(0,Some(Item{id:1993,count:1,vars:0})).unwrap();let before=inv.clone();let mut stats=Stats::default();assert!(eat_slot_with_secondary_effects(&f,0,&mut inv,&mut stats,&Catalog::default(),3,829,&[],&[StatSubtraction{skill:0,amount:-1}],Some(RandomHealing{minimum:1,maximum:3}),&mut NoFoodDraw,|_|panic!("event on refusal")).is_err());assert_eq!(inv,before);
    }

    #[test]
    fn food_stat_formulas_floor_live_base_percentages_and_reject_invalid_policy(){
        for(base,want)in[(1,1),(16,1),(17,2),(50,4),(99,6)]{let mut stats=Stats::default();stats.set_base(0,base);stats.set_current(0,1);assert_eq!(StatFormula{skill:0,percent:6,constant:1}.amount(&stats,3),Ok(want));stats.set_current(0,110);assert_eq!(StatFormula{skill:0,percent:6,constant:1}.amount(&stats,3),Ok(want));}
        let mut stats=Stats::default();stats.set_base(0,99);for formula in[StatFormula{skill:24,percent:0,constant:1},StatFormula{skill:3,percent:0,constant:1},StatFormula{skill:0,percent:101,constant:1},StatFormula{skill:0,percent:100,constant:255}]{assert!(formula.amount(&stats,3).is_err());let policy=StatEffects{boosts:vec![StatFormula{skill:2,percent:2,constant:1}],subtractions:vec![formula]};let before=stats.clone();assert!(policy.resolve(&stats,3).is_err());assert_eq!(stats,before);}
    }

    #[test]
    fn food_percentage_drains_use_current_level_while_boosts_keep_static_base(){
        let policy=StatEffects{boosts:vec![StatFormula{skill:2,percent:2,constant:1}],subtractions:vec![StatFormula{skill:0,percent:6,constant:1}]};let mut stats=Stats::default();stats.set_base(0,99);stats.set_base(2,99);stats.set_current(2,110);
        for(current,want)in[(0,1),(10,1),(16,1),(17,2),(50,4),(99,6),(100,7),(255,16)]{stats.set_current(0,current as u8 as i8);let(b,s)=policy.resolve(&stats,3).unwrap();assert_eq!(b[0].amount,2);assert_eq!(s[0].amount,want);}
        stats.set_base(0,1);stats.set_current(0,99);assert_eq!(policy.resolve(&stats,3).unwrap().1[0].amount,6);
    }

    #[test]
    fn capped_food_stat_healing_policy_and_inventory_refuse_before_rng_or_any_effect(){
        let f=Edible{id:247,heal:2,delay:3,replacement:None,message:"Berries".into()};for mode in 0..9{let mut inv=Inventory::new(1,if mode==6{StackType::Always}else{StackType::Normal},mode==7);inv.restore_slot(0,Some(Item{id:247,count:1,vars:if mode==4{1}else{0}})).unwrap();let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,50);stats.set_base(5,99);stats.set_current(5,0);let mut catalog=Catalog::default();if mode==5{catalog.dummy.insert(247);}if mode==8{catalog.stackable.insert(247);}let bad=match mode{0=>StatHealing{skill:5,amount:-1},1=>StatHealing{skill:24,amount:1},2=>StatHealing{skill:3,amount:1},3=>StatHealing{skill:5,amount:256},_=>StatHealing{skill:5,amount:1}};let before=(inv.clone(),stats.clone());let result=eat_slot_with_stat_healing(&f,0,&mut inv,&mut stats,&catalog,3,829,&[StatBoost{skill:0,amount:2}],&[StatSubtraction{skill:1,amount:1}],&[StatHealing{skill:5,amount:1},bad],Some(RandomHealing{minimum:1,maximum:3}),&mut NoFoodDraw,|_|panic!("effect on rejected food healing"));assert!(!matches!(result,Ok(true)));assert_eq!((inv,stats),before);}
    }
}
