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
    pub fn capacity(self,worn:&crate::inventory::Inventory)->i32 {
        if worn.slots().iter().flatten().any(|item|self.capes.contains(&item.id)){36}else{27}
    }
}
