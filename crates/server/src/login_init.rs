//! LoginScript variable initialization, including preservation of explicitly muted accounts.
use gielinor_cache::{game::Definitions, symbols::Symbols};
use gielinor_protocol::{framing::Packet, varp};
use gielinor_world::{
    bank_actions::Binding,
    variables::{Transmit, Variables, Varp},
};
pub struct Config {
    pub chatbox: Binding,
    pub roofs: Binding,
    pub volumes: [Varp; 4],
    /// MusicAreaScript resets even a saved manual/random mode to Area on login.
    pub music_mode: Option<Varp>,
    pub transmit: Vec<Varp>,
}
pub struct Presentation {
    pub variables: Config,
    pub frames: gielinor_world::gameframe::Frames,
    pub welcome: Option<String>,
    pub broadcast: Option<String>,
}
impl Config {
    /// High priority packets end at the EngineLoginUi dispatch boundary.
    pub fn high_priority(
        &self,
        vars: &mut Variables,
        display_name: &str,
        processed_clock: i32,
        welcome: Option<&str>,
        broadcast: Option<&str>,
    ) -> Result<Vec<Packet>, String> {
        self.high_priority_for_content(vars,display_name,processed_clock,welcome,broadcast,false)
    }
    pub fn high_priority_for_content(&self,vars:&mut Variables,display_name:&str,processed_clock:i32,
        welcome:Option<&str>,broadcast:Option<&str>,tower:bool)->Result<Vec<Packet>,String> {
        use gielinor_protocol::{
            client_script::{self, Argument},
            login_init as init,
        };
        let mut packets = vec![
            init::chat_filters(0, 0),
            init::hide_operations(init::Operations::Npc, false),
            init::hide_operations(init::Operations::Location, false),
            init::hide_operations(init::Operations::Object, false),
        ];
        for (kind, message) in [(108, welcome), (14, broadcast)] {
            if let Some(message) = message {
                packets.push(
                    gielinor_protocol::chat::game(kind, None, message)
                        .map_err(|e| format!("{e:?}"))?,
                );
            }
        }
        packets.extend(
            self.variables_for_content(vars, display_name, processed_clock,tower)
                .map_err(|e| format!("{e:?}"))?,
        );
        packets.push(init::camera_reset());
        packets.push(
            client_script::run(605, &[128, 896, 128, 896].map(Argument::Int))
                .map_err(|e| format!("{e:?}"))?,
        );
        packets.push(
            client_script::run(42, &[512, 512].map(Argument::Int)).map_err(|e| format!("{e:?}"))?,
        );
        Ok(packets)
    }
    pub fn resolve(varps: &Symbols, bits: &Symbols, defs: &Definitions) -> Result<Self, String> {
        let varp = |name: &str| {
            let id = *varps
                .get(name)
                .ok_or_else(|| format!("Missing login varp {name}"))? as u32;
            defs.varps
                .get(&id)
                .copied()
                .ok_or_else(|| format!("Unknown login varp {id}"))
        };
        let binding = |name: &str| -> Result<Binding, String> {
            let id = *bits
                .get(name)
                .ok_or_else(|| format!("Missing login varbit {name}"))? as u32;
            let bit = *defs
                .varbits
                .get(&id)
                .ok_or_else(|| format!("Unknown login varbit {id}"))?;
            let base = *defs
                .varps
                .get(&(bit.base as u32))
                .ok_or("Missing login base varp")?;
            bit.replace(0, 1).map_err(|e| format!("{e:?}"))?;
            Ok(Binding { base, bit })
        };
        Ok(Self {
            chatbox: binding("has_displayname_transmitter")?,
            roofs: binding("option_hide_rooftops")?,
            volumes: [
                varp("option_master_volume")?,
                varp("option_music")?,
                varp("option_sounds")?,
                varp("option_areasounds")?,
            ],
            music_mode: Some(varp("musicplay")?),
            transmit: defs
                .varps
                .values()
                .filter(|ty| ty.transmit != Transmit::Never)
                .copied()
                .collect(),
        })
    }
    pub fn variables(
        &self,
        vars: &mut Variables,
        display_name: &str,
        processed_clock: i32,
    ) -> Result<Vec<Packet>, gielinor_world::variables::Error> {
        self.variables_for_content(vars,display_name,processed_clock,false)
    }
    pub(crate) fn variables_for_content(&self,vars:&mut Variables,display_name:&str,processed_clock:i32,tower:bool)->Result<Vec<Packet>,gielinor_world::variables::Error>{
        let mut packets = vec![varp::reset()];
        if let Some(mode) = self.music_mode {
            vars.set(mode, 1, false);
        }
        let logged_in = processed_clock > 0;
        let nonblank = display_name
            .chars()
            .any(|c| !(c.is_whitespace() && c != '\u{85}' || ('\u{1c}'..='\u{1f}').contains(&c)));
        for (binding, value) in [(self.chatbox, nonblank as i32), (self.roofs, 1)] {
            if let Some(update) = vars.set_bit(binding.base, binding.bit, value, logged_in)? {
                packets.push(varp::update(update.id, update.value));
            }
        }
        for (ty, value) in self.volumes.into_iter().zip([100, 20, 45, 25]) {
            if !vars.contains(ty.id) {
                if let Some(update) = vars.set(ty, value, logged_in) {
                    packets.push(varp::update(update.id, update.value));
                }
            }
        }
        let mut transmit = self.transmit.clone();
        transmit.sort_by_key(|ty| ty.id);
        for ty in transmit {
            if ty.transmit != Transmit::Never && (vars.contains(ty.id) || (tower && ty.client_code==22)) {
                packets.push(varp::update(ty.id, vars.get(ty.id)));
            }
        }
        Ok(packets)
    }
}
/// Run after gameframe initialization and before initial stat/run packets.
pub fn low_priority(
    clock: i32,
    display_name: &str,
) -> Result<Vec<Packet>, gielinor_protocol::buffer::DecodeError> {
    use gielinor_protocol::{
        client_script::{self, Argument},
        login_init as init,
    };
    let mut packets = vec![
        client_script::run(2498, &[1, 0, 0].map(Argument::Int))?,
        client_script::run(828, &[Argument::Int(1)])?,
        client_script::run(5141, &[])?,
    ];
    for (slot, op) in [
        (2, None),
        (3, Some("Follow")),
        (4, Some("Trade with")),
        (5, None),
        (8, Some("Report")),
    ] {
        packets.push(init::player_operation(slot, false, op)?);
    }
    packets.push(client_script::run(
        876,
        &[
            Argument::Int(clock),
            Argument::Int(0),
            Argument::String(display_name.into()),
            Argument::String("REGULAR".into()),
        ],
    )?);
    Ok(packets)
}
pub fn tail() -> [Packet; 2] {
    [
        gielinor_protocol::login_init::reset_animations(),
        gielinor_protocol::login_init::minimap(0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tower_login_sends_absent_attack_zero_and_preserves_explicit_hidden(){
        let ty=|id|Varp{id,permanent:true,transmit:Transmit::Always,bit_protect:false,client_code:if id==1306{22}else{-1}};
        let binding=Binding{base:ty(40),bit:gielinor_world::variables::Varbit{base:40,lsb:0,msb:0}};
        let config=Config{chatbox:binding,roofs:binding,volumes:[ty(41),ty(42),ty(43),ty(44)],music_mode:None,transmit:vec![ty(1306)]};
        for tower in [false,true]{for previous in [None,Some(0),Some(3)]{
            let mut vars=Variables::default();if let Some(v)=previous{vars.set(ty(1306),v,false);}
            let packets=config.variables_for_content(&mut vars,"Test",-1,tower).unwrap();
            let expected=varp::update(1306,previous.unwrap_or(0));
            let count=packets.iter().filter(|p|p.opcode==expected.opcode && p.payload==expected.payload).count();
            assert_eq!(count,usize::from(tower || previous.is_some()));
            assert_eq!(vars.contains(1306),previous.is_some(),"Synchronization must not invent a persisted preference");
        }}
    }

    #[test]
    fn area_music_login_overrides_saved_mode_and_transmits_once_after_reset() {
        let ty = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Different,
            bit_protect: false,
            client_code: -1,
        };
        let binding = Binding {
            base: ty(40),
            bit: gielinor_world::variables::Varbit {
                base: 40,
                lsb: 0,
                msb: 0,
            },
        };
        let config = Config {
            chatbox: binding,
            roofs: binding,
            volumes: [ty(41), ty(42), ty(43), ty(44)],
            music_mode: Some(ty(18)),
            transmit: vec![ty(18)],
        };
        for previous in [None, Some(0), Some(1), Some(2)] {
            let mut vars = Variables::default();
            if let Some(value) = previous {
                vars.set(ty(18), value, false);
            }
            let packets = config.variables(&mut vars, "Music", -1).unwrap();
            assert_eq!(vars.get(18), 1);
            assert_eq!(packets.len(), 2);
            let expected = varp::update(18, 1);
            assert_eq!(
                (packets[1].opcode, &packets[1].payload),
                (expected.opcode, &expected.payload)
            );
        }
    }
}
