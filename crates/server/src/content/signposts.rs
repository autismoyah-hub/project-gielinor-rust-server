//! Exact native compass definitions and runtime-parsed direction enum, never a copied map.
use super::{Definitions, Symbols};
use gielinor_cache::enums::{EnumType, Value};
use gielinor_protocol::{
    camera,
    client_script::{self, Argument},
    framing::Packet,
    interfaces,
};
use gielinor_world::{ui::Component, CoordGrid};
use std::collections::BTreeMap;
#[derive(Clone)]
pub struct Signposts {
    pub parent: Component,
    pub interface: u16,
    pub components: [u32; 4],
    pub values: BTreeMap<i32, [String; 4]>,
    pub default: [String; 4],
}
impl Signposts {
    pub fn packets(&self, at: CoordGrid, location: CoordGrid) -> Result<Vec<Packet>, String> {
        let move_to = at
            .translate(0, -6, 0)
            .map_err(|_| "Invalid signpost camera move")?;
        let look = at
            .translate(0, 3, 0)
            .map_err(|_| "Invalid signpost camera look")?;
        let mut packets = vec![
            client_script::run(143, &[Argument::Int(280), Argument::Int(0)])
                .map_err(|e| format!("{e:?}"))?,
            camera::move_to(move_to.x() as u16, move_to.z() as u16, 1500, 2, 10)
                .map_err(|e| format!("{e:?}"))?,
            camera::look_at(look.x() as u16, look.z() as u16, 450, 2, 10)
                .map_err(|e| format!("{e:?}"))?,
        ];
        let texts = self.values.get(&location.packed()).unwrap_or(&self.default);
        for (&component, text) in self.components.iter().zip(texts) {
            packets.push(interfaces::text(component, text).map_err(|e| format!("{e:?}"))?);
        }
        Ok(packets)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        Symbols,
        Symbols,
        Symbols,
        Symbols,
        BTreeMap<u32, EnumType>,
        Definitions,
    ) {
        let l = [("aide_signpost_1".into(), 18493)].into();
        let e = [("signpost_directions".into(), 65533)].into();
        let i = [("aide_compass".into(), 135)].into();
        let mut c: Symbols = [("toplevel_osrs_stretch:mainmodal".into(), (161 << 16) | 13)].into();
        for (name, child) in [
            ("aide_west_text_2", 11),
            ("aide_south_text_2", 8),
            ("aide_north_text_2", 2),
            ("aide_east_text_2", 7),
        ] {
            c.insert(format!("aide_compass:{name}"), (135 << 16) | child);
        }
        let values = (0..4)
            .map(|i| {
                (
                    CoordGrid::new(3204 + i, 3204, 0).unwrap().packed(),
                    Value::String("West|South|North|East".into()),
                )
            })
            .collect();
        let t = EnumType {
            key_literal: 22,
            value_literal: 36,
            default_int: None,
            default_string: Some("Default West|Default South|Default North|Default East".into()),
            values,
            insertion_order: vec![],
            transmit: false,
            decoded_bytes: 0,
        };
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut loc = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
        loc.name = "Signpost".into();
        loc.collision.width = 2;
        loc.collision.length = 2;
        loc.ops[0] = Some("Read".into());
        d.locations.insert(18493, loc);
        (l, e, i, c, [(65533, t)].into(), d)
    }
    #[test]
    fn signposts_exact_enum_components_prepare_source_camera_and_default_directions_before_ui() {
        let (l, e, i, c, t, d) = fixture();
        let s = signposts(&l, &e, &i, &c, &t, &d).unwrap();
        let at = CoordGrid::new(3200, 3200, 0).unwrap();
        for location in [
            CoordGrid::new(3204, 3204, 0).unwrap(),
            CoordGrid::new(3300, 3300, 0).unwrap(),
        ] {
            let packets = s.packets(at, location).unwrap();
            assert_eq!(packets.len(), 7);
            assert_eq!(
                packets[0].payload,
                client_script::run(143, &[Argument::Int(280), Argument::Int(0)])
                    .unwrap()
                    .payload
            );
            assert_eq!((packets[1].opcode, packets[2].opcode), (146, 139));
            let texts = s.values.get(&location.packed()).unwrap_or(&s.default);
            for (k, text) in texts.iter().enumerate() {
                assert_eq!(
                    packets[3 + k].payload,
                    gielinor_protocol::interfaces::text(s.components[k], text)
                        .unwrap()
                        .payload
                );
            }
        }
    }
    #[test]
    fn signposts_late_malformed_enum_or_wrong_component_refuses_full_configuration() {
        let (l, e, i, mut c, mut t, d) = fixture();
        let key = *t[&65533].values.keys().next_back().unwrap();
        t.get_mut(&65533)
            .unwrap()
            .values
            .insert(key, Value::String("only|three|directions".into()));
        assert!(signposts(&l, &e, &i, &c, &t, &d).is_err());
        let (l, e, i, _, t, d) = fixture();
        c.insert("aide_compass:aide_east_text_2".into(), (135 << 16) | 8);
        assert!(signposts(&l, &e, &i, &c, &t, &d).is_err());
    }
}
fn directions(text: &str) -> Result<[String; 4], String> {
    let parts: Vec<_> = text.split('|').collect();
    if parts.len() != 4
        || parts
            .iter()
            .any(|s| s.is_empty() || s.len() > 1024 || s.contains('\0'))
    {
        return Err("Malformed signpost directions".into());
    }
    Ok(parts
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap())
}
pub fn signposts(
    locations: &Symbols,
    enums: &Symbols,
    interfaces: &Symbols,
    components: &Symbols,
    tables: &BTreeMap<u32, EnumType>,
    defs: &Definitions,
) -> Result<Signposts, String> {
    if locations.get("aide_signpost_1") != Some(&18493)
        || enums.get("signpost_directions") != Some(&65533)
        || interfaces.get("aide_compass") != Some(&135)
        || defs.locations.get(&18493).is_none_or(|d| {
            d.name != "Signpost"
                || d.collision.width != 2
                || d.collision.length != 2
                || d.ops[0].as_deref() != Some("Read")
                || d.ops[1..].iter().any(Option::is_some)
                || !d.multi.is_empty()
                || d.multi_varbit != -1
                || d.multi_varp != -1
                || d.multi_default != -1
        })
    {
        return Err("Unattested native signpost".into());
    }
    let mut output = [0; 4];
    for (i, (alias, child)) in [
        ("aide_west_text_2", 11),
        ("aide_south_text_2", 8),
        ("aide_north_text_2", 2),
        ("aide_east_text_2", 7),
    ]
    .into_iter()
    .enumerate()
    {
        let expected = (135 << 16) | child;
        if components.get(&format!("aide_compass:{alias}")) != Some(&expected) {
            return Err(format!("Unattested signpost component {alias}"));
        }
        output[i] = expected as u32;
    }
    let parent = *components
        .get("toplevel_osrs_stretch:mainmodal")
        .ok_or("Missing main modal parent")?;
    if parent < 0 {
        return Err("Invalid main modal parent".into());
    }
    let table = tables.get(&65533).ok_or("Missing signpost enum")?;
    if table.key_literal != 22
        || table.value_literal != 36
        || table.transmit
        || table.values.len() != 4
    {
        return Err("Unattested signpost enum shape".into());
    }
    let default = directions(
        table
            .default_string
            .as_deref()
            .ok_or("Missing signpost default directions")?,
    )?;
    let mut values = BTreeMap::new();
    for (&key, value) in &table.values {
        let c = CoordGrid::from_packed(key);
        if c == CoordGrid::NULL || CoordGrid::new(c.x(), c.z(), c.level()).is_err() {
            return Err("Invalid signpost enum coordinate".into());
        }
        let Value::String(text) = value else {
            return Err("Invalid signpost direction value".into());
        };
        values.insert(key, directions(text)?);
    }
    // Preflight the exact interface text encoding before configuration publishes anything.
    for texts in values.values().chain(std::iter::once(&default)) {
        for (&id, text) in output.iter().zip(texts) {
            gielinor_protocol::interfaces::text(id, text).map_err(|e| format!("{e:?}"))?;
        }
    }
    Ok(Signposts {
        parent: Component(parent as u32),
        interface: 135,
        components: output,
        values,
        default,
    })
}
