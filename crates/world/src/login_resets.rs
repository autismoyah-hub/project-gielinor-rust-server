//! QuickPrayerScript.disableQuickPrayers and CanoeLogin.resetCanoeVars.
use crate::{
    bank_actions::Binding,
    variables::{self, Variables},
};

#[derive(Clone, Debug)]
pub struct Config {
    pub quick_prayer: Binding,
    pub canoe_station: Binding,
    pub canoe_type: Binding,
    pub canoe_avoid_interface: Binding,
}
impl Config {
    /// Initial variables are sent after admission. Keep explicit zero entries,
    /// just as VarPlayerIntMapSetter does, while preserving neighboring bits.
    pub fn apply(&self, vars: &mut Variables) -> Result<(), variables::Error> {
        for binding in [
            self.quick_prayer,
            self.canoe_station,
            self.canoe_type,
            self.canoe_avoid_interface,
        ] {
            vars.set_bit(binding.base, binding.bit, 0, false)?;
        }
        Ok(())
    }
}
