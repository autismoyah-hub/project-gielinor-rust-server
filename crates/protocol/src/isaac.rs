//! ISAAC-32 opcode stream. This is not a source of login secrets or cryptographic entropy.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedTooLong;

pub struct Isaac {
    memory: [u32; 256],
    results: [u32; 256],
    a: u32,
    b: u32,
    c: u32,
    remaining: usize,
}

fn mix(s: &mut [u32; 8]) {
    s[0] ^= s[1] << 11;
    s[3] = s[3].wrapping_add(s[0]);
    s[1] = s[1].wrapping_add(s[2]);
    s[1] ^= s[2] >> 2;
    s[4] = s[4].wrapping_add(s[1]);
    s[2] = s[2].wrapping_add(s[3]);
    s[2] ^= s[3] << 8;
    s[5] = s[5].wrapping_add(s[2]);
    s[3] = s[3].wrapping_add(s[4]);
    s[3] ^= s[4] >> 16;
    s[6] = s[6].wrapping_add(s[3]);
    s[4] = s[4].wrapping_add(s[5]);
    s[4] ^= s[5] << 10;
    s[7] = s[7].wrapping_add(s[4]);
    s[5] = s[5].wrapping_add(s[6]);
    s[5] ^= s[6] >> 4;
    s[0] = s[0].wrapping_add(s[5]);
    s[6] = s[6].wrapping_add(s[7]);
    s[6] ^= s[7] << 8;
    s[1] = s[1].wrapping_add(s[6]);
    s[7] = s[7].wrapping_add(s[0]);
    s[7] ^= s[0] >> 9;
    s[2] = s[2].wrapping_add(s[7]);
    s[0] = s[0].wrapping_add(s[1]);
}

impl Isaac {
    pub fn new(seed: &[u32]) -> Result<Self, SeedTooLong> {
        if seed.len() > 256 {
            return Err(SeedTooLong);
        }
        let mut cipher = Self {
            memory: [0; 256],
            results: [0; 256],
            a: 0,
            b: 0,
            c: 0,
            remaining: 0,
        };
        cipher.results[..seed.len()].copy_from_slice(seed);
        let mut s = [0x9e3779b9u32; 8];
        for _ in 0..4 {
            mix(&mut s);
        }
        for base in (0..256).step_by(8) {
            for (j, value) in s.iter_mut().enumerate() {
                *value = value.wrapping_add(cipher.results[base + j]);
            }
            mix(&mut s);
            cipher.memory[base..base + 8].copy_from_slice(&s);
        }
        for base in (0..256).step_by(8) {
            for (j, value) in s.iter_mut().enumerate() {
                *value = value.wrapping_add(cipher.memory[base + j]);
            }
            mix(&mut s);
            cipher.memory[base..base + 8].copy_from_slice(&s);
        }
        cipher.generate();
        Ok(cipher)
    }

    fn generate(&mut self) {
        self.c = self.c.wrapping_add(1);
        self.b = self.b.wrapping_add(self.c);
        for i in 0..256 {
            let x = self.memory[i];
            self.a ^= match i & 3 {
                0 => self.a << 13,
                1 => self.a >> 6,
                2 => self.a << 2,
                _ => self.a >> 16,
            };
            self.a = self.a.wrapping_add(self.memory[(i + 128) & 255]);
            let y = self.memory[((x >> 2) & 255) as usize]
                .wrapping_add(self.a)
                .wrapping_add(self.b);
            self.memory[i] = y;
            self.b = self.memory[((y >> 10) & 255) as usize].wrapping_add(x);
            self.results[i] = self.b;
        }
        self.remaining = 256;
    }

    pub fn next_u32(&mut self) -> u32 {
        if self.remaining == 0 {
            self.generate();
        }
        self.remaining -= 1;
        self.results[self.remaining]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_oversized_seed() {
        assert!(Isaac::new(&[0; 257]).is_err());
    }
}
