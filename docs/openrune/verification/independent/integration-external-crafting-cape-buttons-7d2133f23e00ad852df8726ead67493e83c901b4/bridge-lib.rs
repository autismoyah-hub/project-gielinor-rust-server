mod pinned { include!("C:/Projects/OSRSRSPS/agent7-targets/7d2133f23e00ad852df8726ead67493e83c901b4/crates/server/src/runtime/equipment_buttons.rs"); }
pub fn route(button:&gielinor_protocol::buttons::Button,slot:usize,item:i32,rules:&std::collections::BTreeMap<i32,gielinor_world::worn_stat_boost::Rule>)->Option<gielinor_world::simulation::Action>{pinned::boost_action(button,slot,item,rules)}
