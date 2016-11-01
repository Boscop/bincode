# Mincode - minimal encoding

Based on bincode, but encodes to smaller size.
Useful for encoding messages for real-time multiplayer game networking.

A compact encoder / decoder pair that uses an binary zero-fluff encoding scheme.
The size of the encoded object will be the same or smaller than the size that
the object takes up in memory in a running Rust program.

In addition to exposing two simple functions that encode to Vec<u8> and decode
from Vec<u8>, mincode exposes a Reader/Writer API that makes it work
perfectly with other stream-based apis such as rust files, network streams,
and the [flate2-rs](https://github.com/alexcrichton/flate2-rs) compression
library.

## Example

```rust
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

    // 1 byte for id, 1 byte for the length of the vector, 2 bytes per float.
    assert_eq!(encoded.len(), 1 + 1 + 4 * 2);

    let decoded: World = decode(&encoded, FloatEncoding::F16).unwrap();

    assert!(world == decoded);
}

```


It also supports efficient encoding of [bit vectors](https://crates.io/crates/bit-vec):

```rust
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
```


## Details

All integer types use [variable length encoding](https://crates.io/crates/leb128), taking only the necessary number of bytes.
This includes e.g. enum tags, Vec lengths and the elements of Vecs.
Tuples and structs are encoded by encoding their fields one-by-one, and enums are
encoded by first writing out the tag representing the variant and
then the contents.
Floats can be encoded in their original precision, [half precision (f16)](https://crates.io/crates/half),
always f32 or at half of their original precision.

