/*
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, this
 *    list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
 * ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
//! LineValidator's fixed-point sight/walk test, including endpoint blocker exceptions.
use crate::{collision::CollisionMap, CoordGrid};
#[derive(Clone, Copy)]
pub struct Bounds {
    pub coords: CoordGrid,
    pub width: i32,
    pub length: i32,
}
fn coordinate(a: i32, b: i32, size: i32) -> i32 {
    if a >= b {
        a
    } else if a + size - 1 <= b {
        a + size - 1
    } else {
        b
    }
}
pub fn clear(map: &CollisionMap, source: Bounds, target: Bounds, sight: bool, extra: u32) -> bool {
    if source.coords.level() != target.coords.level()
        || [source.width, source.length, target.width, target.length]
            .iter()
            .any(|v| !(1..=64).contains(v))
    {
        return false;
    }
    let sx = coordinate(source.coords.x(), target.coords.x(), source.width);
    let sz = coordinate(source.coords.z(), target.coords.z(), source.length);
    let ex = coordinate(target.coords.x(), source.coords.x(), target.width);
    let ez = coordinate(target.coords.z(), source.coords.z(), target.length);
    if sx == ex && sz == ez {
        return true;
    }
    let level = source.coords.level();
    if sight && map.get(sx, sz, level) & (0x100 | extra) != 0 {
        return false;
    }
    let dx = ex - sx;
    let dz = ez - sz;
    let (west, east, south, north) = if sight {
        (0x30000, 0x21000, 0x24000, 0x20400)
    } else {
        (0x240180, 0x240108, 0x240120, 0x240102)
    };
    let xf = (if dx >= 0 { west } else { east }) | extra;
    let zf = (if dz >= 0 { south } else { north }) | extra;
    // The original has symmetric X-major and Z-major loops. Keep the same tie rule.
    let x_major = dx.abs() > dz.abs();
    let (
        mut major,
        start_minor,
        end_major,
        end_minor,
        delta_major,
        delta_minor,
        mut major_flags,
        mut minor_flags,
    ) = if x_major {
        (sx, sz, ex, ez, dx, dz, xf, zf)
    } else {
        (sz, sx, ez, ex, dz, dx, zf, xf)
    };
    let step = if delta_major >= 0 { 1 } else { -1 };
    let mut scaled = (start_minor << 16) + 32768 + if delta_minor >= 0 { 0 } else { -1 };
    let tangent = (delta_minor << 16) / delta_major.abs();
    let blocked = |a, b, flags| {
        let (x, z) = if x_major { (a, b) } else { (b, a) };
        map.get(x, z, level) & flags != 0
    };
    let endpoint_mask = !(0x20000 | extra);
    while major != end_major {
        major += step;
        let minor = ((scaled as u32) >> 16) as i32;
        if sight && major == end_major && minor == end_minor {
            major_flags &= endpoint_mask;
        }
        if blocked(major, minor, major_flags) {
            return false;
        }
        scaled = scaled.wrapping_add(tangent);
        let next = ((scaled as u32) >> 16) as i32;
        if sight && major == end_major && next == end_minor {
            minor_flags &= endpoint_mask;
        }
        if next != minor && blocked(major, next, minor_flags) {
            return false;
        }
    }
    true
}
