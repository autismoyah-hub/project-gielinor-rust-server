//! Reached held-item refusals share normal NPC routing and selected-slot authority.
#[derive(Clone,Copy,Debug)]
pub struct Rule {
    pub special_item:i32,
    pub special:&'static str,
    pub otherwise:&'static str,
}
