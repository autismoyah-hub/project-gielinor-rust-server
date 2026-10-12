//! FloorOneRules diary transitions. Gameplay callbacks and durable storage are
//! separate: marking a bounty paid here never inserts its items into a container.
pub const CAPS: [i32; 6] = [5, 5, 3, 5, 5, 15];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marks {
    /// Tans, offerings, bows, cooked shrimp, crafted minds, fletched arrows.
    pub counts: [i32; 6],
    /// Crafting, smithing, firemaking levels.
    pub levels: [i32; 3],
    /// Vambrace crafted, scimitar smithed, mind cast, guam, newt, potion, arrow hit.
    pub actions: [bool; 7],
    /// Paid flags for the eleven diary rows, in display order.
    pub done: [bool; 11],
    pub pending: [bool; 2],
    pub bounty: i32,
    pub schema11: bool,
    pub imp_bones: bool,
    pub staff_taken: bool,
}
impl Default for Marks {
    fn default() -> Self {
        Self {
            counts: [0; 6],
            levels: [1; 3],
            actions: [false; 7],
            done: [false; 11],
            pending: [false; 2],
            bounty: 0,
            schema11: false,
            imp_bones: false,
            staff_taken: false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowState {
    Done,
    Owed,
    Now,
    Ahead,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BossCredit {
    pub marks: Marks,
    pub credited: bool,
    pub banked: bool,
    pub quiet_imp: bool,
    pub drop_imp_bones: bool,
    pub warden_first_loot: bool,
}
impl Marks {
    pub fn is_done(self, row: i32) -> bool {
        (1..=11).contains(&row) && self.done[(row - 1) as usize]
    }
    pub fn ready(self, row: i32) -> bool {
        match row {
            1 => self.counts[0] >= 5,
            2 => self.counts[1] >= 5,
            3 => self.levels[0] >= 11 && self.actions[0],
            4 => self.levels[1] >= 5 && self.actions[1],
            5 => self.counts[2] >= 3 && self.levels[2] >= 10,
            6 => self.counts[3] >= 5,
            7 => self.counts[4] >= 5 && self.actions[2],
            8 => self.actions[3] && self.actions[4] && self.actions[5],
            9 => self.counts[5] >= 15 && self.actions[6],
            10 => self.pending[0],
            11 => self.pending[1],
            _ => false,
        }
    }
    pub fn sync(mut self) -> Self {
        if !self.schema11 {
            if self.done[0] {
                self.done[1] = true;
            } else if self.counts[0] >= 5 && self.bounty == 0 {
                self.bounty = 1;
            }
            self.schema11 = true;
        }
        for (count, cap) in self.counts.iter_mut().zip(CAPS) {
            *count = (*count).clamp(0, cap);
        }
        let Some(row) = (1..=11).find(|&row| !self.is_done(row)) else {
            self.bounty = 0;
            return self;
        };
        if self.bounty != 0 && self.bounty != row {
            self.bounty = 0;
        }
        if self.ready(row) {
            self.bounty = row;
        } else if self.bounty == row {
            self.bounty = 0;
        }
        self
    }
    pub fn state(self, row: i32) -> RowState {
        // Kotlin intentionally skips resynchronization for an already migrated
        // value. Callers that need fresh readiness use sync first.
        let m = if self.schema11 { self } else { self.sync() };
        if m.is_done(row) {
            RowState::Done
        } else if m.bounty == row {
            RowState::Owed
        } else if Some(row) == (1..=11).find(|&r| !m.is_done(r)) && m.bounty == 0 {
            RowState::Now
        } else {
            RowState::Ahead
        }
    }
    pub fn counts_for(self, row: i32) -> bool {
        let m = self.sync();
        !m.is_done(row) && matches!(m.state(row), RowState::Now | RowState::Owed)
    }
    /// Return the original value unchanged when the event is ineligible, as Kotlin does.
    pub fn add(mut self, row: i32, amount: i32) -> Self {
        let index = match row {
            1 => 0,
            2 => 1,
            5 => 2,
            6 => 3,
            7 => 4,
            9 => 5,
            _ => return self,
        };
        if amount > 0 && self.counts_for(row) {
            self.counts[index] = self.counts[index]
                .wrapping_add(amount)
                .clamp(0, CAPS[index]);
        }
        self
    }
    pub fn note(mut self, action: usize) -> Self {
        let row = match action {
            0 => 3,
            1 => 4,
            2 => 7,
            3..=5 => 8,
            6 => 9,
            _ => return self,
        };
        if !self.counts_for(row)
            || (action == 2 && self.counts[4] < 5)
            || (action == 5 && (!self.actions[3] || !self.actions[4]))
            || (action == 6 && self.counts[5] < 15)
        {
            return self;
        }
        self.actions[action] = true;
        self
    }
    /// Caller must successfully transact chest items before committing this state.
    pub fn pay(self) -> Self {
        let mut m = self.sync();
        if (1..=11).contains(&m.bounty) {
            let row = m.bounty;
            m.done[(row - 1) as usize] = true;
            m.bounty = 0;
            if row >= 10 {
                m.pending[(row - 10) as usize] = false;
            }
            m = m.sync();
        }
        m
    }
    pub fn credit_boss(self, row: i32, armed: bool) -> Result<BossCredit, &'static str> {
        if row != 10 && row != 11 {
            return Err("Boss diary row must be 10 or 11");
        }
        let synced = self.sync();
        let mut result = BossCredit {
            marks: synced,
            credited: armed,
            banked: false,
            quiet_imp: false,
            drop_imp_bones: false,
            warden_first_loot: false,
        };
        if !armed || synced.is_done(row) {
            return Ok(result);
        }
        let ahead = synced.state(row) == RowState::Ahead;
        let mut pending = synced;
        pending.pending[(row - 10) as usize] = true;
        let mut next = pending.sync();
        result.warden_first_loot =
            row == 11 && !synced.pending[1] && synced.bounty != 11 && !synced.done[10];
        result.quiet_imp = row == 10 && next.bounty == 10 && synced.bounty != 10;
        result.drop_imp_bones = result.quiet_imp && !synced.imp_bones;
        if result.drop_imp_bones {
            next.imp_bones = true;
        }
        result.banked = ahead && next.bounty != row;
        result.marks = next;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_runs_once_and_chest_payment_does_not_skip_new_offering_row() {
        let old = Marks {
            done: [
                true, false, false, false, false, false, false, false, false, false, false,
            ],
            ..Marks::default()
        }
        .sync();
        assert!(old.done[1]);
        let fresh = Marks {
            counts: [5, 0, 0, 0, 0, 0],
            schema11: true,
            ..Marks::default()
        }
        .pay();
        assert!(fresh.done[0]);
        assert!(!fresh.done[1]);
        assert_eq!(fresh.state(2), RowState::Now);
        assert_eq!(fresh.sync(), fresh);
    }
    #[test]
    fn early_boss_credit_banks_once_and_imp_bones_do_not_duplicate() {
        let credit = Marks::default().credit_boss(11, true).unwrap();
        assert!(credit.banked && credit.warden_first_loot);
        assert!(
            !credit
                .marks
                .credit_boss(11, true)
                .unwrap()
                .warden_first_loot
        );
        let mut m = Marks {
            schema11: true,
            ..Marks::default()
        };
        m.done[..9].fill(true);
        let credit = m.credit_boss(10, true).unwrap();
        assert!(credit.quiet_imp && credit.drop_imp_bones);
        assert_eq!(credit.marks.bounty, 10);
        assert!(!credit.marks.credit_boss(10, true).unwrap().drop_imp_bones);
        let paid = credit.marks.pay();
        assert!(paid.done[9]);
        assert!(!paid.pending[0]);
    }
}
