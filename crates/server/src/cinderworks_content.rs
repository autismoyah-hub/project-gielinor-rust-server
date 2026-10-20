//! Exact detached authored-floor integration, adapted from f9180b23.
//! Explicit operator-installed immutable content; no environment-driven mounting.
use crate::cinderworks_contract as contract;
use gielinor_cache::{
    asset_package::{read_pinned, AssetPackage, Namespace},
    game::{Definitions, StaticLocation},
    symbols::Symbols,
    tower_shell::Shell,
};
use gielinor_world::{cinderworks_floor as spatial, collision::CollisionMap};
use std::path::Path;

pub struct Content {
    package: AssetPackage,
    floor: spatial::Floor,
    spatial: spatial::Spatial,
}
impl Content {
    /// Root is art/tower, containing interior-production and floor-handoff.
    /// Companion bytes are pinned to the same handoff before any world/database opens.
    pub fn load(root: &Path) -> Result<Self, String> {
        let package = AssetPackage::load(
            &root.join("floor-handoff/runtime-package"),
            contract::RUNTIME_ID,
            Namespace {
                models: contract::MODELS,
                objects: contract::OBJECTS,
            },
        )
        .map_err(|e| format!("Cinderworks package: {e}"))?;
        let floor = read_pinned(
            &root.join("interior-production/floor.bwr"),
            contract::FLOOR_SHA256,
            2_000_000,
        )
        .map_err(|e| format!("Cinderworks floor: {e}"))?;
        let spatial = read_pinned(
            &root.join("interior-production/spatial-overrides.txt"),
            contract::SPATIAL_SHA256,
            16_384,
        )
        .map_err(|e| format!("Cinderworks spatial: {e}"))?;
        for (name, hash) in [
            ("visual-bindings.txt", contract::VISUAL_SHA256),
            ("cutaway-groups.txt", contract::CUTAWAY_SHA256),
        ] {
            read_pinned(&root.join("interior-production").join(name), hash, 131_072)
                .map_err(|e| format!("Cinderworks {name}: {e}"))?;
        }
        let floor = spatial::parse(std::str::from_utf8(&floor).map_err(|e| e.to_string())?)?;
        let spatial =
            spatial::Spatial::parse(std::str::from_utf8(&spatial).map_err(|e| e.to_string())?)?;
        Ok(Self {
            package,
            floor,
            spatial,
        })
    }
    pub fn package(&self) -> &AssetPackage {
        &self.package
    }
    pub fn floor(&self) -> &spatial::Floor {
        &self.floor
    }
    pub fn spatial(&self) -> &spatial::Spatial {
        &self.spatial
    }
    pub fn prepare(
        &self,
        normal: &CollisionMap,
        locations: &[StaticLocation],
        definitions: &Definitions,
        symbols: &Symbols,
    ) -> Result<Shell, String> {
        let origin = spatial::origin();
        let floor = &self.floor;
        let mut copied = gielinor_cache::constructed::build_small(
            origin,
            &spatial::template(),
            normal,
            locations,
            definitions,
        )?;
        let base = std::mem::take(&mut copied.locations);
        copied.collision = CollisionMap::default();
        for c in &floor.cells {
            copied
                .collision
                .allocate(origin.x() + c.x, origin.z() + c.z, 0);
            copied.collision.set(
                origin.x() + c.x,
                origin.z() + c.z,
                0,
                if c.walk { 0 } else { 0x200000 },
            );
        }
        let mut shell = Shell {
            map: copied,
            removed: base.clone(),
            base,
        };
        let resolve = |name: &str| {
            symbols
                .get(name)
                .and_then(|n| u32::try_from(*n).ok())
                .ok_or_else(|| format!("Missing integration symbol {name}"))
        };
        for (x, z, name) in [
            (53, 60, "aide_bankbooth"),
            (55, 51, "threshold_chest"),
            (60, 52, "altar"),
            (34, 70, "tree"),
            (42, 70, "tree"),
            (34, 76, "tree"),
            (42, 76, "tree"),
            (34, 54, "tinrock1"),
            (34, 58, "tinrock1"),
            (44, 54, "copperrock1"),
            (44, 58, "copperrock1"),
            (34, 50, "tut2_furnace"),
            (37, 50, "anvil"),
            (67, 53, "newbierange"),
            (69, 52, "mistmyst_barrel_water"),
        ] {
            shell.stamp(
                self.spatial
                    .at(origin, x, z)
                    .map_err(|e| format!("{e:?}"))?,
                resolve(name)?,
                definitions,
            )?;
        }
        for (x, z) in spatial::COOLANT_TILES {
            shell.stamp(
                self.spatial.at(origin, x, z).unwrap(),
                resolve("mistmyst_barrel_water")?,
                definitions,
            )?;
        }
        shell.stamp_shaped(
            self.spatial.at(origin, 54, 51).unwrap(),
            resolve("plaque")?,
            // The isolated facility represents this identity as a freestanding
            // lectern on a blocked footing. Shape4 is coordinate-only reach and
            // would require standing inside that footing. Use ordinary rectangular
            // reach for this authored representation; default prototype stays4.
            10,
            1,
            definitions,
        )?;
        shell.stamp_shaped(
            self.spatial.at(origin, 64, 60).unwrap(),
            resolve("threshold_portal")?,
            0,
            0,
            definitions,
        )?;
        for f in gielinor_world::cinderworks_scenery::LANDMARKS {
            shell.stamp_shaped(
                self.spatial.at(origin, f.x, f.z).unwrap(),
                f.dormant,
                10,
                f.angle,
                definitions,
            )?;
        }
        for o in &floor.objects {
            if !definitions.locations.contains_key(&o.id) {
                return Err(format!("Missing original integration definition {}", o.id));
            }
            let at = origin.translate(o.x, o.z, 0).unwrap();
            if shell.map.locations.iter().any(|l| {
                l.coords == at && ((9..=21).contains(&l.shape) == (9..=21).contains(&o.shape))
            }) {
                return Err(format!(
                    "Original scenery {} overwrites a protected anchor at {},{}",
                    o.id, o.x, o.z
                ));
            }
            shell.stamp_shaped(at, o.id, o.shape, o.rotation, definitions)?;
        }
        // Server and native apply solid/decorative policy differently to the bank.
        // Explicitly retain a non-walkable bank footing in shared terrain; do not edit
        // the cache definition or native opcode-74 parity behavior.
        let (x, z) = self.spatial.relocate(53, 60);
        let old = shell.map.collision.get(origin.x() + x, origin.z() + z, 0);
        shell
            .map
            .collision
            .set(origin.x() + x, origin.z() + z, 0, old | 0x200000);
        Ok(shell)
    }
}
