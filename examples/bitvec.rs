extern crate mincode;
extern crate rustc_serialize;

use mincode::{SizeLimit, FloatEncoding, BVec, BitVec};
use mincode::rustc_serialize::{encode, decode};

fn main() {
    let bitvec = BVec::new(BitVec::from_fn(126, |i| { i % 2 == 0 }));

    let encoded: Vec<u8> = encode(&bitvec, SizeLimit::Infinite, FloatEncoding::Normal).unwrap();

    // 1 byte for the length of the vector, ceil(126 / 8) == 16 bytes for the bits.
    assert_eq!(encoded.len(), 17);

    let decoded: BVec = decode(&encoded, FloatEncoding::Normal).unwrap();

    assert!(bitvec == decoded);
}
