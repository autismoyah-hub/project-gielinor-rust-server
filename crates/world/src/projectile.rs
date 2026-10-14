//! ProjAnim timing and cache-authored trajectory parameters from the Kotlin engine.
//! Distance is supplied by the caller's bounds calculation, not origin distance.
use crate::CoordGrid;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flight {
    pub spotanim: i32,
    pub style: Style,
    pub duration: Duration,
    pub source: EntityIndex,
    pub target: EntityIndex,
    pub start: CoordGrid,
    pub end: CoordGrid,
}
impl Flight {
    pub fn between(
        spotanim: i32,
        style: Style,
        source: EntityIndex,
        target: EntityIndex,
        start: crate::raycast::Bounds,
        end: crate::raycast::Bounds,
    ) -> Result<Self, &'static str> {
        let distance = bounds_distance(start, end)?;
        Ok(Self {
            spotanim,
            style,
            duration: style.duration(distance),
            source,
            target,
            start: start.coords,
            end: end.coords,
        })
    }
}
/// Bounds.distanceTo is plane-sensitive, unlike CoordGrid.chebyshevDistance.
pub fn bounds_distance(
    a: crate::raycast::Bounds,
    b: crate::raycast::Bounds,
) -> Result<i32, &'static str> {
    if a.coords.level() != b.coords.level() {
        return Err("Projectile bounds on different levels");
    }
    if [a.width, a.length, b.width, b.length]
        .iter()
        .any(|v| !(0..=65535).contains(v))
    {
        return Err("Projectile bounds size outside packed range");
    }
    let axis = |origin_a: i32, size_a: i32, origin_b: i32, size_b: i32| {
        // Both original min/max selectors choose b when origins tie.
        let (minimum, size, maximum) = if origin_a < origin_b {
            (origin_a, size_a, origin_b)
        } else if origin_a > origin_b {
            (origin_b, size_b, origin_a)
        } else {
            (origin_b, size_b, origin_b)
        };
        (maximum - (minimum + size - 1)).max(0)
    };
    Ok(axis(a.coords.x(), a.width, b.coords.x(), b.width).max(axis(
        a.coords.z(),
        a.length,
        b.coords.z(),
        b.length,
    )))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub start_height: i32,
    pub end_height: i32,
    pub delay: i32,
    pub angle: i32,
    pub length_adjustment: i32,
    pub progress: i32,
    pub step_multiplier: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Duration {
    pub server_delay: i32,
    pub client_delay: i32,
}
impl Style {
    pub fn duration(self, distance: i32) -> Duration {
        // JVM Int operations wrap; negative division truncates toward zero.
        let end = self
            .delay
            .wrapping_add(self.length_adjustment)
            .wrapping_add(self.step_multiplier.wrapping_mul(distance));
        Duration {
            server_delay: 1 + end / 30,
            client_delay: end,
        }
    }
}
/// The original index convention is shared by every ProjAnim constructor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityIndex {
    None,
    Player(u16),
    Npc(u16),
}
impl EntityIndex {
    pub fn encoded(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Player(slot) => -(i32::from(slot) + 1),
            Self::Npc(slot) => i32::from(slot) + 1,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_timing_uses_entity_edges_and_rejects_other_planes() {
        use crate::raycast::Bounds;
        let a = Bounds {
            coords: CoordGrid::new(3200, 3200, 0).unwrap(),
            width: 3,
            length: 2,
        };
        let b = Bounds {
            coords: CoordGrid::new(3204, 3203, 0).unwrap(),
            width: 1,
            length: 1,
        };
        assert_eq!(bounds_distance(a, b), Ok(2));
        assert_eq!(bounds_distance(b, a), Ok(2));
        assert_eq!(
            bounds_distance(
                a,
                Bounds {
                    coords: CoordGrid::new(3204, 3203, 1).unwrap(),
                    ..b
                }
            ),
            Err("Projectile bounds on different levels")
        );
        assert_eq!(
            bounds_distance(
                a,
                Bounds {
                    coords: a.coords,
                    width: 0,
                    length: 0
                }
            ),
            Ok(1)
        );
    }
    #[test]
    fn flight_uses_full_client_duration_and_truncated_server_cycles() {
        let magic = Style {
            start_height: 172,
            end_height: 124,
            delay: 51,
            angle: 16,
            length_adjustment: -5,
            progress: 64,
            step_multiplier: 10,
        };
        assert_eq!(
            magic.duration(0),
            Duration {
                client_delay: 46,
                server_delay: 2
            }
        );
        assert_eq!(
            magic.duration(2),
            Duration {
                client_delay: 66,
                server_delay: 3
            }
        );
        let negative = Style {
            delay: 0,
            length_adjustment: -29,
            step_multiplier: 0,
            ..magic
        };
        assert_eq!(
            negative.duration(7),
            Duration {
                client_delay: -29,
                server_delay: 1
            }
        );
        assert_eq!(EntityIndex::Player(0).encoded(), -1);
        assert_eq!(EntityIndex::Npc(0).encoded(), 1);
        assert_eq!(EntityIndex::None.encoded(), 0);
    }
}
