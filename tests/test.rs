#![feature(proc_macro, step_by)]

extern crate mincode;
extern crate rustc_serialize;
extern crate serde;
#[macro_use] extern crate serde_derive;

use std::fmt::Debug;
use std::collections::HashMap;
use std::ops::Deref;

use rustc_serialize::{Encodable, Decodable};

use mincode::{RefBox, StrBox, SliceBox, BVec, BitVec, FloatEncoding/*, BPack*/};

use mincode::SizeLimit::{self, Infinite, Bounded};
use mincode::rustc_serialize::{encode, decode, decode_from, DecodingError};
use mincode::serde::{serialize, deserialize, deserialize_from, DeserializeError, DeserializeResult};

fn proxy_encode<V>(element: &V, size_limit: SizeLimit, float_enc: FloatEncoding) -> Vec<u8>
    where V: Encodable + Decodable + serde::Serialize + serde::Deserialize + PartialEq + Debug + 'static
{
    let v1 = mincode::rustc_serialize::encode(element, size_limit, float_enc).unwrap();
    let v2 = mincode::serde::serialize(element, size_limit, float_enc).unwrap();
    assert_eq!(v1, v2);

    v1
}

fn proxy_decode<V>(slice: &[u8], float_enc: FloatEncoding) -> V
    where V: Encodable + Decodable + serde::Serialize + serde::Deserialize + PartialEq + Debug + 'static
{
    let e1 = mincode::rustc_serialize::decode(slice, float_enc).unwrap();
    let e2 = mincode::serde::deserialize(slice, float_enc).unwrap();

    assert_eq!(e1, e2);

    e1
}

fn proxy_encoded_size<V>(element: &V, float_enc: FloatEncoding) -> u64
    where V: Encodable + serde::Serialize + PartialEq + Debug + 'static
{
    let ser_size = mincode::rustc_serialize::encoded_size(element, float_enc);
    let serde_size = mincode::serde::serialized_size(element, float_enc);
    assert_eq!(ser_size, serde_size);
    ser_size
}

fn the_same<V>(element: V, float_enc: FloatEncoding)
    where V: Encodable+Decodable+serde::Serialize+serde::Deserialize+PartialEq+Debug+'static
{
    // Make sure that the bahavior isize correct when wrapping with a RefBox.
    fn ref_box_correct<V>(v: &V, float_enc: FloatEncoding) -> bool
        where V: Encodable + Decodable + PartialEq + Debug + 'static
    {
        let rf = RefBox::new(v);
        let encoded = mincode::rustc_serialize::encode(&rf, Infinite, float_enc).unwrap();
        let decoded: RefBox<'static, V> = mincode::rustc_serialize::decode(&encoded[..], float_enc).unwrap();

        decoded.take().deref() == v
    }

    let size = proxy_encoded_size(&element, float_enc);

    let encoded = proxy_encode(&element, Infinite, float_enc);
    let decoded = proxy_decode(&encoded, float_enc);

    assert_eq!(element, decoded);
    assert_eq!(size, encoded.len() as u64);
    assert!(ref_box_correct(&element, float_enc));
}

#[test]
fn test_numbers() {
    // unsigned positive
    the_same(5u8, FloatEncoding::Normal);
    the_same(5u16, FloatEncoding::Normal);
    the_same(5u32, FloatEncoding::Normal);
    the_same(5u64, FloatEncoding::Normal);
    the_same(5usize, FloatEncoding::Normal);
    // signed positive
    the_same(5i8, FloatEncoding::Normal);
    the_same(5i16, FloatEncoding::Normal);
    the_same(5i32, FloatEncoding::Normal);
    the_same(5i64, FloatEncoding::Normal);
    the_same(5isize, FloatEncoding::Normal);
    // signed negative
    the_same(-5i8, FloatEncoding::Normal);
    the_same(-5i16, FloatEncoding::Normal);
    the_same(-5i32, FloatEncoding::Normal);
    the_same(-5i64, FloatEncoding::Normal);
    the_same(-5isize, FloatEncoding::Normal);
    // floating
    the_same(-100f32, FloatEncoding::Normal);
    the_same(0f32, FloatEncoding::Normal);
    the_same(5f32, FloatEncoding::Normal);
    the_same(-100f64, FloatEncoding::Normal);
    the_same(5f64, FloatEncoding::Normal);
}

