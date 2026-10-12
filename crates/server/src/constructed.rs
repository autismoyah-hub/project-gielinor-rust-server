//! Validated, immutable constructed-region presentation maps.
use gielinor_protocol::{
    buffer::DecodeError,
    framing::Packet,
    region_rebuild::{self, Zone},
};
use gielinor_world::{tower_layout::Template, CoordGrid};

pub struct Region {
    origin: CoordGrid,
    template: Template,
}
impl Region {
    pub fn small(origin: CoordGrid, template: Template) -> Result<Self, &'static str> {
        if origin.level() != 0
            || origin.x() % 8 != 0
            || origin.z() % 8 != 0
            || origin.x() < 48
            || origin.z() < 48
            || origin.x() + 176 > 16384
            || origin.z() + 176 > 16384
        {
            return Err("Invalid padded constructed region origin");
        }
        if template.iter().any(|(&(x, z, level), s)| {
            x >= 16
                || z >= 16
                || level > 3
                || s.x > 1023
                || s.z > 2047
                || s.level > 3
                || s.rotation > 3
        }) {
            return Err("Invalid constructed region template");
        }
        Ok(Self { origin, template })
    }
    pub fn contains(&self, coords: CoordGrid) -> bool {
        (0..128).contains(&(coords.x() - self.origin.x()))
            && (0..128).contains(&(coords.z() - self.origin.z()))
    }
    fn packet(&self, coords: CoordGrid) -> Result<Packet, DecodeError> {
        region_rebuild::encode(
            (coords.x() >> 3) as u16,
            (coords.z() >> 3) as u16,
            true,
            |x, z, level| {
                let x = u8::try_from(x - (self.origin.x() >> 3)).ok()?;
                let z = u8::try_from(z - (self.origin.z() >> 3)).ok()?;
                let s = self.template.get(&(x, z, level))?;
                Some(Zone {
                    x: s.x,
                    z: s.z,
                    level: s.level,
                    rotation: s.rotation,
                })
            },
        )
    }
}
#[derive(Default)]
pub struct Regions(Vec<Region>);
impl Regions {
    pub fn new(regions: Vec<Region>) -> Result<Self, &'static str> {
        for (index, a) in regions.iter().enumerate() {
            for b in &regions[..index] {
                if (a.origin.x() - b.origin.x()).abs() < 128
                    && (a.origin.z() - b.origin.z()).abs() < 128
                {
                    return Err("Overlapping constructed regions");
                }
            }
        }
        Ok(Self(regions))
    }
    pub fn rebuild(&self, coords: CoordGrid) -> Result<Option<Packet>, DecodeError> {
        self.0
            .iter()
            .find(|region| region.contains(coords))
            .map(|region| region.packet(coords))
            .transpose()
    }
    /// Login still starts with the GPI-bearing normal rebuild. Follow it with the
    /// instance template before any zone changes, even when already inside a region.
    pub fn login(&self, coords: CoordGrid, packets: &mut Vec<Packet>) -> Result<(), DecodeError> {
        if let Some(packet) = self.rebuild(coords)? {
            packets.push(packet);
        }
        Ok(())
    }
    pub fn replace_rebuilds(
        &self,
        coords: CoordGrid,
        packets: &mut [Packet],
    ) -> Result<(), DecodeError> {
        for packet in packets.iter_mut().filter(|p| p.opcode == 49) {
            if let Some(rebuild) = self.rebuild(coords)? {
                *packet = rebuild;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_gpi_is_preserved_and_region_template_precedes_world_updates() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let regions = Regions::new(vec![Region::small(
            origin,
            gielinor_world::tower_layout::template(),
        )
        .unwrap()])
        .unwrap();
        let hub = origin.translate(56, 56, 0).unwrap();
        let mut login = vec![Packet {
            opcode: 49,
            payload: vec![1, 2, 3],
        }];
        regions.login(hub, &mut login).unwrap();
        assert_eq!(
            login.iter().map(|p| p.opcode).collect::<Vec<_>>(),
            vec![49, 125]
        );
        assert_eq!(login[0].payload, vec![1, 2, 3]);
        let mut cycle = vec![
            Packet {
                opcode: 49,
                payload: vec![0; 6],
            },
            Packet {
                opcode: 28,
                payload: vec![],
            },
        ];
        regions.replace_rebuilds(hub, &mut cycle).unwrap();
        assert_eq!(
            cycle.iter().map(|p| p.opcode).collect::<Vec<_>>(),
            vec![125, 28]
        );
        assert!(regions
            .rebuild(CoordGrid::new(3222, 3218, 0).unwrap())
            .unwrap()
            .is_none());
        assert!(regions
            .rebuild(origin.translate(128, 0, 0).unwrap())
            .unwrap()
            .is_none());
    }
}
