//! Iteration order for small Java HashMaps used by ordered gameplay configuration.
//! Callers supply distinct keys in insertion order and their Java hashCode values.
//! Tree bins are rejected explicitly instead of silently changing priority.
pub fn order(hashes: &[i32]) -> Result<Vec<usize>, &'static str> {
    let hashes: Vec<u32> = hashes
        .iter()
        .map(|&h| {
            let h = h as u32;
            h ^ (h >> 16)
        })
        .collect();
    let mut buckets = vec![Vec::new(); 16];
    let resize = |buckets: &mut Vec<Vec<usize>>| {
        let mut next = vec![Vec::new(); buckets.len() * 2];
        let mask = next.len() - 1;
        for bucket in buckets.iter() {
            for &index in bucket {
                next[hashes[index] as usize & mask].push(index);
            }
        }
        *buckets = next;
    };
    for (index, &hash) in hashes.iter().enumerate() {
        let bucket = hash as usize & (buckets.len() - 1);
        buckets[bucket].push(index);
        if buckets[bucket].len() >= 9 {
            if buckets.len() >= 64 {
                return Err("Java HashMap tree-bin configuration is not supported");
            }
            resize(&mut buckets);
        }
        if index + 1 > buckets.len() * 3 / 4 {
            resize(&mut buckets);
        }
    }
    Ok(buckets.into_iter().flatten().collect())
}