#[test]
fn test_string() {
    the_same("".to_string(), FloatEncoding::Normal);
    the_same("a".to_string(), FloatEncoding::Normal);
}

#[test]
fn test_tuple() {
    the_same((1isize,), FloatEncoding::Normal);
    the_same((1isize,2isize,3isize), FloatEncoding::Normal);
    the_same((1isize,"foo".to_string(),()), FloatEncoding::Normal);
}

#[test]
fn test_basic_struct() {
    #[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, PartialEq, Debug)]
    struct Easy {
        x: isize,
        s: String,
        y: usize
    }
    the_same(Easy{x: -4, s: "foo".to_string(), y: 10}, FloatEncoding::Normal);
}

#[test]
fn test_nested_struct() {
    #[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, PartialEq, Debug)]
    struct Easy {
        x: isize,
        s: String,
        y: usize
    }
    #[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, PartialEq, Debug)]
    struct Nest {
        f: Easy,
        b: usize,
        s: Easy
    }

    the_same(Nest {
        f: Easy {x: -1, s: "foo".to_string(), y: 20},
        b: 100,
        s: Easy {x: -100, s: "bar".to_string(), y: 20}
    }, FloatEncoding::Normal);
}

#[test]
fn test_struct_newtype() {
    #[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, PartialEq, Debug)]
    struct NewtypeStr(usize);

    the_same(NewtypeStr(5), FloatEncoding::Normal);
}

#[test]
fn test_struct_tuple() {
    #[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, PartialEq, Debug)]
    struct TubStr(usize, String, f32);

    the_same(TubStr(5, "hello".to_string(), 3.2), FloatEncoding::Normal);
}

#[test]
fn test_option() {
    the_same(Some(5usize), FloatEncoding::Normal);
    the_same(Some("foo bar".to_string()), FloatEncoding::Normal);
    the_same(None::<usize>, FloatEncoding::Normal);
}

#[test]
fn test_enum() {
    #[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, PartialEq, Debug)]
    enum TestEnum {
        NoArg,
        OneArg(usize),
        Args(usize, usize),
        AnotherNoArg,
        StructLike{x: usize, y: f32}
    }
    the_same(TestEnum::NoArg, FloatEncoding::Normal);
    the_same(TestEnum::OneArg(4), FloatEncoding::Normal);
    the_same(TestEnum::Args(4, 5), FloatEncoding::Normal);
    the_same(TestEnum::AnotherNoArg, FloatEncoding::Normal);
    the_same(TestEnum::StructLike{x: 4, y: 3.14159}, FloatEncoding::Normal);
    the_same(vec![TestEnum::NoArg, TestEnum::OneArg(5), TestEnum::AnotherNoArg,
                  TestEnum::StructLike{x: 4, y:1.4}], FloatEncoding::Normal);
}

#[test]
fn test_vec() {
    let v: Vec<u8> = vec![];
    the_same(v, FloatEncoding::Normal);
    the_same(vec![1u64], FloatEncoding::Normal);
    the_same(vec![1u64,2,3,4,5,6], FloatEncoding::Normal);
}

#[test]
fn test_map() {
    let mut m = HashMap::new();
    m.insert(4u64, "foo".to_string());
    m.insert(0u64, "bar".to_string());
    the_same(m, FloatEncoding::Normal);
}

#[test]
fn test_bool() {
    the_same(true, FloatEncoding::Normal);
    the_same(false, FloatEncoding::Normal);
}

#[test]
fn test_unicode() {
    the_same("å".to_string(), FloatEncoding::Normal);
    the_same("aåååååååa".to_string(), FloatEncoding::Normal);
}

#[test]
fn test_fixed_size_array() {
    the_same([24u32; 32], FloatEncoding::Normal);
    the_same([1u64, 2, 3, 4, 5, 6, 7, 8], FloatEncoding::Normal);
    the_same([0u8; 19], FloatEncoding::Normal);
}

