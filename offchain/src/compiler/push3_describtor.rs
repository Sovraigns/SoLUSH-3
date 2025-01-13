//! src/compiler/push3_descriptor.rs
//! 
//! A small module of helper functions to construct or interpret the 256-bit descriptors
//! used by our on-chain `Push3Interpreter`. Each descriptor encodes a tag, an offset,
//! a length, and any leftover bits (e.g., immediate data).

use ethers::types::U256;

/// The 256-bit descriptor layout is as follows:
/// [  8 bits: tag  |  32 bits: offset  |  32 bits: length  |  184 bits: leftover ]
///
/// We'll define small helper functions to build or parse these. 
/// Our `make_sublist_descriptor` is an example for tag=3 (SUBLIST).

/// Tag constants, if you want them:
pub const TAG_SUBLIST: u8 = 3;
pub const TAG_INT_LITERAL: u8 = 2;
pub const TAG_INSTRUCTION: u8 = 1;
pub const TAG_NONE: u8 = 0;

/// A small helper to shift the tag into the top 8 bits.
#[inline]
fn tag_bits(tag: u8) -> U256 {
    U256::from(tag) << 248
}

/// Build a descriptor for a SUBLIST with the given offset and length (in bytes),
/// leftover=0. 
///
/// # Example
/// ```
/// let desc = make_sublist_descriptor(0, code_bytes.len() as u32);
/// // pass `desc` into the interpreter's exec stack
/// ```
pub fn make_sublist_descriptor(offset: u32, length: u32) -> U256 {
    tag_bits(TAG_SUBLIST as u8)
        | (U256::from(offset) << 216)
        | (U256::from(length) << 184)
        | U256::zero() // leftover
}

/// Possibly you want a more general helper that builds any descriptor:
/// If you have other uses for leftover bits, you can pass that in:
pub fn make_descriptor(tag: u8, offset: u32, length: u32, leftover: U256) -> U256 {
    tag_bits(tag)
        | (U256::from(offset) << 216)
        | (U256::from(length) << 184)
        | leftover
}

/// If you want to parse a descriptor, you can define getTag, getOffset, getLength, etc:
pub fn get_tag(desc: U256) -> u8 {
    // top 8 bits => shift right 248
    (desc >> 248).as_u32() as u8
}

pub fn get_offset(desc: U256) -> u32 {
    ((desc >> 216) & U256::from(u32::MAX)).as_u32()
}

pub fn get_length(desc: U256) -> u32 {
    ((desc >> 184) & U256::from(u32::MAX)).as_u32()
}

pub fn get_low_184(desc: U256) -> U256 {
    desc & ((U256::from(1u64) << 184) - 1)
}
