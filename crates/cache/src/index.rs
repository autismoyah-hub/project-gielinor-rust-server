use crate::invalid;
use std::{collections::BTreeMap, io};
#[derive(Debug, Clone)]
pub struct Group {
    pub name_hash: Option<i32>,
    pub checksum: u32,
    pub uncompressed_checksum: Option<u32>,
    pub digest: Option<[u8; 64]>,
    pub lengths: Option<(u32, u32)>,
    pub version: i32,
    pub files: BTreeMap<u32, Option<i32>>,
}
#[derive(Debug, Clone)]
pub struct Index {
    pub protocol: u8,
    pub version: i32,
    pub groups: BTreeMap<u32, Group>,
}
struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| invalid("Index length overflow"))?;
        let bytes = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| invalid("Truncated index"))?;
        self.pos = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn int(&mut self) -> io::Result<u32> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn count(&mut self, smart: bool) -> io::Result<u32> {
        if smart
            && self
                .bytes
                .get(self.pos)
                .ok_or_else(|| invalid("Truncated index"))?
                & 128
                != 0
        {
            Ok(self.int()? & 0x7fffffff)
        } else {
            Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()) as u32)
        }
    }
}
impl Index {
    /// Original OpenRS2 index order, including optional digests, lengths and raw checksums.
    pub fn decode(bytes: &[u8], max_entries: usize) -> io::Result<Self> {
        let mut c = Cursor { bytes, pos: 0 };
        let protocol = c.byte()?;
        if !(5..=7).contains(&protocol) {
            return Err(invalid("Unsupported index protocol"));
        }
        let version = if protocol >= 6 { c.int()? as i32 } else { 0 };
        let flags = c.byte()?;
        if flags & !15 != 0 {
            return Err(invalid("Unknown index flags"));
        }
        let smart = protocol >= 7;
        let count = c.count(smart)? as usize;
        if count > max_entries {
            return Err(invalid("Index group count exceeds limit"));
        }
        let mut groups = BTreeMap::new();
        let mut id = 0u32;
        for ordinal in 0..count {
            let delta = c.count(smart)?;
            if ordinal > 0 && delta == 0 {
                return Err(invalid("Duplicate group id"));
            }
            id = id
                .checked_add(delta)
                .filter(|v| *v <= i32::MAX as u32)
                .ok_or_else(|| invalid("Group id overflow"))?;
            groups.insert(
                id,
                Group {
                    name_hash: None,
                    checksum: 0,
                    uncompressed_checksum: None,
                    digest: None,
                    lengths: None,
                    version: 0,
                    files: BTreeMap::new(),
                },
            );
        }
        if flags & 1 != 0 {
            for group in groups.values_mut() {
                group.name_hash = Some(c.int()? as i32);
            }
        }
        for group in groups.values_mut() {
            group.checksum = c.int()?;
        }
        if flags & 8 != 0 {
            for group in groups.values_mut() {
                group.uncompressed_checksum = Some(c.int()?);
            }
        }
        if flags & 2 != 0 {
            for group in groups.values_mut() {
                group.digest = Some(c.take(64)?.try_into().unwrap());
            }
        }
        if flags & 4 != 0 {
            for group in groups.values_mut() {
                group.lengths = Some((c.int()?, c.int()?));
            }
        }
        for group in groups.values_mut() {
            group.version = c.int()? as i32;
        }
        let mut counts = Vec::with_capacity(count);
        let mut entries = count;
        for _ in 0..count {
            let n = c.count(smart)? as usize;
            entries = entries
                .checked_add(n)
                .filter(|n| *n <= max_entries)
                .ok_or_else(|| invalid("Index file count exceeds limit"))?;
            counts.push(n);
        }
        for (group, count) in groups.values_mut().zip(counts) {
            let mut id = 0u32;
            for ordinal in 0..count {
                let delta = c.count(smart)?;
                if ordinal > 0 && delta == 0 {
                    return Err(invalid("Duplicate file id"));
                }
                id = id
                    .checked_add(delta)
                    .filter(|v| *v <= i32::MAX as u32)
                    .ok_or_else(|| invalid("File id overflow"))?;
                group.files.insert(id, None);
            }
        }
        if flags & 1 != 0 {
            for group in groups.values_mut() {
                for hash in group.files.values_mut() {
                    *hash = Some(c.int()? as i32);
                }
            }
        }
        if c.pos != bytes.len() {
            return Err(invalid("Trailing index bytes"));
        }
        Ok(Self {
            protocol,
            version,
            groups,
        })
    }
    pub fn named_group(&self, hash: i32) -> Option<u32> {
        self.groups
            .iter()
            .find_map(|(id, g)| (g.name_hash == Some(hash)).then_some(*id))
    }
}
