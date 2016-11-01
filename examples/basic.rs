extern crate mincode;
extern crate rustc_serialize;

use mincode::{SizeLimit, FloatEncoding};
use mincode::rustc_serialize::{encode, decode};

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
struct World {
    id: u32,
    entities: Vec<Entity>
}

fn main() {
    let world = World {
        id: 127,
        entities: vec![Entity {x: 0.25, y: 4.0}, Entity {x: 10.0, y: 20.5}]
    };

    let encoded: Vec<u8> = encode(&world, SizeLimit::Infinite, FloatEncoding::F16).unwrap();

    // 1 byte for the length of the vector, 1 byte for id, 2 bytes per float.
    assert_eq!(encoded.len(), 1 + 1 + 4 * 2);

    let decoded: World = decode(&encoded, FloatEncoding::F16).unwrap();

    assert!(world == decoded);
}
