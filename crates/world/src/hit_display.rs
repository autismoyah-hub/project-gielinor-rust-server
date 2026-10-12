//! HitBuilder display selection, captured when the hit is queued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    pub target: u16,
    pub source: u16,
    pub public: Option<u16>,
    pub delay: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Builder {
    pub target: u16,
    pub source: u16,
    pub public: Option<u16>,
    pub zero_lit: Option<u16>,
    pub zero_tint: Option<u16>,
    pub max_lit: Option<u16>,
    pub target_threshold: i32,
    pub source_threshold: i32,
    pub delay: u16,
}
impl Builder {
    pub fn build(self, damage: i32) -> Mark {
        let mut mark = Mark {
            target: self.target,
            source: self.source,
            public: self.public,
            delay: self.delay,
        };
        if damage == 0 {
            mark.target = self.zero_lit.unwrap_or(mark.target);
            mark.source = self.zero_lit.unwrap_or(mark.source);
            if mark.public.is_some() {
                mark.public = self.zero_tint.or(mark.public);
            }
        } else {
            if damage >= self.target_threshold {
                mark.target = self.max_lit.unwrap_or(mark.target);
            }
            if damage >= self.source_threshold {
                mark.source = self.max_lit.unwrap_or(mark.source);
            }
        }
        mark.public = mark.public.filter(|id| *id != 255);
        mark
    }
}
/// Kotlin deliberately truncates; a living entity can have an empty bar.
pub fn fill(segments: u8, current: i32, maximum: i32) -> Option<i32> {
    (maximum != 0).then(|| current.wrapping_mul(segments as i32).wrapping_div(maximum))
}
