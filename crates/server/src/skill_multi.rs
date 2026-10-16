//! ProtectedAccess.skillMultiDialog / Player.ifSkillMulti presentation.
//! The host owns modal identity, protected resumption and continuation cancellation.
use gielinor_protocol::{
    buffer::DecodeError,
    client_script::{self, Argument},
    framing::Packet,
    interfaces,
};
use gielinor_world::ui::Component;

pub const MAX_OPTIONS: usize = 18;
pub const MAX_COUNT: i32 = 28;
pub const FIRST_OPTION: u16 = 15;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionItem {
    pub object: i32,
    pub label: String,
}
#[derive(Clone, Debug)]
pub struct Dialog {
    pub title: String,
    pub options: Vec<OptionItem>,
    pub verb: i32,
    pub max_count: i32,
    pub selected_count: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Presentation {
    pub interface: u16,
    pub parent: Component,
    pub setup_script: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub index: usize,
    pub count: i32,
}
#[derive(Clone)]
pub(crate) struct Active {
    pub smelting: bool,
    pub dough: Option<gielinor_world::dough::Request>,
    pub finished_potion: Option<gielinor_world::finished_potions::Request>,
    pub unfinished_potion: Option<gielinor_world::unfinished_potions::Request>,
    pub token: u64,
    pub presentation: Presentation,
    pub dialog: Dialog,
}
impl Dialog {
    pub fn validate(&self) -> Result<(), DecodeError> {
        if self.options.is_empty() || self.options.len() > MAX_OPTIONS {
            return Err(DecodeError::InvalidValue("skillmulti options count"));
        }
        Ok(())
    }
    /// 21 integers then the joined title/labels. Count clamps are independent;
    /// selectedCount is not clamped to the configured All-button maxCount.
    pub fn setup_arguments(&self) -> Result<Vec<Argument>, DecodeError> {
        self.validate()?;
        let mut args = vec![
            Argument::Int(self.verb),
            Argument::Int(self.max_count.clamp(1, MAX_COUNT)),
        ];
        for slot in 0..MAX_OPTIONS {
            args.push(Argument::Int(
                self.options.get(slot).map_or(-1, |option| option.object),
            ));
        }
        args.push(Argument::Int(self.selected_count.clamp(1, MAX_COUNT)));
        let joined = std::iter::once(self.title.as_str())
            .chain(self.options.iter().map(|option| option.label.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        args.push(Argument::String(joined));
        Ok(args)
    }
    /// After the host has validated permissions, retained modal access and
    /// called ifClose, a foreign pause component returns null, not an option.
    pub fn select(
        &self,
        presentation: Presentation,
        component: Component,
        count: i32,
    ) -> Option<Selection> {
        if component.interface() != presentation.interface {
            return None;
        }
        let first = Component::new(presentation.interface, FIRST_OPTION).0;
        let index = component.0.checked_sub(first)? as usize;
        (index < self.options.len()).then(|| Selection {
            index,
            count: count.clamp(1, MAX_COUNT),
        })
    }
}
impl Presentation {
    /// Open through Runtime::open_interface so authoritative UI state and close
    /// hooks are updated. These are the ordered setup packets after that open.
    pub fn setup_packets(&self, dialog: &Dialog) -> Result<Vec<Packet>, DecodeError> {
        let mut packets = vec![client_script::run(
            self.setup_script,
            &dialog.setup_arguments()?,
        )?];
        for slot in 0..dialog.options.len() {
            packets.push(interfaces::events(
                Component::new(self.interface, FIRST_OPTION + slot as u16).0,
                0,
                MAX_COUNT,
                1,
            )?);
        }
        Ok(packets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dialog() -> Dialog {
        Dialog {
            title: "Smelt".into(),
            options: vec![
                OptionItem {
                    object: 2349,
                    label: "Bronze".into(),
                },
                OptionItem {
                    object: 2351,
                    label: "Iron".into(),
                },
            ],
            verb: 13,
            max_count: 14,
            selected_count: 3,
        }
    }
    fn presentation() -> Presentation {
        Presentation {
            interface: 270,
            parent: Component::new(162, 0),
            setup_script: 2046,
        }
    }
    #[test]
    fn quantity_is_carried_by_subcomponent_and_foreign_slots_return_none() {
        let d = dialog();
        let p = presentation();
        assert_eq!(
            d.select(p, Component::new(270, 16), 7),
            Some(Selection { index: 1, count: 7 })
        );
        assert_eq!(
            d.select(p, Component::new(270, 15), 0),
            Some(Selection { index: 0, count: 1 })
        );
        assert_eq!(
            d.select(p, Component::new(270, 15), i32::MAX)
                .unwrap()
                .count,
            28
        );
        for component in [
            Component::new(270, 14),
            Component::new(270, 17),
            Component::new(271, 15),
        ] {
            assert_eq!(d.select(p, component, 1), None);
        }
    }
    #[test]
    fn setup_layout_and_filled_slot_permissions_match_kotlin_contract() {
        let d = dialog();
        let p = presentation();
        let args = d.setup_arguments().unwrap();
        assert_eq!(args.len(), 22);
        assert_eq!(
            &args[..4],
            &[
                Argument::Int(13),
                Argument::Int(14),
                Argument::Int(2349),
                Argument::Int(2351)
            ]
        );
        assert!(args[4..20].iter().all(|v| *v == Argument::Int(-1)));
        assert_eq!(args[20], Argument::Int(3));
        assert_eq!(args[21], Argument::String("Smelt|Bronze|Iron".into()));
        let packets = p.setup_packets(&d).unwrap();
        assert_eq!(packets.len(), 3);
        assert_eq!(
            packets[1],
            interfaces::events(Component::new(270, 15).0, 0, 28, 1).unwrap()
        );
        assert_eq!(
            packets[2],
            interfaces::events(Component::new(270, 16).0, 0, 28, 1).unwrap()
        );
    }
    #[test]
    fn option_bounds_and_independent_count_clamps() {
        let mut d = dialog();
        d.max_count = -1;
        d.selected_count = 29;
        let args = d.setup_arguments().unwrap();
        assert_eq!(args[1], Argument::Int(1));
        assert_eq!(args[20], Argument::Int(28));
        d.options.clear();
        assert!(d.setup_arguments().is_err());
        d.options.resize(
            18,
            OptionItem {
                object: 2349,
                label: "Bronze".into(),
            },
        );
        assert_eq!(presentation().setup_packets(&d).unwrap().len(), 19);
        assert_eq!(
            d.select(presentation(), Component::new(270, 32), 28)
                .unwrap()
                .index,
            17
        );
        d.options.push(OptionItem {
            object: 2351,
            label: "Iron".into(),
        });
        assert!(d.setup_arguments().is_err());
    }
}
