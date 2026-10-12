//! Reference FontMetricsDecoder semantics, including absolute left-kern reads.
use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::dialogue_text::Font;
pub fn decode(bytes: &[u8]) -> Result<Font, DecodeError> {
    let mut r = Reader::new(bytes);
    let mut advances = [0; 256];
    for v in &mut advances {
        *v = r.u8()? as i32;
    }
    if bytes.len() == 257 {
        r.u8()?;
        return Ok(Font {
            advances,
            kerning: vec![],
        });
    }
    let mut heights = [0usize; 256];
    let mut bearings = [0usize; 256];
    for v in &mut heights {
        *v = r.u8()? as usize;
    }
    for v in &mut bearings {
        *v = r.u8()? as usize;
    }
    let mut right = Vec::with_capacity(256);
    for height in heights {
        let mut value = 0i8;
        let mut glyph = Vec::new();
        for _ in 0..height {
            value = value.wrapping_add(r.u8()? as i8);
            glyph.push(value);
        }
        right.push(glyph);
    }
    // Kotlin uses data.getByte(tmpPos++), beginning at absolute byte zero.
    // Do not substitute the conventional sequential font decoder here.
    let mut absolute = Reader::new(bytes);
    let mut left = Vec::with_capacity(256);
    for height in heights {
        let mut value = 0i8;
        let mut glyph = Vec::new();
        for _ in 0..height {
            value = value.wrapping_add(absolute.u8()? as i8);
            glyph.push(value);
        }
        left.push(glyph);
    }
    let mut kerning = vec![0i8; 65536];
    for a in 0..256 {
        if a == 32 || a == 160 {
            continue;
        }
        for b in 0..256 {
            let min = bearings[a].max(bearings[b]);
            let max = (bearings[a] + heights[a]).min(bearings[b] + heights[b]);
            let mut k = advances[a].min(advances[b]);
            for y in min..max {
                k = k.min(left[a][y - bearings[a]] as i32 + right[b][y - bearings[b]] as i32);
            }
            kerning[(a << 8) | b] = (-k) as i8;
        }
    }
    Ok(Font { advances, kerning })
}
