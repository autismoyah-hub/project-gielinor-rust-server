const DELTA: u32 = 0x9E3779B9;
/// Matches RSProt: only complete 8-byte blocks are transformed; a tail is preserved.
pub fn encrypt(bytes: &mut [u8], key: [u32; 4]) {
    crypt(bytes, key, false);
}
pub fn decrypt(bytes: &mut [u8], key: [u32; 4]) {
    crypt(bytes, key, true);
}
fn crypt(bytes: &mut [u8], key: [u32; 4], decrypt: bool) {
    for block in bytes.chunks_exact_mut(8) {
        let mut a = u32::from_be_bytes(block[..4].try_into().unwrap());
        let mut b = u32::from_be_bytes(block[4..].try_into().unwrap());
        let mut sum = if decrypt { DELTA.wrapping_mul(32) } else { 0 };
        for _ in 0..32 {
            if decrypt {
                b = b.wrapping_sub(
                    ((a << 4 ^ a >> 5).wrapping_add(a))
                        ^ sum.wrapping_add(key[((sum >> 11) & 3) as usize]),
                );
                sum = sum.wrapping_sub(DELTA);
                a = a.wrapping_sub(
                    ((b << 4 ^ b >> 5).wrapping_add(b)) ^ sum.wrapping_add(key[(sum & 3) as usize]),
                );
            } else {
                a = a.wrapping_add(
                    ((b << 4 ^ b >> 5).wrapping_add(b)) ^ sum.wrapping_add(key[(sum & 3) as usize]),
                );
                sum = sum.wrapping_add(DELTA);
                b = b.wrapping_add(
                    ((a << 4 ^ a >> 5).wrapping_add(a))
                        ^ sum.wrapping_add(key[((sum >> 11) & 3) as usize]),
                );
            }
        }
        block[..4].copy_from_slice(&a.to_be_bytes());
        block[4..].copy_from_slice(&b.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_vector_and_tail() {
        let mut block = [0; 9];
        block[8] = 239;
        encrypt(&mut block, [0; 4]);
        assert_eq!(block, [0xDE, 0xE9, 0xD4, 0xD8, 0xF7, 0x13, 0x1E, 0xD9, 239]);
        decrypt(&mut block, [0; 4]);
        assert_eq!(block, [0, 0, 0, 0, 0, 0, 0, 0, 239]);
    }
}
