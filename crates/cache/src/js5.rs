//! Read-only JS5 provider. Compressed/encrypted groups are served without decoding.
use crate::{container, crc32, disk::DiskStore, index::Index, invalid};
use std::{io, path::Path};
pub struct Store {
    disk: DiskStore,
    master: Vec<u8>,
    pub checksums: [u32; 23],
    limit: usize,
}
impl Store {
    pub fn open(root: &Path, limit: usize) -> io::Result<Self> {
        let mut disk = DiskStore::open(root, limit)?;
        // VERSIONED master index contains checksum and version pairs up to the
        // final archive, including zero pairs for gaps. Archive 255 is excluded.
        let count = disk
            .groups(255, 256)?
            .last()
            .map_or(0, |id| *id as usize + 1);
        let mut entries = Vec::with_capacity(count * 8);
        let mut checksums = [0; 23];
        for id in 0..count {
            let (checksum, version) = if let Some(bytes) = disk.read(255, id as u32)? {
                let decoded = container::decode(&bytes, None, limit)?;
                let index = Index::decode(&decoded, 1_000_000)?;
                (crc32(&bytes), index.version)
            } else {
                (0, 0)
            };
            if id < checksums.len() {
                checksums[id] = checksum;
            }
            entries.extend(checksum.to_be_bytes());
            entries.extend(version.to_be_bytes());
        }
        let mut master = vec![0];
        master.extend((entries.len() as u32).to_be_bytes());
        master.extend(entries);
        if master.len() > limit {
            return Err(invalid("JS5 master index exceeds group limit"));
        }
        Ok(Self {
            disk,
            master,
            checksums,
            limit,
        })
    }
    /// Unframed container, preserving index bytes and stripping only group trailers.
    pub fn container(&mut self, archive: u8, group: u16) -> io::Result<Vec<u8>> {
        if archive == 255 && group == 255 {
            return Ok(self.master.clone());
        }
        let mut bytes = self
            .disk
            .read(archive, group as u32)?
            .ok_or_else(|| invalid("JS5 group missing"))?;
        if archive != 255 {
            // OpenRS2 VersionTrailer.strip removes the two-byte version; it does
            // not attempt to decompress or decrypt the container.
            if bytes.len() >= 2 {
                bytes.truncate(bytes.len() - 2);
            }
        }
        if bytes.len() > self.limit {
            return Err(invalid("JS5 response exceeds limit"));
        }
        Ok(bytes)
    }
    pub fn response(&mut self, archive: u8, group: u16) -> io::Result<Vec<u8>> {
        Ok(gielinor_protocol::js5::response(
            archive,
            group,
            &self.container(archive, group)?,
        ))
    }
}
