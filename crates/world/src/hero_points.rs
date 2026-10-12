//! Bounded damage contributions and the reference's alternating-quicksort tie order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hero {
    pub uuid: u64,
    pub points: i32,
}
#[derive(Debug, Clone)]
pub struct HeroPoints {
    capacity: usize,
    entries: Vec<Hero>,
}
impl HeroPoints {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Vec::new(),
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn entries(&self) -> &[Hero] {
        &self.entries
    }
    /// PathingEntity.heroPoints ignores nonpositive contributions before calling HeroPoints.add.
    pub fn credit(&mut self, uuid: u64, points: i32) {
        if points <= 0 {
            return;
        }
        if let Some(hero) = self.entries.iter_mut().find(|hero| hero.uuid == uuid) {
            hero.points = hero.points.saturating_add(points);
        } else if self.entries.len() < self.capacity {
            self.entries.push(Hero { uuid, points });
        }
    }
    pub fn winner(&self) -> Option<u64> {
        fn sort(entries: &mut [Hero], low: usize, high: usize) {
            if low >= high {
                return;
            }
            let pivot = entries[(low + high) / 2];
            entries.swap((low + high) / 2, high);
            let mut counter = low;
            for index in low..high {
                if pivot.points - entries[index].points < (index & 1) as i32 {
                    entries.swap(index, counter);
                    counter += 1;
                }
            }
            entries.swap(high, counter);
            if counter > low + 1 {
                sort(entries, low, counter - 1);
            }
            if counter + 1 < high {
                sort(entries, counter + 1, high);
            }
        }
        let mut entries = self.entries.clone();
        let last = entries.len().checked_sub(1)?;
        sort(&mut entries, 0, last);
        Some(entries[0].uuid)
    }
}
impl Default for HeroPoints {
    fn default() -> Self {
        Self::new(16)
    }
}
