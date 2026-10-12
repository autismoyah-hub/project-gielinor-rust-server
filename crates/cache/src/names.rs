//! Archive 24 game-value names, matching GameValDecoder's three file layouts.
use gielinor_protocol::buffer::{DecodeError, Reader};
use std::collections::BTreeMap;
#[derive(Default)]
pub struct Names {
    pub groups: BTreeMap<u32, BTreeMap<u32, String>>,
    pub components: BTreeMap<u32, String>,
    pub columns: BTreeMap<u32, String>,
}
impl Names {
    pub fn group(&mut self, group: u32, files: &BTreeMap<u32, Vec<u8>>) -> Result<(), DecodeError> {
        let mut names = BTreeMap::new();
        for (&id, bytes) in files {
            let mut reader = Reader::new(bytes);
            let name = match group {
                10 => {
                    reader.u8()?;
                    let name = reader.string()?;
                    let mut column = 0u32;
                    while reader.remaining() > 0 {
                        if reader.u8()? == 0 {
                            break;
                        }
                        self.columns.insert((id << 16) | column, reader.string()?);
                        column += 1;
                    }
                    name
                }
                14 => {
                    let name = reader.string()?;
                    while reader.remaining() > 0 {
                        let component = reader.u16()?;
                        if component == 65535 {
                            break;
                        }
                        self.components
                            .insert((id << 16) | component as u32, reader.string()?);
                    }
                    name
                }
                _ => String::from_utf8_lossy(bytes).into_owned(),
            };
            names.insert(id, name);
        }
        self.groups.insert(group, names);
        Ok(())
    }
    pub fn id(&self, group: u32, name: &str) -> Result<u32, &'static str> {
        let values = self.groups.get(&group).ok_or("Missing game-value group")?;
        let mut matches = values.iter().filter(|(_, value)| value.as_str() == name);
        let id = *matches.next().ok_or("Unknown game-value name")?.0;
        if matches.next().is_some() {
            return Err("Ambiguous game-value name");
        }
        Ok(id)
    }
    pub fn human_animations(&self) -> Result<[u16; 7], &'static str> {
        let mut result = [0; 7];
        for (index, name) in [
            "human_ready",
            "human_turnonspot",
            "human_walk_f",
            "human_walk_b",
            "human_walk_l",
            "human_walk_r",
            "human_running",
        ]
        .into_iter()
        .enumerate()
        {
            result[index] = u16::try_from(self.id(7, name)?)
                .map_err(|_| "Animation ID exceeds protocol range")?;
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_utf8_interface_components_and_table_columns() {
        let mut names = Names::default();
        names
            .group(7, &BTreeMap::from([(2, "ready_€".as_bytes().to_vec())]))
            .unwrap();
        assert_eq!(names.id(7, "ready_€"), Ok(2));
        names
            .group(
                14,
                &BTreeMap::from([(3, b"inventory\0\0\x04items\0\xff\xff".to_vec())]),
            )
            .unwrap();
        assert_eq!(names.components[&0x30004], "items");
        names
            .group(
                10,
                &BTreeMap::from([(5, b"\x01table\0\x01first\0\x01second\0\0".to_vec())]),
            )
            .unwrap();
        assert_eq!(names.columns[&0x50001], "second");
        assert_eq!(names.id(10, "table"), Ok(5));
    }
}
