#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stats {
    fine_xp: [i32; 256],
    base: [i8; 256],
    current: [i8; 256],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidXp;
impl Default for Stats {
    fn default() -> Self {
        Self {
            fine_xp: [0; 256],
            base: [1; 256],
            current: [1; 256],
        }
    }
}
impl Stats {
    pub const MAX_XP: i32 = 200_000_000;
    pub const MAX_FINE_XP: i32 = 2_000_000_000;
    pub fn xp(&self, skill: u8) -> i32 {
        self.fine_xp(skill) / 10
    }
    pub fn fine_xp(&self, skill: u8) -> i32 {
        self.fine_xp[skill as usize]
    }
    pub fn set_fine_xp(&mut self, skill: u8, xp: i32) -> Result<(), InvalidXp> {
        if !(0..=Self::MAX_FINE_XP).contains(&xp) {
            return Err(InvalidXp);
        }
        self.fine_xp[skill as usize] = xp;
        Ok(())
    }
    pub fn set_xp(&mut self, skill: u8, xp: i32) -> Result<(), InvalidXp> {
        self.set_fine_xp(skill, xp.wrapping_mul(10))
    }
    pub fn base(&self, skill: u8) -> i8 {
        self.base[skill as usize]
    }
    pub fn current(&self, skill: u8) -> i8 {
        self.current[skill as usize]
    }
    pub fn set_base(&mut self, skill: u8, value: i8) {
        self.base[skill as usize] = value;
    }
    pub fn set_current(&mut self, skill: u8, value: i8) {
        self.current[skill as usize] = value;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fine_precision_caps_and_defaults() {
        let mut s = Stats::default();
        assert_eq!((s.xp(0), s.base(0), s.current(0)), (0, 1, 1));
        s.set_fine_xp(0, 159).unwrap();
        assert_eq!(s.xp(0), 15);
        assert_eq!(s.set_fine_xp(0, -1), Err(InvalidXp));
        s.set_xp(0, 200_000_000).unwrap();
        assert_eq!(s.fine_xp(0), 2_000_000_000);
    }
}
