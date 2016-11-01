#[cfg(feature = "rustc-serialize")]
use rustc_serialize_crate::{Encodable, Encoder, Decodable, Decoder};

#[cfg(feature = "serde")]
use serde_crate as serde;

pub use bit_vec::BitVec;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone)]
pub struct BVec {
    vec: BitVec,
}
impl BVec {
    pub fn new(vec: BitVec) -> Self {
        BVec { vec: vec }
    }
    pub fn get(&self) -> &BitVec {
        &self.vec
    }
    pub fn get_mut(&mut self) -> &mut BitVec {
        &mut self.vec
    }
}

#[cfg(feature = "rustc-serialize")]
impl Encodable for BVec {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.vec.len().encode(s)?;
        //self.vec.to_bytes().encode(s)

        // avoid encoding the Vec's len
        for b in self.vec.to_bytes() {
            b.encode(s)?;
        }
        Ok(())
    }
}

#[cfg(feature = "rustc-serialize")]
impl Decodable for BVec {
    fn decode<D: Decoder>(d: &mut D) -> Result<BVec, D::Error> {
        let bit_len: usize = Decodable::decode(d)?;
        //let bytes: Vec<u8> = Decodable::decode(d)?;
        let byte_len = if bit_len % 8 == 0 { bit_len / 8 } else { bit_len / 8 + 1 };
        let mut bytes = Vec::with_capacity(byte_len);
        for _ in 0..byte_len {
            bytes.push(Decodable::decode(d)?);
        }
        let mut vec = BitVec::from_bytes(&bytes);
        unsafe { vec.set_len(bit_len); }
        Ok(BVec { vec: vec })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for BVec {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serde::Serialize::serialize(&self.vec.len(), s)?;

        // avoid encoding the Vec's len
        for b in self.vec.to_bytes() {
            serde::Serialize::serialize(&b, s)?;
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
impl serde::Deserialize for BVec {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        let bit_len: usize = serde::Deserialize::deserialize(d)?;
        let byte_len = if bit_len % 8 == 0 { bit_len / 8 } else { bit_len / 8 + 1 };
        let mut bytes = Vec::with_capacity(byte_len);
        for _ in 0..byte_len {
            bytes.push(serde::Deserialize::deserialize(d)?);
        }
        let mut vec = BitVec::from_bytes(&bytes);
        unsafe { vec.set_len(bit_len); }
        Ok(BVec { vec: vec })
    }
}