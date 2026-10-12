use gielinor_cache::symbols::Symbols;
use gielinor_world::ui::Component;
#[derive(Clone, Copy)]
pub struct Presentation {
    pub parent: Component,
    pub interface: u16,
    pub combat_interface: u16,
    pub spells: Component,
    pub normal: Component,
    pub defensive: Component,
    pub layout: u32,
}
impl Presentation {
    pub fn resolve(
        interfaces: &Symbols,
        components: &Symbols,
        varps: &Symbols,
    ) -> Result<Self, String> {
        let id = |map: &Symbols, name: &str| {
            map.get(name)
                .copied()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| format!("Missing autocast picker symbol {name}"))
        };
        let short = |name: &str| {
            u16::try_from(id(interfaces, name)?)
                .map_err(|_| format!("Invalid picker interface {name}"))
        };
        Ok(Self {
            parent: Component(id(components, "toplevel_osrs_stretch:side0")?),
            interface: short("autocast")?,
            combat_interface: short("combat_interface")?,
            spells: Component(id(components, "autocast:spells")?),
            normal: Component(id(components, "combat_interface:normal_container")?),
            defensive: Component(id(components, "combat_interface:defensive_container")?),
            layout: id(varps, "spitfire_coord")?,
        })
    }
    pub fn layout_weapon(weapon: i32) -> i32 {
        if [
            1409, 4170, 4675, 4710, 8841, 9013, 11791, 21006, 21276, 22296, 24144, 27676, 27679,
            27785, 27788,
        ]
        .contains(&weapon)
        {
            weapon
        } else {
            -1
        }
    }
}