#[test]
fn decoding_errors() {
    fn isize_invalid_encoding<T>(res: mincode::rustc_serialize::DecodingResult<T>) {
        match res {
            Ok(_) => panic!("Expecting error"),
            Err(DecodingError::IoError(_)) => panic!("Expecting InvalidEncoding"),
            Err(DecodingError::SizeLimit) => panic!("Expecting InvalidEncoding"),
            Err(DecodingError::InvalidEncoding(_)) => {},
        }
    }

    isize_invalid_encoding(decode::<bool>(&vec![0xA][..], FloatEncoding::Normal));
    isize_invalid_encoding(decode::<String>(&vec![1, 0xFF][..], FloatEncoding::Normal));
    // Out-of-bounds variant
    #[derive(RustcEncodable, RustcDecodable, Serialize)]
    enum Test {
        One,
        Two,
    };
    isize_invalid_encoding(decode::<Test>(&vec![5][..], FloatEncoding::Normal));
    isize_invalid_encoding(decode::<Option<u8>>(&vec![5, 0][..], FloatEncoding::Normal));
}

#[test]
fn deserializing_errors() {
    fn isize_invalid_deserialize<T: Debug>(res: DeserializeResult<T>) {
        match res {
            Err(DeserializeError::InvalidEncoding(_)) => {},
            Err(DeserializeError::Serde(serde::de::value::Error::UnknownVariant(_))) => {},
            Err(DeserializeError::Serde(serde::de::value::Error::InvalidValue(_))) => {},
            _ => panic!("Expecting InvalidEncoding, got {:?}", res),
        }
    }

    isize_invalid_deserialize(deserialize::<bool>(&vec![0xA][..], FloatEncoding::Normal));
    isize_invalid_deserialize(deserialize::<String>(&vec![1, 0xFF][..], FloatEncoding::Normal));
    // Out-of-bounds variant
    #[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, Debug)]
    enum Test {
        One,
        Two,
    };
    isize_invalid_deserialize(deserialize::<Test>(&vec![5][..], FloatEncoding::Normal));
    isize_invalid_deserialize(deserialize::<Option<u8>>(&vec![5, 0][..], FloatEncoding::Normal));
}

#[test]
fn too_big_decode() {
    let encoded = vec![128, 128, 128, 1];
    let decoded: Result<u32, _> = decode_from(&mut &encoded[..], Bounded(3), FloatEncoding::Normal);
    assert!(decoded.is_err());

    let encoded = vec![128, 128, 128, 1];
    let decoded: Result<u32, _> = decode_from(&mut &encoded[..], Bounded(4), FloatEncoding::Normal);
    assert!(decoded.is_ok());

    assert_eq!(decoded.unwrap(), 1 << (7 * 3));
}

#[test]
fn too_big_deserialize() {
    let serialized = vec![128, 128, 128, 1];
    let deserialized: Result<u32, _> = deserialize_from(&mut &serialized[..], Bounded(3), FloatEncoding::Normal);
    assert!(deserialized.is_err());

    let serialized = vec![128, 128, 128, 1];
    let deserialized: Result<u32, _> = deserialize_from(&mut &serialized[..], Bounded(4), FloatEncoding::Normal);
    assert!(deserialized.is_ok());

    assert_eq!(deserialized.unwrap(), 1 << (7 * 3));
}

#[test]
fn char_serialization() {
    let chars = "Aa\0☺♪";
    for c in chars.chars() {
        let encoded = serialize(&c, Bounded(4), FloatEncoding::Normal).expect("serializing char failed");
        let decoded: char = deserialize(&encoded, FloatEncoding::Normal).expect("deserializing failed");
        assert_eq!(decoded, c);
    }
}

#[test]
fn too_big_char_decode() {
    let encoded = vec![0x41];
    let decoded: Result<char, _> = decode_from(&mut &encoded[..], Bounded(1), FloatEncoding::Normal);
    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap(), 'A');
}

#[test]
fn too_big_char_deserialize() {
    let serialized = vec![0x41];
    let deserialized: Result<char, _> = deserialize_from(&mut &serialized[..], Bounded(1), FloatEncoding::Normal);
    assert!(deserialized.is_ok());
    assert_eq!(deserialized.unwrap(), 'A');
}

#[test]
fn too_big_encode() {
    assert!(encode(&127u32, Bounded(1), FloatEncoding::Normal).is_ok());
    assert!(encode(&128u32, Bounded(1), FloatEncoding::Normal).is_err());
    assert!(encode(&0u32, Bounded(4), FloatEncoding::Normal).is_ok());

    assert!(encode(&"abcde", Bounded(1 + 4), FloatEncoding::Normal).is_err());
    assert!(encode(&"abcde", Bounded(1 + 5), FloatEncoding::Normal).is_ok());
}

