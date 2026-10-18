//! Resource preservation for selected bone burial, using the existing staged burial transaction.
use crate::{
    inventory::{Catalog, Inventory, StackType},
    npc::RandomSource,
    prayer::{Completed, Config, Error, Pending},
    simulation::PlayerData,
};
#[derive(Clone, Debug)]
pub struct Rule {
    pub pieces: [(i32, usize); 4],
    pub excluded: std::collections::BTreeSet<i32>,
}
impl Rule {
    /// Numerator over80: one piece contributes1.25%, four contribute5%.
    pub fn numerator(&self, bone: i32, member: bool, worn: &Inventory, items: &Catalog) -> u32 {
        if !member
            || self.excluded.contains(&bone)
            || worn.stack != StackType::Normal
            || worn.placeholders
            || self
                .pieces
                .iter()
                .map(|p| p.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != 4
            || self
                .pieces
                .iter()
                .map(|p| p.1)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != 4
        {
            return 0;
        }
        self.pieces
            .iter()
            .filter(|&&(id, slot)| {
                id >= 0
                    && worn.get(slot).is_some_and(|i| {
                        i.id == id && i.count == 1 && i.vars == 0 && items.ordinary_nonstackable(id)
                    })
            })
            .count() as u32
    }
}
/// Prepare source debit/restore/XP before a saving roll. An invalid action never consumes RNG.
pub fn finish_selected(
    config: &Config,
    player: &mut PlayerData,
    items: &Catalog,
    pending: Pending,
    slot: usize,
    rule: Option<&Rule>,
    member: bool,
    random: &mut impl RandomSource,
) -> Result<Option<Completed>, Error> {
    let numerator = rule.map_or(0, |r| {
        r.numerator(pending.bone, member, &player.equipment, items)
    });
    if numerator == 0 {
        return config.finish_selected(player, items, pending, slot);
    }
    let mut next = player.clone();
    let completed = config.finish_selected(&mut next, items, pending, slot)?;
    if completed.is_some() {
        // Same consume interval as the source: save only at the upper end of the uniform roll.
        if random.below(80) >= 80 - numerator {
            next.inventory = player.inventory.clone();
        }
        *player = next;
    }
    Ok(completed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item, stats::Stats, CoordGrid};
    struct Fixed {
        draw: u32,
        calls: u32,
    }
    impl RandomSource for Fixed {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 80);
            self.calls += 1;
            self.draw
        }
    }
    fn fixture() -> (Config, PlayerData, Rule) {
        let c = Config {
            skill: 5,
            animation: 827,
            bones: [(526, 4.5), (25766, 10.0)].into(),
            necklace: None,
            restore: Default::default(),
            worn_xp_percent: Default::default(),
            chaos: None,
        };
        let mut p = PlayerData::new(
            1,
            "Zealot".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        for slot in [0, 7] {
            p.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 526,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let rule = Rule {
            pieces: [(25438, 0), (25434, 4), (25436, 7), (25440, 10)],
            excluded: [25766, 25769, 25772, 25775, 25778].into(),
        };
        (c, p, rule)
    }
    #[test]
    fn zealot_selected_burial_all_piece_counts_have_exact_save_intervals_without_extra_xp() {
        for pieces in 0..=4 {
            let (c, mut p, rule) = fixture();
            for &(id, slot) in rule.pieces.iter().take(pieces) {
                p.equipment
                    .restore_slot(
                        slot,
                        Some(Item {
                            id,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            let mut saved = 0;
            for draw in 0..80 {
                let mut next = p.clone();
                let mut rng = Fixed { draw, calls: 0 };
                let outcome = finish_selected(
                    &c,
                    &mut next,
                    &Catalog::default(),
                    Pending {
                        bone: 526,
                        restore: 0,
                    },
                    7,
                    Some(&rule),
                    true,
                    &mut rng,
                )
                .unwrap();
                assert!(outcome.is_some());
                assert_eq!(next.stats.fine_xp(5), 45);
                assert_eq!(next.equipment, p.equipment);
                assert_eq!(next.inventory.get(0), p.inventory.get(0));
                let keep = pieces > 0 && draw >= 80 - pieces as u32;
                assert_eq!(next.inventory.get(7).is_some(), keep);
                assert_eq!(rng.calls, u32::from(pieces > 0));
                saved += u32::from(keep);
            }
            assert_eq!(saved, pieces as u32);
        }
    }
    #[test]
    fn zealot_invalid_authority_ashes_and_transaction_failures_never_draw_or_publish_partial_state()
    {
        for mode in 0..9 {
            let (mut c, mut p, rule) = fixture();
            p.equipment
                .restore_slot(
                    0,
                    Some(Item {
                        id: 25438,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            let mut items = Catalog::default();
            let mut pending = Pending {
                bone: 526,
                restore: 0,
            };
            match mode {
                0 => {}
                1 => {
                    p.equipment.restore_slot(0, None).unwrap();
                    p.equipment
                        .restore_slot(
                            1,
                            Some(Item {
                                id: 25438,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                2 => {
                    p.equipment
                        .restore_slot(
                            0,
                            Some(Item {
                                id: 25438,
                                count: 1,
                                vars: 1,
                            }),
                        )
                        .unwrap();
                }
                3 => {
                    items.dummy.insert(25438);
                }
                4 => {
                    pending.bone = 25766;
                    p.inventory
                        .restore_slot(
                            7,
                            Some(Item {
                                id: 25766,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                5 => {
                    p.inventory.restore_slot(7, None).unwrap();
                }
                6 => p.xp_rate_hundreds = 0,
                7 => {
                    c.bones.insert(526, f64::NAN);
                }
                _ => pending.restore = -1,
            }
            let before = p.clone();
            let mut rng = Fixed { draw: 79, calls: 0 };
            let out = finish_selected(
                &c,
                &mut p,
                &items,
                pending,
                7,
                Some(&rule),
                mode != 0,
                &mut rng,
            );
            assert_eq!(rng.calls, 0, "mode {mode}");
            if mode < 5 {
                assert!(out.unwrap().is_some());
                assert!(p.inventory.get(7).is_none());
                assert_eq!(p.stats.fine_xp(5), if mode == 4 { 100 } else { 45 });
            } else {
                assert!(!matches!(out, Ok(Some(_))));
                assert_eq!(p.inventory, before.inventory);
                assert_eq!(p.stats, before.stats);
            }
            assert_eq!(p.equipment, before.equipment);
        }
    }
}
