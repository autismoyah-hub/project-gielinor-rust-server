#[path="C:/Projects/OSRSRSPS/agent7-targets/79a16ed1b33a402b59d57ace40a7ec978e28b32e/crates/server/src/food_operations.rs"] mod pinned;
pub fn operation(item:&gielinor_cache::definitions::ItemType)->Option<u8>{pinned::button_operation(item)}
pub fn route(button:&gielinor_protocol::buttons::Button,slot:usize,current:i32,foods:&std::collections::BTreeMap<i32,gielinor_world::food::Edible>,defs:&gielinor_cache::game::Definitions,native:bool)->Option<gielinor_world::simulation::Action>{pinned::held_action(button,slot,current,foods,defs,native)}
