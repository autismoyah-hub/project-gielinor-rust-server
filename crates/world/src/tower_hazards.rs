//! FloorOneRules cinder movement policy, shared by gameplay and future NPC/test agents.
//! These decisions do not themselves move an actor or apply damage.
pub fn arm(safe: bool, boss_alive: bool, hustle: bool) -> bool {
    !safe && (boss_alive || hustle)
}
pub fn erupt(previous_set: bool, still_on_tile: bool) -> bool {
    previous_set && still_on_tile
}
pub fn distance(tile: [i32; 2], npc: [i32; 2], size: i32) -> i32 {
    let axis = |a: i32, b: i32| {
        if a < b {
            b.wrapping_sub(a)
        } else if a >= b.wrapping_add(size) {
            a.wrapping_sub(b.wrapping_add(size).wrapping_sub(1))
        } else {
            0
        }
    };
    axis(tile[0], npc[0]).max(axis(tile[1], npc[1]))
}
pub fn step_off(
    player: [i32; 2],
    ash: Option<[i32; 2]>,
    npc: [i32; 2],
    size: i32,
    reach: i32,
    mut open: impl FnMut([i32; 2]) -> bool,
    mut avoid: impl FnMut([i32; 2]) -> bool,
) -> Option<[i32; 2]> {
    if ash != Some(player) {
        return None;
    }
    let neighbours = [
        [player[0].wrapping_add(1), player[1]],
        [player[0].wrapping_sub(1), player[1]],
        [player[0], player[1].wrapping_add(1)],
        [player[0], player[1].wrapping_sub(1)],
    ]
    .into_iter()
    .filter(|&tile| {
        open(tile)
            && !((npc[0]..npc[0].wrapping_add(size)).contains(&tile[0])
                && (npc[1]..npc[1].wrapping_add(size)).contains(&tile[1]))
    })
    .collect::<Vec<_>>();
    let has_reachable = neighbours
        .iter()
        .any(|&tile| distance(tile, npc, size) <= reach);
    neighbours
        .into_iter()
        .filter(|&tile| !has_reachable || distance(tile, npc, size) <= reach)
        .min_by_key(|&tile| (avoid(tile), distance(tile, npc, size)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cinder_escape_keeps_reach_then_avoids_safe_entrance_then_preserves_tie_order() {
        assert_eq!(
            step_off([5, 5], Some([5, 5]), [7, 5], 1, 1, |_| true, |_| false),
            Some([6, 5])
        );
        assert_eq!(
            step_off(
                [5, 5],
                Some([5, 5]),
                [5, 5],
                1,
                1,
                |_| true,
                |t| t == [6, 5]
            ),
            Some([4, 5])
        );
        assert_eq!(
            step_off([5, 5], Some([5, 5]), [7, 5], 1, 0, |_| true, |_| false),
            Some([6, 5])
        );
        assert_eq!(
            step_off([5, 5], Some([4, 5]), [7, 5], 1, 1, |_| true, |_| false),
            None
        );
    }
}
