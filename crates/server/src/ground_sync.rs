use gielinor_protocol::{
    buffer::DecodeError,
    framing::Packet,
    zone::{self, Add, Object},
};
use gielinor_world::{
    zones::{Emission, Update},
    CoordGrid,
};
pub fn encode(base: CoordGrid, emissions: &[Emission]) -> Result<Vec<Packet>, DecodeError> {
    let mut packets = Vec::new();
    for emission in emissions {
        let coords = match emission {
            Emission::Reset(c) => *c,
            Emission::Enclosed { base, .. } => *base,
        };
        let x = u8::try_from(coords.x() - base.x())
            .map_err(|_| DecodeError::InvalidValue("zone x outside build area"))?;
        let z = u8::try_from(coords.z() - base.z())
            .map_err(|_| DecodeError::InvalidValue("zone z outside build area"))?;
        let level = coords.level() as u8;
        match emission {
            Emission::Reset(_) => packets.push(zone::reset(x, z, level)),
            Emission::Enclosed { updates, .. } => {
                let objects = updates
                    .iter()
                    .map(|update| {
                        if let Update::Graphic(v)=update{return Object::Graphic{id:v.id,height:v.height,delay:v.delay,x:(v.coords.x()&7)as u8,z:(v.coords.z()&7)as u8};}
                        if let Update::Sound(sound) = update {
                            return Object::SoundArea(zone::SoundArea {
                                id: sound.id as u16,
                                delay: sound.delay as u8,
                                loops: sound.loops as u8,
                                range: sound.radius as u8,
                                drop_off_range: sound.size as u8,
                                x: (sound.coords.x() & 7) as u8,
                                z: (sound.coords.z() & 7) as u8,
                            });
                        }
                        if let Update::Projectile(flight) = update {
                            return projectile(*flight);
                        }
                        if let Update::Location(event) = update {
                            return location(*event);
                        }
                        let obj = match update {
                            Update::Add(obj)
                            | Update::Delete(obj)
                            | Update::Count { object: obj, .. } => obj,
                            Update::Location(_) | Update::Projectile(_) | Update::Sound(_) | Update::Graphic(_) => {
                                unreachable!()
                            }
                        };
                        let (x, z) = ((obj.coords.x() & 7) as u8, (obj.coords.z() & 7) as u8);
                        match update {
                            Update::Add(_) => Object::Add(Add::visible(obj.item, obj.count, x, z)),
                            Update::Delete(_) => Object::Delete {
                                id: obj.item,
                                count: obj.count,
                                x,
                                z,
                            },
                            Update::Count { old, .. } => Object::Count {
                                id: obj.item,
                                old: *old,
                                new: obj.count,
                                x,
                                z,
                            },
                            Update::Location(_) | Update::Projectile(_) | Update::Sound(_) | Update::Graphic(_) => {
                                unreachable!()
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                if !objects.is_empty() {
                    packets.push(zone::enclosed(x, z, level, &objects)?);
                }
            }
        }
    }
    Ok(packets)
}
pub fn location(event: gielinor_world::locations::Event) -> Object {
    use gielinor_world::locations::Event;
    let loc = event.location();
    let (x, z) = ((loc.coords.x() & 7) as u8, (loc.coords.z() & 7) as u8);
    match event {
        Event::Add(_) => Object::LocAdd {
            id: loc.id as u16,
            x,
            z,
            shape: loc.shape,
            angle: loc.angle,
            ops: 255,
        },
        Event::Delete(_) => Object::LocDelete {
            x,
            z,
            shape: loc.shape,
            angle: loc.angle,
        },
        Event::Animate { sequence, .. } => Object::LocAnimate {
            sequence,
            x,
            z,
            shape: loc.shape,
            angle: loc.angle,
        },
    }
}
pub fn projectile(flight: gielinor_world::projectile::Flight) -> Object {
    Object::Projectile(zone::Projectile {
        id: flight.spotanim as u16,
        start_height: flight.style.start_height as u16,
        end_height: flight.style.end_height as u16,
        start_time: flight.style.delay as u16,
        end_time: flight.duration.client_delay as u16,
        angle: flight.style.angle as u8,
        progress: flight.style.progress as u16,
        source_index: flight.source.encoded(),
        target_index: flight.target.encoded(),
        x: (flight.start.x() & 7) as u8,
        z: (flight.start.z() & 7) as u8,
        end: flight.end.packed(),
    })
}
