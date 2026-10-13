//! Resolve the music subsystem from actual DB data and cache variable metadata.
use crate::startup::Names;
use gielinor_cache::{
    game::{Definitions, GameCache},
    music::Input,
};
use gielinor_world::{bank_actions::Binding, music::Event, music_player::Config};
use std::io;
pub fn resolve(cache: &mut GameCache, names: &Names, defs: &Definitions) -> io::Result<Config> {
    let varp = |name: &str| -> io::Result<_> {
        defs.varps
            .get(&names.id("varp", name)?)
            .copied()
            .ok_or_else(|| io::Error::other(format!("Missing music varp {name}")))
    };
    let bit = |name: &str| -> io::Result<_> {
        let bit = defs
            .varbits
            .get(&names.id("varbit", name)?)
            .copied()
            .ok_or_else(|| io::Error::other(format!("Missing music varbit {name}")))?;
        let base = defs
            .varps
            .get(&(bit.base as u32))
            .copied()
            .ok_or_else(|| io::Error::other(format!("Missing music varbit base {name}")))?;
        bit.replace(0, 0)
            .map_err(|e| io::Error::other(format!("Invalid music binding: {e:?}")))?;
        Ok(Binding { base, bit })
    };
    let rows = cache.db_rows()?;
    let tables = cache.db_tables()?;
    let area_ids = cache.area_ids()?;
    let unlock_varps = (1..=25)
        .map(|i| varp(&format!("musicmulti_{i}")))
        .collect::<io::Result<Vec<_>>>()?;
    let catalog = Input {
        rows: &rows,
        tables: &tables,
        table_names: names.group("dbtable"),
        columns: names.group("dbcol"),
        area_ids: &area_ids,
        unlock_varps: &unlock_varps,
    }
    .resolve()
    .map_err(io::Error::other)?;
    Ok(Config {
        catalog,
        mode: varp("musicplay")?,
        playlist: varp("music_playlist")?,
        volume: varp("option_music")?,
        area_mode: bit("music_area_mode")?,
        area: bit("music_curr_area")?,
        current: bit("music_curr_id")?,
        last: bit("music_last_id")?,
        clocks: bit("music_curr_clocks")?,
        duration: bit("music_curr_duration")?,
        suppress_message: bit("music_unlock_text_toggle")?,
        transparency: bit("chatbox_transparency")?,
        stop_midi: names.id("midi", "stop_music")? as i32,
        timer: names.short("timer", "music_sync")?,
    })
}
pub fn packets(
    events: &[Event],
    component: u32,
) -> Result<Vec<gielinor_protocol::framing::Packet>, gielinor_protocol::buffer::DecodeError> {
    events
        .iter()
        .map(|event| {
            Ok(match event {
                Event::Variable(update) => gielinor_protocol::varp::update(update.id, update.value),
                Event::Message(text) => gielinor_protocol::chat::game(0, None, text)?,
                Event::NowPlaying(text) => gielinor_protocol::interfaces::text(component, text)?,
                Event::Midi {
                    id,
                    fade_out_speed,
                    fade_in_delay,
                } => gielinor_protocol::music::song(
                    u16::try_from(*id).map_err(|_| {
                        gielinor_protocol::buffer::DecodeError::InvalidValue("music id")
                    })?,
                    0,
                    *fade_out_speed,
                    *fade_in_delay,
                    0,
                ),
            })
        })
        .collect()
}
