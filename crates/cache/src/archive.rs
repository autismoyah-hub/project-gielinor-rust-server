use crate::{container, crc32, disk::DiskStore, index::Index, invalid};
use std::{collections::BTreeMap, io};

/// Split the interleaved stripes of a decompressed multi-file JS5 group.
pub fn split(bytes: &[u8], file_ids: &[u32], limit: usize) -> io::Result<BTreeMap<u32, Vec<u8>>> {
    if bytes.len() > limit {
        return Err(invalid("Group exceeds split limit"));
    }
    if file_ids.windows(2).any(|w| w[0] >= w[1]) {
        return Err(invalid("File ids must be ascending"));
    }
    if file_ids.len() == 1 {
        return Ok(BTreeMap::from([(file_ids[0], bytes.to_vec())]));
    }
    if file_ids.is_empty() {
        return if bytes.is_empty() {
            Ok(BTreeMap::new())
        } else {
            Err(invalid("Nonempty group without files"))
        };
    }
    let stripes = *bytes
        .last()
        .ok_or_else(|| invalid("Missing stripe count"))? as usize;
    if stripes == 0 {
        return Err(invalid("Zero stripe count"));
    }
    let table_bytes = stripes
        .checked_mul(file_ids.len())
        .and_then(|v| v.checked_mul(4))
        .ok_or_else(|| invalid("Stripe table overflow"))?;
    let table_start = bytes
        .len()
        .checked_sub(table_bytes + 1)
        .ok_or_else(|| invalid("Truncated stripe table"))?;
    let mut sizes = vec![0usize; file_ids.len()];
    let mut chunks = Vec::with_capacity(table_bytes / 4);
    let mut total = 0usize;
    for stripe in bytes[table_start..bytes.len() - 1].chunks_exact(file_ids.len() * 4) {
        let mut length = 0i64;
        for (index, delta) in stripe.chunks_exact(4).enumerate() {
            length += i32::from_be_bytes(delta.try_into().unwrap()) as i64;
            if length < 0 {
                return Err(invalid("Negative stripe length"));
            }
            let n = length as usize;
            total = total
                .checked_add(n)
                .filter(|v| *v <= table_start)
                .ok_or_else(|| invalid("Stripe exceeds data"))?;
            sizes[index] = sizes[index]
                .checked_add(n)
                .ok_or_else(|| invalid("File length overflow"))?;
            chunks.push(n);
        }
    }
    if total != table_start {
        return Err(invalid("Unconsumed stripe data"));
    }
    let mut outputs: Vec<Vec<u8>> = sizes.into_iter().map(Vec::with_capacity).collect();
    let mut pos = 0;
    for (i, n) in chunks.into_iter().enumerate() {
        outputs[i % file_ids.len()].extend_from_slice(&bytes[pos..pos + n]);
        pos += n;
    }
    Ok(file_ids.iter().copied().zip(outputs).collect())
}

pub struct Archive {
    pub id: u8,
    pub index: Index,
    pub index_checksum: u32,
    limit: usize,
}
impl Archive {
    pub fn load(store: &mut DiskStore, id: u8, limit: usize) -> io::Result<Self> {
        let encoded = store
            .read(255, id as u32)?
            .ok_or_else(|| invalid("Missing archive index"))?;
        let index_checksum = crc32(&encoded);
        let index = Index::decode(&container::decode(&encoded, None, limit)?, 1_000_000)?;
        Ok(Self {
            id,
            index,
            index_checksum,
            limit,
        })
    }
    pub fn files(
        &self,
        store: &mut DiskStore,
        group: u32,
        key: Option<[u32; 4]>,
    ) -> io::Result<Option<BTreeMap<u32, Vec<u8>>>> {
        let Some(meta) = self.index.groups.get(&group) else {
            return Ok(None);
        };
        let encoded = store
            .read(self.id, group)?
            .ok_or_else(|| invalid("Indexed group missing from disk"))?;
        if encoded.len() < 7 {
            return Err(invalid("Truncated versioned group"));
        }
        let end = encoded.len() - 2;
        if u16::from_be_bytes(encoded[end..].try_into().unwrap()) != meta.version as u16 {
            return Err(invalid("Group version mismatch"));
        }
        if crc32(&encoded[..end]) != meta.checksum {
            return Err(invalid("Group checksum mismatch"));
        }
        let decoded = container::decode(&encoded, key, self.limit)?;
        if meta
            .uncompressed_checksum
            .is_some_and(|c| c != crc32(&decoded))
        {
            return Err(invalid("Uncompressed group checksum mismatch"));
        }
        split(
            &decoded,
            &meta.files.keys().copied().collect::<Vec<_>>(),
            self.limit,
        )
        .map(Some)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interleaved_stripes_accept_negative_deltas() {
        let mut bytes = b"abcXYdefgZ".to_vec();
        for n in [3i32, -1, 4, -3] {
            bytes.extend(n.to_be_bytes())
        }
        bytes.push(2);
        let result = split(&bytes, &[0, 9], 100).unwrap();
        assert_eq!(result[&0], b"abcdefg");
        assert_eq!(result[&9], b"XYZ");
        bytes[10..14].copy_from_slice(&100i32.to_be_bytes());
        assert!(split(&bytes, &[0, 9], 100).is_err());
    }
}
