// A mapped static worn component/current item remains authoritative.
pub(super) fn worn_item_claim_matches(
    button: &gielinor_protocol::buttons::Button,
    item: i32,
) -> bool {
    button.slot == -1 && (button.object == -1 || button.object == item)
}

pub(super) fn boost_action(
    button: &gielinor_protocol::buttons::Button,
    slot: usize,
    item: i32,
    rules: &std::collections::BTreeMap<i32, gielinor_world::worn_stat_boost::Rule>,
) -> Option<gielinor_world::simulation::Action> {
    let rule = rules.get(&item)?;
    (rule.slot == slot
        && rule.button_operation == button.op
        && worn_item_claim_matches(button, item))
    .then_some(gielinor_world::simulation::Action::BoostWornItem { slot, item })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_protocol::buttons::{button, Button};
    use gielinor_world::{simulation::Action, worn_stat_boost::Rule};
    #[test]
    fn worn_cape_boost_button_decoder_claims_and_operation_do_not_alias_remove_or_teleport() {
        let rules = [(
            9780,
            Rule {
                slot: 1,
                skill: 12,
                button_operation: 2,
                members: true,
            },
        )]
        .into();
        for object in [-1, 9780] {
            let mut packet = ((387u32 << 16) | 16).to_be_bytes().to_vec();
            packet.extend(u16::MAX.to_be_bytes());
            packet.extend((if object < 0 { u16::MAX } else { object as u16 }).to_be_bytes());
            packet.push(2);
            let b = button(&packet).unwrap();
            assert_eq!(b.component, (387 << 16) | 16);
            assert!(matches!(
                boost_action(&b, 1, 9780, &rules),
                Some(Action::BoostWornItem {
                    slot: 1,
                    item: 9780
                })
            ));
            for bad in [
                Button { op: 1, ..b },
                Button { op: 3, ..b },
                Button { slot: 0, ..b },
                Button { object: 9781, ..b },
            ] {
                assert!(boost_action(&bad, 1, 9780, &rules).is_none());
            }
            assert!(boost_action(&b, 2, 9780, &rules).is_none());
            assert!(boost_action(&b, 1, 9781, &rules).is_none());
        }
    }
}