#[test]
fn too_big_serialize() {
    assert!(serialize(&127u32, Bounded(1), FloatEncoding::Normal).is_ok());
    assert!(serialize(&128u32, Bounded(1), FloatEncoding::Normal).is_err());
    assert!(serialize(&0u32, Bounded(4), FloatEncoding::Normal).is_ok());

    assert!(serialize(&"abcde", Bounded(1 + 4), FloatEncoding::Normal).is_err());
    assert!(serialize(&"abcde", Bounded(1 + 5), FloatEncoding::Normal).is_ok());
}

#[test]
fn test_proxy_encoded_size() {
    assert!(proxy_encoded_size(&0u8, FloatEncoding::Normal) == 1);
    assert!(proxy_encoded_size(&0u16, FloatEncoding::Normal) == 1);
    assert!(proxy_encoded_size(&0u32, FloatEncoding::Normal) == 1);
    assert!(proxy_encoded_size(&0u64, FloatEncoding::Normal) == 1);

    // length isize stored as u64
    assert!(proxy_encoded_size(&"", FloatEncoding::Normal) == 1);
    assert!(proxy_encoded_size(&"a", FloatEncoding::Normal) == 1 + 1);

    assert!(proxy_encoded_size(&vec![0u32, 127u32, 2u32], FloatEncoding::Normal) == 1 + 3 * (1));
    assert!(proxy_encoded_size(&vec![0u32, 128u32, 2u32], FloatEncoding::Normal) == 1 + 1 + 2 + 1);
}

#[test]
fn test_serialized_size() {
    assert!(proxy_encoded_size(&0u8, FloatEncoding::Normal) == 1);
    assert!(proxy_encoded_size(&0u16, FloatEncoding::Normal) == 1);
    assert!(proxy_encoded_size(&0u32, FloatEncoding::Normal) == 1);
    assert!(proxy_encoded_size(&0u64, FloatEncoding::Normal) == 1);

    // length isize stored as u64
    assert!(proxy_encoded_size(&"", FloatEncoding::Normal) == 1);
    assert!(proxy_encoded_size(&"a", FloatEncoding::Normal) == 1 + 1);

    assert!(proxy_encoded_size(&vec![0u32, 127u32, 2u32], FloatEncoding::Normal) == 1 + 3 * (1));
    assert!(proxy_encoded_size(&vec![0u32, 128u32, 2u32], FloatEncoding::Normal) == 1 + 1 + 2 + 1);
}

#[test]
fn encode_box() {
    the_same(Box::new(5), FloatEncoding::Normal);
}

