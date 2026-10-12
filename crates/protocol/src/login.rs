use crate::{
    buffer::{DecodeError, Order, Reader},
    rsa::PrivateKey,
    xtea,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub revision: u32,
    pub subrevision: u32,
    pub server_version: u32,
    pub client_type: u8,
    pub platform_type: u8,
    pub external_authenticator: u8,
}
impl Header {
    pub fn decode(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let revision = r.u32()?;
        if revision != 239 {
            return Err(DecodeError::InvalidValue("client revision"));
        }
        let header = Self {
            revision,
            subrevision: r.u32()?,
            server_version: r.u32()?,
            client_type: r.u8()?,
            platform_type: r.u8()?,
            external_authenticator: r.u8()?,
        };
        // The pinned Kotlin factory permits DESKTOP only. It records, but does not reject, subversions.
        if header.client_type != 1 {
            return Err(DecodeError::InvalidValue("unsupported client type"));
        }
        Ok(header)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Otp {
    TrustedDevice(u32),
    TrustedCode(u32),
    None,
    UntrustedCode(u32),
}
pub struct Secret(Vec<u8>);
impl Secret {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl Drop for Secret {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}
pub enum Authentication {
    Password(Secret),
    Token(Secret),
}
pub struct Login {
    pub header: Header,
    pub seed: [u32; 4],
    pub server_seed: u64,
    pub otp: Otp,
    pub authentication: Authentication,
    pub username: String,
    pub low_detail: bool,
    pub resized: bool,
    pub width: u16,
    pub height: u16,
    pub machine_id: [u8; 24],
    pub site_settings: String,
    pub affiliate_id: u32,
    pub archive_checksums: Vec<u32>,
    pub platform_blob: Vec<u8>,
    pub verification: u8,
    pub client_checksum: u32,
    pub cache_crcs: [u32; 23],
    pub trailing_bytes: Vec<u8>,
}

pub fn decode(payload: &[u8], rsa: &PrivateKey) -> Result<Login, DecodeError> {
    let mut input = Reader::new(payload);
    let header = Header::decode(&mut input)?;
    let rsa_length = input.u16()? as usize;
    let mut plaintext = rsa.decrypt(input.take(rsa_length)?)?;
    let auth = decode_auth(&plaintext);
    plaintext.fill(0);
    let (seed, server_seed, otp, authentication) = auth?;
    let mut encrypted = input.take(input.remaining())?.to_vec();
    xtea::decrypt(&mut encrypted, seed);
    decode_body(header, seed, server_seed, otp, authentication, &encrypted)
}

fn decode_auth(bytes: &[u8]) -> Result<([u32; 4], u64, Otp, Authentication), DecodeError> {
    let mut r = Reader::new(bytes);
    if r.u8()? != 1 {
        return Err(DecodeError::InvalidValue("RSA marker"));
    }
    let seed = [r.u32()?, r.u32()?, r.u32()?, r.u32()?];
    let server_seed = r.u64()?;
    let otp = match r.u8()? {
        0 => Otp::TrustedDevice(r.u32()?),
        1 => {
            let code = r.u24()?;
            r.take(1)?;
            Otp::TrustedCode(code)
        }
        2 => {
            r.take(4)?;
            Otp::None
        }
        3 => {
            let code = r.u24()?;
            r.take(1)?;
            Otp::UntrustedCode(code)
        }
        _ => return Err(DecodeError::InvalidValue("OTP authentication type")),
    };
    let kind = r.u8()?;
    let secret = Secret(r.string()?.into_bytes());
    let authentication = match kind {
        0 => Authentication::Password(secret),
        2 => Authentication::Token(secret),
        _ => return Err(DecodeError::InvalidValue("authentication type")),
    };
    Ok((seed, server_seed, otp, authentication))
}

fn decode_body(
    header: Header,
    seed: [u32; 4],
    server_seed: u64,
    otp: Otp,
    authentication: Authentication,
    bytes: &[u8],
) -> Result<Login, DecodeError> {
    let mut r = Reader::new(bytes);
    let username = r.string()?;
    let flags = r.u8()?;
    let width = r.u16()?;
    let height = r.u16()?;
    let machine_id = r.take(24)?.try_into().unwrap();
    let site_settings = r.string()?;
    let affiliate_id = r.u32()?;
    let count = r.u8()?;
    let mut archive_checksums = Vec::with_capacity(count as usize);
    for _ in 0..count {
        archive_checksums.push(r.u32()?);
    }
    let start = r.position();
    // Consume the original HostPlatformStats schema while preserving all fields byte-for-byte.
    r.take(18)?;
    for _ in 0..4 {
        r.prefixed_string()?;
    }
    r.take(3)?;
    for _ in 0..2 {
        r.prefixed_string()?;
    }
    r.take(18)?;
    for _ in 0..2 {
        r.prefixed_string()?;
    }
    let platform_blob = bytes[start..r.position()].to_vec();
    let verification = r.u8()?;
    let client_checksum = r.u32()?;
    use Order::*;
    let crc_order = [
        (18, Middle),
        (19, Middle),
        (22, Big),
        (12, Little),
        (9, Little),
        (1, Middle),
        (17, Middle),
        (16, InverseMiddle),
        (7, InverseMiddle),
        (13, Little),
        (3, Big),
        (5, Middle),
        (10, InverseMiddle),
        (11, Big),
        (2, Middle),
        (8, Little),
        (0, InverseMiddle),
        (21, Big),
        (4, InverseMiddle),
        (20, Middle),
        (6, Little),
        (14, InverseMiddle),
        (15, Little),
    ];
    let mut cache_crcs = [0; 23];
    for (archive, order) in crc_order {
        cache_crcs[archive] = r.u32_order(order)?;
    }
    let trailing_bytes = r.take(r.remaining())?.to_vec();
    Ok(Login {
        header,
        seed,
        server_seed,
        otp,
        authentication,
        username,
        low_detail: flags & 1 != 0,
        resized: flags & 2 != 0,
        width,
        height,
        machine_id,
        site_settings,
        affiliate_id,
        archive_checksums,
        platform_blob,
        verification,
        client_checksum,
        cache_crcs,
        trailing_bytes,
    })
}

/// Initial game handshake response: status 0 followed by a fresh server challenge.
pub fn challenge(seed: u64) -> [u8; 9] {
    let mut bytes = [0; 9];
    bytes[1..].copy_from_slice(&seed.to_be_bytes());
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn header_accepts_target_and_preserves_reference_subversion_behavior() {
        let bytes = [0, 0, 0, 239, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0];
        assert_eq!(
            Header::decode(&mut Reader::new(&bytes))
                .unwrap()
                .subrevision,
            1
        );
        for end in 0..bytes.len() {
            assert!(Header::decode(&mut Reader::new(&bytes[..end])).is_err());
        }
        let mut invalid = bytes;
        invalid[3] = 238;
        assert!(Header::decode(&mut Reader::new(&invalid)).is_err());
        invalid = bytes;
        invalid[12] = 4;
        assert!(Header::decode(&mut Reader::new(&invalid)).is_err());
    }
}
