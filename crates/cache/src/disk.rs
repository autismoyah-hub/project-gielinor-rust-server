use crate::{invalid, u24};
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub struct DiskStore {
    root: PathBuf,
    data: File,
    max_group_bytes: usize,
}
impl DiskStore {
    /// Enumerate populated index records without opening or modifying groups.
    pub fn groups(&self, archive: u8, limit: usize) -> io::Result<Vec<u32>> {
        let mut index = File::open(self.root.join(format!("main_file_cache.idx{archive}")))?;
        let count = index.metadata()?.len() / 6;
        if count > limit as u64 {
            return Err(invalid("Disk index exceeds group limit"));
        }
        let mut groups = Vec::new();
        for group in 0..count {
            let mut entry = [0; 6];
            index.read_exact(&mut entry)?;
            if entry != [0; 6] {
                groups.push(group as u32);
            }
        }
        Ok(groups)
    }
    pub fn open(root: &Path, max_group_bytes: usize) -> io::Result<Self> {
        if max_group_bytes == 0 || max_group_bytes > 0xff_ffff {
            return Err(invalid("Invalid disk group limit"));
        }
        Ok(Self {
            root: root.to_owned(),
            data: File::open(root.join("main_file_cache.dat2"))?,
            max_group_bytes,
        })
    }
    pub fn read(&mut self, archive: u8, group: u32) -> io::Result<Option<Vec<u8>>> {
        let mut index = File::open(self.root.join(format!("main_file_cache.idx{archive}")))?;
        let offset = group as u64 * 6;
        if index.metadata()?.len() < offset + 6 {
            return Ok(None);
        }
        index.seek(SeekFrom::Start(offset))?;
        let mut entry = [0; 6];
        index.read_exact(&mut entry)?;
        let length = u24(&entry[..3]) as usize;
        let mut sector = u24(&entry[3..]);
        if length == 0 && sector == 0 {
            return Ok(None);
        }
        if length > self.max_group_bytes {
            return Err(invalid("Disk group exceeds size limit"));
        }
        let mut result = Vec::with_capacity(length);
        let mut chunk = 0u16;
        let mut seen = std::collections::HashSet::new();
        while result.len() < length {
            if sector == 0 || !seen.insert(sector) {
                return Err(invalid("Invalid or cyclic sector chain"));
            }
            self.data.seek(SeekFrom::Start(sector as u64 * 520))?;
            let large = group > u16::MAX as u32;
            let header_len = if large { 10 } else { 8 };
            let mut header = [0; 10];
            self.data.read_exact(&mut header[..header_len])?;
            let (stored_group, pos) = if large {
                (u32::from_be_bytes(header[..4].try_into().unwrap()), 4)
            } else {
                (
                    u16::from_be_bytes(header[..2].try_into().unwrap()) as u32,
                    2,
                )
            };
            if stored_group != group
                || u16::from_be_bytes(header[pos..pos + 2].try_into().unwrap()) != chunk
                || header[pos + 5] != archive
            {
                return Err(invalid("Sector identity mismatch"));
            }
            let next = u24(&header[pos + 2..pos + 5]);
            let count = (length - result.len()).min(520 - header_len);
            let start = result.len();
            result.resize(start + count, 0);
            self.data.read_exact(&mut result[start..])?;
            if result.len() == length && next != 0 {
                return Err(invalid("Trailing sector chain"));
            }
            sector = next;
            chunk = chunk
                .checked_add(1)
                .ok_or_else(|| invalid("Too many sectors"))?;
        }
        Ok(Some(result))
    }
}
