//! Manual inventory operations for the attested coal-bag forms.
#[derive(Clone,Copy,Debug)]
pub struct Config {
    pub closed:i32,
    pub open:i32,
    pub coal:i32,
    pub capes:[i32;3],
    pub base:crate::variables::Varp,
    pub stored:crate::variables::Varbit,
}
impl Config {
    pub fn contains(self,item:i32)->bool {item==self.closed||item==self.open}
    /// Held options 1..4 use widget buttons 2/3/4/6; button5 is reserved.
    pub fn accepts_operation(self,operation:u8)->bool {matches!(operation,2|3|4|6)}
    pub fn capacity(self,worn:&crate::inventory::Inventory)->i32 {
        if worn.slots().iter().flatten().any(|item|self.capes.contains(&item.id)){36}else{27}
    }
    /// Delayed furnace consumption uses current open-form/membership authority.
    /// Saved coal above the current cape capacity remains spendable.
    pub fn consumable(self, inventory:&crate::inventory::Inventory, variables:&crate::variables::Variables, member:bool)->i32 {
        if !member || !inventory.slots().iter().flatten().any(|i|i.id==self.open && i.count==1 && i.vars==0){return 0;}
        variables.bit(self.stored).unwrap_or(0).max(0)
    }
    /// Stage incoming coal separately from the actor until the reward commits.
    pub(crate) fn incoming(self,inventory:&crate::inventory::Inventory,worn:&crate::inventory::Inventory,variables:&crate::variables::Variables,member:bool,item:i32)->Result<Option<crate::variables::Variables>,crate::variables::Error>{
        if !member||item!=self.coal||!inventory.slots().iter().flatten().any(|i|i.id==self.open&&i.count==1&&i.vars==0){return Ok(None);}
        let stored=variables.bit(self.stored)?;
        if stored>=self.capacity(worn){return Ok(None);}
        let mut next=variables.clone();next.set_bit(self.base,self.stored,stored+1,false)?;
        Ok(Some(next))
    }
}
