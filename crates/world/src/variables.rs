use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transmit {
    Never,
    Different,
    Always,
}
#[derive(Clone, Copy, Debug)]
pub struct Varp {
    pub id: u16,
    pub permanent: bool,
    pub transmit: Transmit,
    pub bit_protect: bool,
    pub client_code: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Varbit {
    pub base: u16,
    pub lsb: u8,
    pub msb: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    BitRange,
    ValueOutOfRange,
    WrongBase,
}
impl Varbit {
    fn mask(self) -> Result<u32, Error> {
        if self.lsb > self.msb || self.msb > 31 {
            return Err(Error::BitRange);
        }
        Ok(((1u64 << (self.msb - self.lsb + 1)) - 1) as u32)
    }
    pub fn extract(self, base: i32) -> Result<i32, Error> {
        let mask = self.mask()?;
        Ok(((base as u32 >> self.lsb) & mask) as i32)
    }
    pub fn replace(self, base: i32, value: i32) -> Result<i32, Error> {
        let mask = self.mask()?;
        if value < 0 || value as u32 > mask {
            return Err(Error::ValueOutOfRange);
        }
        Ok(((base as u32 & !(mask << self.lsb)) | ((value as u32) << self.lsb)) as i32)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Update {
    pub id: u16,
    pub value: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Variables {
    values: BTreeMap<u16, i32>,
}
impl Variables {
    pub fn get(&self, id: u16) -> i32 {
        self.values.get(&id).copied().unwrap_or(0)
    }
    pub fn contains(&self, id: u16) -> bool {
        self.values.contains_key(&id)
    }
    pub fn restore(values: BTreeMap<u16, i32>) -> Self {
        Self { values }
    }
    /// Return each immediate transmission, including repeated writes for Always vars.
    pub fn set(&mut self, ty: Varp, value: i32, engine_logged_in: bool) -> Option<Update> {
        let previous = self.get(ty.id);
        self.values.insert(ty.id, value);
        (engine_logged_in
            && (ty.transmit == Transmit::Always
                || (ty.transmit == Transmit::Different && previous != value)))
            .then_some(Update { id: ty.id, value })
    }
    pub fn bit(&self, ty: Varbit) -> Result<i32, Error> {
        ty.extract(self.get(ty.base))
    }
    pub fn set_bit(
        &mut self,
        base: Varp,
        bit: Varbit,
        value: i32,
        engine_logged_in: bool,
    ) -> Result<Option<Update>, Error> {
        if base.id != bit.base {
            return Err(Error::WrongBase);
        }
        let packed = bit.replace(self.get(base.id), value)?;
        Ok(self.set(base, packed, engine_logged_in))
    }
    pub fn persistent(&self, types: &BTreeMap<u16, Varp>) -> BTreeMap<u16, i32> {
        self.values
            .iter()
            .filter(|(id, _)| types.get(id).is_some_and(|ty| ty.permanent))
            .map(|(id, value)| (*id, *value))
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn immediate_sync_preserves_repeated_writes_and_persistent_zeroes() {
        let mut vars = Variables::default();
        let base = Varp {
            id: 7,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        assert_eq!(vars.set(base, 0, false), None);
        assert!(vars.contains(7));
        assert_eq!(vars.set(base, 0, true), Some(Update { id: 7, value: 0 }));
        let mut different = base;
        different.transmit = Transmit::Different;
        assert_eq!(vars.set(different, 0, true), None);
        let bit = Varbit {
            base: 7,
            lsb: 28,
            msb: 31,
        };
        vars.set_bit(base, bit, 15, true).unwrap();
        assert_eq!(vars.bit(bit), Ok(15));
        let old = vars.get(7);
        assert_eq!(
            vars.set_bit(base, bit, 16, true),
            Err(Error::ValueOutOfRange)
        );
        assert_eq!(vars.get(7), old);
        assert_eq!(vars.persistent(&BTreeMap::from([(7, base)]))[&7], old);
    }
}
