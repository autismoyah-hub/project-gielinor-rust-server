//! RegionRotations: footprints are adjusted for the loc's original angle by the caller.
pub fn translate(
    rotation: u8,
    x: i32,
    z: i32,
    grid_width: i32,
    grid_length: i32,
    width: i32,
    length: i32,
) -> Result<(i32, i32), &'static str> {
    Ok(match rotation {
        0 => (x, z),
        1 => (z, grid_width - x - width),
        2 => (grid_width - x - width, grid_length - z - length),
        3 => (grid_length - z - length, x),
        _ => return Err("Region rotation must be in 0..4"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oversized_loc_can_shift_into_the_previous_zone() {
        assert_eq!(translate(1, 7, 2, 8, 8, 3, 1), Ok((2, -2)));
        assert_eq!(translate(3, 4, 7, 8, 8, 1, 4), Ok((-3, 4)));
        assert!(translate(4, 0, 0, 8, 8, 1, 1).is_err());
    }
}
