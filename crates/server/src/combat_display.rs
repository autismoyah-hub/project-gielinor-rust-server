//! Translate resolved impacts into observer-specific synchronization data.
use gielinor_protocol::combat_updates::{Headbar, Hitmark};
use gielinor_world::{
    hit_display::{Builder, Mark},
    hits::{Impact, Target},
};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub struct Bar {
    pub id: u16,
    pub segments: u8,
}
pub struct Config {
    pub regular: Builder,
    pub zero: (u16, Option<u16>),
    pub player_bar: Bar,
    pub npc_bars: BTreeMap<u16, Bar>,
}
#[derive(Default)]
pub struct Display {
    pub spot_animations: Vec<gielinor_protocol::spot_animation::Animation>,
    pub sequence: Option<(u16, u8)>,
    pub hits: Vec<Hitmark>,
    pub bars: Vec<Headbar>,
}
impl Config {
    pub fn impact(
        &self,
        npc: bool,
        impact: Impact,
        maximum: i32,
        bar: Bar,
        display: &mut Display,
    ) -> Result<(), String> {
        let mut mark = impact
            .hit
            .display
            .unwrap_or_else(|| self.regular.build(impact.hit.damage));
        // NPC overflow clamps the number but preserves the original highlight unless zero.
        if npc && impact.hit.damage > impact.displayed && impact.displayed == 0 {
            mark = Mark {
                target: self.zero.0,
                source: self.zero.0,
                public: mark.public.and(self.zero.1),
                ..mark
            };
        }
        let source_index = match impact.hit.source {
            Some(Target::Player(id)) => 65536 + id.slot as i32,
            Some(Target::Npc(id)) => id.slot as i32,
            None => -1,
        };
        let fill = gielinor_world::hit_display::fill(bar.segments, impact.health, maximum)
            .ok_or("Zero base health for hit display")?;
        let fill = u8::try_from(fill).map_err(|_| "Healthbar fill outside byte range")?;
        display.hits.push(Hitmark {
            source_index,
            source_type: mark.source,
            self_type: if npc { mark.source } else { mark.target },
            other_type: mark.public.unwrap_or(65535),
            value: impact.displayed as u16,
            delay: mark.delay,
            limit: 4,
        });
        // Kotlin only makes a headbar private for a hit originating from a player.
        let private = matches!(impact.hit.source, Some(Target::Player(_))) && mark.public.is_none();
        display.bars.push(Headbar {
            source_index,
            self_type: bar.id,
            other_type: if private { 65535 } else { bar.id },
            start_fill: fill,
            end_fill: fill,
            start_time: mark.delay,
            end_time: mark.delay,
        });
        Ok(())
    }
}
pub fn resolve(
    names: &crate::startup::Names,
    defs: &gielinor_cache::game::Definitions,
    params: &BTreeMap<u32, gielinor_cache::parameters::ParameterType>,
    bars: &BTreeMap<u32, gielinor_cache::headbars::HeadbarType>,
) -> std::io::Result<Config> {
    let headbar = |id: u32| -> std::io::Result<Bar> {
        Ok(Bar {
            id: u16::try_from(id).map_err(std::io::Error::other)?,
            segments: bars
                .get(&id)
                .ok_or_else(|| std::io::Error::other(format!("Missing headbar {id}")))?
                .segments,
        })
    };
    let zero = (
        names.short("hitmark", "zero_damage_lit")?,
        Some(names.short("hitmark", "zero_damage_tint")?),
    );
    let lit = names.short("hitmark", "regular_damage_lit")?;
    let key = names.id("param", "headbar")?;
    let default = params
        .get(&key)
        .and_then(|p| p.default_int)
        .ok_or_else(|| std::io::Error::other("Missing headbar default"))?;
    let mut npc_bars = BTreeMap::new();
    for (&id, npc) in &defs.npcs {
        let bar = match npc.params.get(&key) {
            Some(gielinor_cache::definitions::Param::Integer(v)) => *v,
            None => default,
            _ => return Err(std::io::Error::other("Invalid NPC headbar param")),
        };
        npc_bars.insert(
            u16::try_from(id).map_err(std::io::Error::other)?,
            headbar(bar as u32)?,
        );
    }
    Ok(Config {
        regular: Builder {
            target: lit,
            source: lit,
            public: Some(names.short("hitmark", "regular_damage_tint")?),
            zero_lit: Some(zero.0),
            zero_tint: zero.1,
            max_lit: Some(names.short("hitmark", "regular_damage_max")?),
            target_threshold: i32::MAX,
            source_threshold: i32::MAX,
            delay: 0,
        },
        zero,
        player_bar: headbar(names.id("headbar", "health_30")?)?,
        npc_bars,
    })
}
