extern crate mincode;
extern crate rustc_serialize;
extern crate bit_vec;

use mincode::{SizeLimit, FloatEncoding, BVec};
use mincode::rustc_serialize::{encode, decode};

use bit_vec::BitVec;

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
struct World {
    //id: u16,
    entities: Vec<Entity>
}

fn main() {
    let bitvec = BVec::new(BitVec::from_fn(126, |i| { i % 1 == 0 }));

    /*let encoded: Vec<u8> = bitvec.get().to_bytes();
    assert_eq!(encoded.len(), 32);*/

    let encoded: Vec<u8> = encode(&bitvec, SizeLimit::Infinite, FloatEncoding::Normal).unwrap();

    println!("encoded {} {:?}", encoded.len(), encoded);

    // 1 byte for the length of the vector, ceil(126 / 8) == 16 bytes for the bits.
    assert_eq!(encoded.len(), 17);

    let decoded: BVec = decode(&encoded, FloatEncoding::Normal).unwrap();
    // let mut decoded: BVec = BVec::new(BitVec::from_bytes(&encoded));
    // unsafe { decoded.get_mut().set_len(bitvec.get().len()); }
    println!("before {} {:?}", bitvec.get().len(), bitvec);
    println!("after  {} {:?}", decoded.get().len(), decoded);
    assert!(bitvec == decoded);

    /*let world = World {
        //id: 1234,
        entities: vec![Entity {x: 0.0, y: 4.0}, Entity {x: 10.0, y: 20.5}]
    };

    let encoded: Vec<u8> = encode(&world, SizeLimit::Infinite).unwrap();

    // 8 bytes for the length of the vector, 4 bytes per float.
    //assert_eq!(encoded.len(), 8 + 4 * 4);
    println!("encoded {} {:?}", encoded.len(), encoded);

    let decoded: World = decode(&encoded[..]).unwrap();

    assert!(world == decoded);*/
}
