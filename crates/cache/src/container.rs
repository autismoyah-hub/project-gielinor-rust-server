use crate::{crc32, invalid};
use std::io;

/// Decode a JS5 container, allowing its optional two-byte group version trailer.
pub fn decode(input: &[u8], key: Option<[u32; 4]>, limit: usize) -> io::Result<Vec<u8>> {
    if input.len() < 5 {
        return Err(invalid("Truncated container"));
    }
    let compression = input[0];
    let length = u32::from_be_bytes(input[1..5].try_into().unwrap()) as usize;
    let header = if compression == 0 { 5usize } else { 9 };
    let end = header
        .checked_add(length)
        .ok_or_else(|| invalid("Container length overflow"))?;
    if length > limit || (input.len() != end && input.len() != end + 2) {
        return Err(invalid("Invalid container length"));
    }
    let mut data = input[..end].to_vec();
    if let Some(key) = key.filter(|k| *k != [0; 4]) {
        gielinor_protocol::xtea::decrypt(&mut data[5..], key);
    }
    if compression == 0 {
        return Ok(data[5..].to_vec());
    }
    let expected = u32::from_be_bytes(data[5..9].try_into().unwrap()) as usize;
    if expected > limit {
        return Err(invalid("Expanded container exceeds limit"));
    }
    let payload = &data[9..];
    let output = match compression {
        1 => {
            let mut source = b"BZh1".to_vec();
            source.extend_from_slice(payload);
            let mut output = vec![0; expected.max(1)];
            let mut output_len = output.len() as u32;
            // The library receives valid buffers of exactly the supplied lengths.
            let status = unsafe {
                libbz2_rs_sys::BZ2_bzBuffToBuffDecompress(
                    output.as_mut_ptr().cast(),
                    &mut output_len,
                    source.as_mut_ptr().cast(),
                    source.len() as u32,
                    0,
                    0,
                )
            };
            if status != 0 {
                return Err(invalid("Invalid bzip2 payload"));
            }
            output.truncate(output_len as usize);
            output
        }
        2 => gzip(payload, expected)?,
        _ => return Err(invalid("Unknown JS5 compression")),
    };
    if output.len() != expected {
        return Err(invalid("Expanded container length mismatch"));
    }
    Ok(output)
}

fn gzip(input: &[u8], limit: usize) -> io::Result<Vec<u8>> {
    if input.len() < 18 || input[..3] != [31, 139, 8] || input[3] & 0xe0 != 0 {
        return Err(invalid("Invalid gzip header"));
    }
    let flags = input[3];
    let end = input.len() - 8;
    let mut pos = 10;
    if flags & 4 != 0 {
        if pos + 2 > end {
            return Err(invalid("Truncated gzip extra field"));
        }
        let len = u16::from_le_bytes(input[pos..pos + 2].try_into().unwrap()) as usize;
        pos += 2 + len;
        if pos > end {
            return Err(invalid("Truncated gzip extra field"));
        }
    }
    for flag in [8, 16] {
        if flags & flag != 0 {
            loop {
                if pos >= end {
                    return Err(invalid("Truncated gzip string"));
                }
                let byte = input[pos];
                pos += 1;
                if byte == 0 {
                    break;
                }
            }
        }
    }
    if flags & 2 != 0 {
        if pos + 2 > end
            || crc32(&input[..pos]) as u16
                != u16::from_le_bytes(input[pos..pos + 2].try_into().unwrap())
        {
            return Err(invalid("Gzip header checksum mismatch"));
        }
        pos += 2;
    }
    if pos > end {
        return Err(invalid("Truncated gzip payload"));
    }
    let output = miniz_oxide::inflate::decompress_to_vec_with_limit(&input[pos..end], limit)
        .map_err(|_| invalid("Invalid gzip deflate stream"))?;
    let checksum = u32::from_le_bytes(input[end..end + 4].try_into().unwrap());
    let size = u32::from_le_bytes(input[end + 4..].try_into().unwrap());
    if output.len() as u32 != size || crc32(&output) != checksum {
        return Err(invalid("Gzip trailer mismatch"));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_encrypted_and_bounded() {
        let mut data = vec![0, 0, 0, 0, 9];
        data.extend_from_slice(b"test-data");
        gielinor_protocol::xtea::encrypt(&mut data[5..], [1, 2, 3, 4]);
        data.extend_from_slice(&[0, 1]);
        assert_eq!(decode(&data, Some([1, 2, 3, 4]), 32).unwrap(), b"test-data");
        assert!(decode(&data, None, 8).is_err());
        assert_eq!(crc32(b"123456789"), 0xcbf43926);
    }
    #[test]
    fn gzip_checks_payload_and_trailer() {
        let value = b"hello cache";
        let compressed = miniz_oxide::deflate::compress_to_vec(value, 6);
        let mut gz = vec![31, 139, 8, 0, 0, 0, 0, 0, 0, 255];
        gz.extend(compressed);
        gz.extend(crc32(value).to_le_bytes());
        gz.extend((value.len() as u32).to_le_bytes());
        let mut c = vec![2];
        c.extend((gz.len() as u32).to_be_bytes());
        c.extend((value.len() as u32).to_be_bytes());
        c.extend(gz);
        assert_eq!(decode(&c, None, 128).unwrap(), value);
        *c.last_mut().unwrap() ^= 1;
        assert!(decode(&c, None, 128).is_err());
    }
}
