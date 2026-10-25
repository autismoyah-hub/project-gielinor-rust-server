//! Revision 239 Huffman (`lz`), including its symbol-order code assignment.
//! Shared with rust-client/src/huffman.rs; the cache table is binary archive 10 "huffman".
//! The assignment differs from sorting symbols by code length first.
use crate::buffer::DecodeError;
type Result<T> = std::result::Result<T, DecodeError>;

#[derive(Clone, Default)]
struct Node {
    children: [Option<usize>; 2],
    symbol: Option<u8>,
}

pub struct Huffman {
    lengths: [u8; 256],
    codes: [u32; 256],
    tree: Vec<Node>,
}
impl Huffman {
    pub fn new(lengths: &[u8]) -> Result<Self> {
        if lengths.len() != 256 {
            return Err(DecodeError::InvalidValue("Huffman alphabet must contain 256 lengths"));
        }
        let mut codec = Self {
            lengths: lengths.try_into().unwrap(),
            codes: [0; 256],
            tree: vec![Node::default()],
        };
        let mut next = [0u32; 33];
        for (symbol, &len) in lengths.iter().enumerate() {
            if len > 32 {
                return Err(DecodeError::InvalidValue("Huffman code exceeds 32 bits"));
            }
            if len == 0 {
                continue;
            }
            let len = len as usize;
            let bit = 1u32 << (32 - len);
            let code = next[len];
            codec.codes[symbol] = code;
            let replacement = if code & bit != 0 {
                next[len - 1]
            } else {
                let value = code | bit;
                for shorter in (1..len).rev() {
                    let previous = next[shorter];
                    if previous != code {
                        break;
                    }
                    let mask = 1u32 << (32 - shorter);
                    if previous & mask != 0 {
                        next[shorter] = next[shorter - 1];
                        break;
                    }
                    next[shorter] = previous | mask;
                }
                value
            };
            next[len] = replacement;
            for value in &mut next[len + 1..] {
                if *value == code {
                    *value = replacement;
                }
            }
            let mut node = 0;
            for depth in 0..len {
                if codec.tree[node].symbol.is_some() {
                    return Err(DecodeError::InvalidValue("Huffman prefix collision"));
                }
                let branch = ((code >> (31 - depth)) & 1) as usize;
                node = match codec.tree[node].children[branch] {
                    Some(child) => child,
                    None => {
                        let child = codec.tree.len();
                        codec.tree.push(Node::default());
                        codec.tree[node].children[branch] = Some(child);
                        child
                    }
                };
            }
            if codec.tree[node].symbol.is_some()
                || codec.tree[node].children.iter().any(Option::is_some)
            {
                return Err(DecodeError::InvalidValue("Huffman code collision"));
            }
            codec.tree[node].symbol = Some(symbol as u8);
        }
        Ok(codec)
    }
    pub fn encode(&self, input: &[u8]) -> Result<Vec<u8>> {
        let bits = input.iter().try_fold(0usize, |sum, &symbol| {
            let length = self.lengths[symbol as usize] as usize;
            if length == 0 {
                return Err(DecodeError::InvalidValue("symbol has no Huffman code"));
            }
            sum.checked_add(length)
                .ok_or(DecodeError::InvalidValue("Huffman output overflow"))
        })?;
        let mut result = vec![0u8; bits.div_ceil(8)];
        let mut pos = 0;
        for &symbol in input {
            for depth in 0..self.lengths[symbol as usize] {
                let bit = (self.codes[symbol as usize] >> (31 - depth)) & 1;
                result[pos / 8] |= (bit as u8) << (7 - pos % 8);
                pos += 1;
            }
        }
        Ok(result)
    }
    /// The packet carries the decoded byte count; returns whole input bytes used.
    pub fn decode(
        &self,
        input: &[u8],
        output_len: usize,
        max_output: usize,
    ) -> Result<(Vec<u8>, usize)> {
        if output_len > max_output {
            return Err(DecodeError::InvalidValue("Huffman output limit"));
        }
        if output_len == 0 {
            return Ok((Vec::new(), 0));
        }
        let mut result = Vec::with_capacity(output_len.min(4096));
        let mut node = 0;
        for (index, &byte) in input.iter().enumerate() {
            for bit in (0..8).rev() {
                let branch = ((byte >> bit) & 1) as usize;
                node = self.tree[node].children[branch]
                    .ok_or(DecodeError::InvalidValue("invalid Huffman prefix"))?;
                if let Some(symbol) = self.tree[node].symbol {
                    result.push(symbol);
                    node = 0;
                    if result.len() == output_len {
                        return Ok((result, index + 1));
                    }
                }
            }
        }
        Err(DecodeError::Truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn symbol_order_is_not_sorted_canonical_order() {
        let mut lens = [0; 256];
        lens[..4].copy_from_slice(&[2, 1, 3, 3]);
        let h = Huffman::new(&lens).unwrap();
        assert_eq!(h.encode(&[0, 1, 2, 3]).unwrap(), [0b00101001, 0b10000000]);
        assert_eq!(
            h.decode(&[0b00101001, 0b10000000], 4, 4).unwrap(),
            (vec![0, 1, 2, 3], 2)
        );
    }
    #[test]
    fn invalid_tables_and_truncation() {
        let mut lens = [0; 256];
        lens[..3].fill(1);
        assert!(Huffman::new(&lens).is_err());
        let h = Huffman::new(&[8; 256]).unwrap();
        assert_eq!(h.encode(b"test").unwrap(), b"test");
        assert_eq!(h.decode(b"abc", 4, 4), Err(DecodeError::Truncated));
        assert!(h.decode(b"abc", 3, 2).is_err());
        let empty = Huffman::new(&[0; 256]).unwrap();
        assert!(empty.encode(b"a").is_err());
    }
}
