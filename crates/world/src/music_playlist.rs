//! MusicPlaylist's packed cursor and seeded Kotlin shuffle. The seed is chosen
//! by the host RNG; replaying a packed cursor is entirely deterministic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Playlist(pub i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Empty,
    Position,
    Cycle,
    Seed,
    TooMany,
}
impl Playlist {
    pub fn new(position: u32, cycle: u32, seed: u32) -> Result<Self, Error> {
        if position > 255 {
            return Err(Error::Position);
        }
        if cycle > 255 {
            return Err(Error::Cycle);
        }
        if seed > 65535 {
            return Err(Error::Seed);
        }
        Ok(Self((position | cycle << 8 | seed << 16) as i32))
    }
    pub fn position(self) -> usize {
        self.0 as u32 as u8 as usize
    }
    pub fn cycle(self) -> u32 {
        (self.0 as u32 >> 8) & 255
    }
    pub fn seed(self) -> u32 {
        self.0 as u32 >> 16
    }
    pub fn next(self, count: usize) -> Result<Self, Error> {
        if count == 0 {
            return Err(Error::Empty);
        }
        let position = (self.position() + 1) % count;
        Self::new(
            u32::try_from(position).map_err(|_| Error::Position)?,
            self.cycle() + u32::from(position == 0),
            self.seed(),
        )
    }
    pub fn shuffled(self, tracks: &[u16]) -> Result<Vec<u16>, Error> {
        if tracks.is_empty() {
            return Err(Error::Empty);
        }
        if tracks.len() > i32::MAX as usize {
            return Err(Error::TooMany);
        }
        let mut shuffled = tracks.to_vec();
        Random::new(self.seed()).shuffle(&mut shuffled);
        if self.cycle() == 0 {
            return Ok(shuffled);
        }
        let first = shuffled[0];
        // Kotlin filter removes every occurrence equal to the first track.
        let mut rest: Vec<_> = tracks.iter().copied().filter(|id| *id != first).collect();
        Random::new(self.seed() + self.cycle()).shuffle(&mut rest);
        rest.insert(0, first);
        Ok(rest)
    }
    pub fn track(self, tracks: &[u16]) -> Result<u16, Error> {
        self.shuffled(tracks)?
            .get(self.position())
            .copied()
            .ok_or(Error::Position)
    }
}
// kotlin-stdlib2.2.0 XorWowRandom, restricted to the nonnegative playlist seeds.
// Long seeds split into low/high32 bits; playlist high32 is always zero.
struct Random {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
    v: u32,
    addend: u32,
}
impl Random {
    fn new(seed: u32) -> Self {
        let mut r = Self {
            x: seed,
            y: 0,
            z: 0,
            w: 0,
            v: !seed,
            addend: seed << 10,
        };
        for _ in 0..64 {
            r.next();
        }
        r
    }
    fn next(&mut self) -> u32 {
        let mut t = self.x ^ (self.x >> 2);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let v = self.v;
        self.w = v;
        t = t ^ (t << 1) ^ v ^ (v << 4);
        self.v = t;
        self.addend = self.addend.wrapping_add(362437);
        t.wrapping_add(self.addend)
    }
    fn bounded(&mut self, bound: u32) -> usize {
        if bound.is_power_of_two() {
            let bits = bound.trailing_zeros();
            let value = self.next();
            return if bits == 0 {
                0
            } else {
                (value >> (32 - bits)) as usize
            };
        }
        loop {
            let bits = self.next() >> 1;
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) as i32 >= 0 {
                return value as usize;
            }
        }
    }
    fn shuffle(&mut self, tracks: &mut [u16]) {
        for i in (1..tracks.len()).rev() {
            tracks.swap(i, self.bounded((i + 1) as u32));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cursor_boundaries_and_repeated_track_filter() {
        assert_eq!(
            Playlist::new(0, 255, 65535).unwrap().next(1),
            Err(Error::Cycle)
        );
        assert_eq!(
            Playlist::new(255, 0, 1).unwrap().next(257),
            Err(Error::Position)
        );
        assert_eq!(Playlist(0).shuffled(&[]), Err(Error::Empty));
        assert_eq!(
            Playlist::new(0, 1, 1)
                .unwrap()
                .shuffled(&[7, 7, 7])
                .unwrap(),
            [7]
        );
        for seed in [0, 1, 2391, 65535] {
            let first = Playlist::new(0, 0, seed)
                .unwrap()
                .track(&[1, 2, 3, 4, 5])
                .unwrap();
            assert_eq!(
                Playlist::new(0, 3, seed)
                    .unwrap()
                    .track(&[1, 2, 3, 4, 5])
                    .unwrap(),
                first
            );
        }
    }
}
