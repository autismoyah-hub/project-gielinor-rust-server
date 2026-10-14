//! Bounded, read-only JS5 access. Opening a cache never creates or modifies it.
pub mod archive;
pub mod areas;
pub mod components;
pub mod container;
pub mod definitions;
pub mod db;
pub mod int_map_order;
pub mod disk;
pub mod enums;
pub mod game;
pub mod headbars;
pub mod index;
pub mod item_identity;
pub mod js5;
pub mod jvm_hash;
pub mod locations;
pub mod maps;
pub mod names;
pub mod npcs;
pub mod parameters;
pub mod projectiles;
pub mod symbols;
pub mod music;
pub mod variables;

use std::io;
pub(crate) fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
pub(crate) fn u24(bytes: &[u8]) -> u32 {
    (bytes[0] as u32) << 16 | (bytes[1] as u32) << 8 | bytes[2] as u32
}

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
pub mod sequences;
pub mod fonts;
pub mod constructed;
pub mod tower_shell;
