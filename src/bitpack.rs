#[cfg(feature = "rustc-serialize")]
use rustc_serialize_crate::{Encodable, Encoder, Decodable, Decoder};

#[cfg(feature = "serde")]
use serde_crate as serde;

pub use bit_pack::BitPack;

#[derive(Debug, PartialEq, Eq)]
pub struct BPack {
    pack: BitPack<Vec<u8>>
}
impl BPack {
    pub fn new<'b>(buf: Vec<u8>) -> BPack {
        BPack { pack: BitPack::<Vec<u8>>::new(buf) }
    }
    pub fn get(&self) -> &BitPack<Vec<u8>> {
        &self.pack
    }
    pub fn get_mut(&mut self) -> &mut BitPack<Vec<u8>> {
        &mut self.pack
    }
}

#[cfg(feature = "rustc-serialize")]
impl Encodable for BPack {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.pack.buff.encode(s)
    }
}

#[cfg(feature = "rustc-serialize")]
impl Decodable for BPack {
    fn decode<D: Decoder>(d: &mut D) -> Result<BPack, D::Error> {
        let v = Decodable::decode(d)?;
        Ok(BPack { pack: BitPack::<Vec<u8>>::new(v) })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for BPack {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serde::Serialize::serialize(&self.pack.buff, s)
    }
}

#[cfg(feature = "serde")]
impl serde::Deserialize for BPack {
    fn deserialize<D>(d: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        let v = serde::Deserialize::deserialize(d)?;
        Ok(BPack { pack: BitPack::<Vec<u8>>::new(v) })
    }
}