#[test]
fn test_refbox_encode() {
    let large_object = vec![1u32,2,3,4,5,6];
    let mut large_map = HashMap::new();
    large_map.insert(1, 2);


    #[derive(RustcEncodable, RustcDecodable, Debug)]
    enum Message<'a> {
        M1(RefBox<'a, Vec<u32>>),
        M2(RefBox<'a, HashMap<u32, u32>>)
    }

    // Test 1
    {
        let encoded = encode(&Message::M1(RefBox::new(&large_object)), Infinite, FloatEncoding::Normal).unwrap();
        let decoded: Message<'static> = decode(&encoded, FloatEncoding::Normal).unwrap();

        match decoded {
            Message::M1(b) => assert!(b.take().deref() == &large_object),
            _ => assert!(false)
        }
    }

    // Test 2
    {
        let encoded = encode(&Message::M2(RefBox::new(&large_map)), Infinite, FloatEncoding::Normal).unwrap();
        let decoded: Message<'static> = decode(&encoded, FloatEncoding::Normal).unwrap();

        match decoded {
            Message::M2(b) => assert!(b.take().deref() == &large_map),
            _ => assert!(false)
        }
    }
}

#[test]
fn test_refbox_serialize() {
    let large_object = vec![1u32,2,3,4,5,6];
    let mut large_map = HashMap::new();
    large_map.insert(1, 2);


    #[derive(RustcEncodable, RustcDecodable, Serialize, Deserialize, Debug)]
    enum Message<'a> {
        M1(RefBox<'a, Vec<u32>>),
        M2(RefBox<'a, HashMap<u32, u32>>)
    }

    // Test 1
    {
        let serialized = serialize(&Message::M1(RefBox::new(&large_object)), Infinite, FloatEncoding::Normal).unwrap();
        let deserialized: Message<'static> = deserialize_from(&mut &serialized[..], Infinite, FloatEncoding::Normal).unwrap();

        match deserialized {
            Message::M1(b) => assert!(b.take().deref() == &large_object),
            _ => assert!(false)
        }
    }

    // Test 2
    {
        let serialized = serialize(&Message::M2(RefBox::new(&large_map)), Infinite, FloatEncoding::Normal).unwrap();
        let deserialized: Message<'static> = deserialize_from(&mut &serialized[..], Infinite, FloatEncoding::Normal).unwrap();

        match deserialized {
            Message::M2(b) => assert!(b.take().deref() == &large_map),
            _ => assert!(false)
        }
    }
}

#[test]
fn test_strbox_encode() {
    let strx: &'static str = "hello world";
    let encoded = encode(&StrBox::new(strx), Infinite, FloatEncoding::Normal).unwrap();
    let decoded: StrBox<'static> = decode(&encoded, FloatEncoding::Normal).unwrap();
    let stringx: String = decoded.take();
    assert!(strx == &stringx[..]);
}

#[test]
fn test_strbox_serialize() {
    let strx: &'static str = "hello world";
    let serialized = serialize(&StrBox::new(strx), Infinite, FloatEncoding::Normal).unwrap();
    let deserialized: StrBox<'static> = deserialize_from(&mut &serialized[..], Infinite, FloatEncoding::Normal).unwrap();
    let stringx: String = deserialized.take();
    assert!(strx == &stringx[..]);
}

#[test]
fn test_slicebox_encode() {
    let slice = [1u32, 2, 3 ,4, 5];
    let encoded = encode(&SliceBox::new(&slice), Infinite, FloatEncoding::Normal).unwrap();
    let decoded: SliceBox<'static, u32> = decode(&encoded, FloatEncoding::Normal).unwrap();
    {
        let sb: &[u32] = &decoded;
        assert!(slice == sb);
    }
    let vecx: Vec<u32> = decoded.take();
    assert!(slice == &vecx[..]);
}

#[test]
fn test_slicebox_serialize() {
    let slice = [1u32, 2, 3 ,4, 5];
    let serialized = serialize(&SliceBox::new(&slice), Infinite, FloatEncoding::Normal).unwrap();
    let deserialized: SliceBox<'static, u32> = deserialize_from(&mut &serialized[..], Infinite, FloatEncoding::Normal).unwrap();
    {
        let sb: &[u32] = &deserialized;
        assert!(slice == sb);
    }
    let vecx: Vec<u32> = deserialized.take();
    assert!(slice == &vecx[..]);
}

#[test]
fn test_multi_strings_encode() {
    assert!(encode(&("foo", "bar", "baz"), Infinite, FloatEncoding::Normal).is_ok());
}

#[test]
fn test_multi_strings_serialize() {
    assert!(serialize(&("foo", "bar", "baz"), Infinite, FloatEncoding::Normal).is_ok());
}

#[test]
fn test_oom_protection() {
    use std::io::Cursor;
    use std::mem::size_of;
    if size_of::<usize>() == 8 {
        #[derive(RustcEncodable, RustcDecodable)]
        struct FakeVec {
            len: u64,
            byte: u8
        }
        let x = mincode::rustc_serialize::encode(&FakeVec { len: 0xffffffffffffffffu64, byte: 1 }, Bounded(11), FloatEncoding::Normal).unwrap();
        let y : Result<Vec<u8>, _> = mincode::rustc_serialize::decode_from(&mut Cursor::new(&x[..]), Bounded(11), FloatEncoding::Normal);
        match y {
            Err(DecodingError::SizeLimit) => (),
            _ => panic!("error SizeLimit expected"),
        }
    } else {
        #[derive(RustcEncodable, RustcDecodable)]
        struct FakeVec {
            len: u32,
            byte: u8
        }
        let x = mincode::rustc_serialize::encode(&FakeVec { len: 0xffffffffu32, byte: 1 }, Bounded(6), FloatEncoding::Normal).unwrap();
        let y : Result<Vec<u8>, _> = mincode::rustc_serialize::decode_from(&mut Cursor::new(&x[..]), Bounded(6), FloatEncoding::Normal);
        match y {
            Err(DecodingError::SizeLimit) => (),
            _ => panic!("error SizeLimit expected"),
        }
    }
}

#[test]
fn path_buf() {
    use std::path::{Path, PathBuf};
    let path = Path::new("foo").to_path_buf();
    let serde_encoded = mincode::serde::serialize(&path, Infinite, FloatEncoding::Normal).unwrap();
    let decoded: PathBuf = mincode::serde::deserialize(&serde_encoded, FloatEncoding::Normal).unwrap();
    assert!(path.to_str() == decoded.to_str());
}

#[test]
fn test_u8_same() {
    the_same(127u8, FloatEncoding::Normal);
    the_same(128u8, FloatEncoding::Normal);
}

#[test]
fn test_bitvec_same() {
    let bitvec = BVec::new(BitVec::from_fn(127, |i| { i % 2 == 0 }));
    the_same(bitvec.get().to_bytes(), FloatEncoding::Normal);
    the_same(bitvec, FloatEncoding::Normal);
    the_same(BVec::new(BitVec::from_fn(128, |i| { i % 2 == 0 })), FloatEncoding::Normal);
    the_same(BVec::new(BitVec::from_fn(254, |i| { i % 3 == 0 })), FloatEncoding::Normal);
    the_same(BVec::new(BitVec::from_fn(255, |i| { i % 4 == 0 })), FloatEncoding::Normal);
    for bit_len in (0..1000).step_by(3) {
        let bitvec = BVec::new(BitVec::from_fn(bit_len, |i| { i % 2 == 0 }));
        the_same(bitvec, FloatEncoding::Normal);
        let bitvec = BVec::new(BitVec::from_fn(bit_len, |i| { i % 3 == 0 }));
        the_same(bitvec, FloatEncoding::Normal);
        let bitvec = BVec::new(BitVec::from_fn(bit_len, |i| { i % 5 == 0 }));
        the_same(bitvec, FloatEncoding::Normal);
    }
}

#[test]
fn test_bitvec_min_len() {
    for bit_len in (0..1000).step_by(3) {
        let bitvec = BVec::new(BitVec::from_fn(bit_len, |i| { i % 3 == 0 }));
        let byte_len_of_encoded_len = encode(&bitvec.get().len(), SizeLimit::Infinite, FloatEncoding::Normal).unwrap().len();
        let byte_len_of_vec = if bit_len % 8 == 0 { bit_len / 8 } else { bit_len / 8 + 1 };
        println!("{}, {}, {}", bit_len, byte_len_of_encoded_len, byte_len_of_vec);
        let encoded: Vec<u8> = encode(&bitvec, SizeLimit::Infinite, FloatEncoding::Normal).unwrap();
        assert_eq!(encoded.len(), byte_len_of_encoded_len + byte_len_of_vec);
    }
}

/*#[test]
fn test_bitpack_same() {
    let mut bitpack = BPack::new(vec![0; 4]);
    bitpack.get_mut().write(10, 4).unwrap();
    bitpack.get_mut().write(1021, 10).unwrap();
    bitpack.get_mut().write(3, 2).unwrap();
    bitpack.get_mut().flush();
    the_same(bitpack);
}*/

#[test]
fn test_float_enc_same() {
    the_same(vec![0.0f32, 2., 4., 6., 8., 10.], FloatEncoding::Normal);
    the_same(vec![0.0f32, 2., 4., 6., 8., 10.], FloatEncoding::F16);
    the_same(vec![0.0f32, 2., 4., 6., 8., 10.], FloatEncoding::F32);
    the_same(vec![0.0f32, 2., 4., 6., 8., 10.], FloatEncoding::HalvePrecision);

    for i in 0..1000 {
        let v = i as f32;
        the_same(v, FloatEncoding::Normal);
        the_same(v, FloatEncoding::F16);
        the_same(v, FloatEncoding::F32);
        the_same(v, FloatEncoding::HalvePrecision);
    }

    for i in 0..1000 {
        let v = i as f64;
        the_same(v, FloatEncoding::Normal);
        the_same(v, FloatEncoding::F16);
        the_same(v, FloatEncoding::F32);
        the_same(v, FloatEncoding::HalvePrecision);
    }
}