use std::io::{Read, Write};
use std::io::Error as IoError;

use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

use num_traits;

use half::f16;

/// How floats will be encoded.
#[repr(usize)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum FloatEncoding {
    /// encode f32 as f32 and f64 as f64.
    Normal,
    /// f32 and f64 will be endoded as half precision floats using the half crate.
    F16,
    /// f32 and f64 will be endoded as f32.
    F32,
    /// f32 will be encoded as [half](https://docs.rs/half/)::F16 and f64 will be endoded as f32.
    HalvePrecision,
}

pub type FloatEncoder<F: num_traits::Float> = fn(&mut Write, F) -> Result<(), IoError>;
pub type FloatDecoder<F: num_traits::Float> = fn(&mut Read) -> Result<F, IoError>;

static FLOAT_ENCODERS: [(FloatEncoder<f32>, FloatEncoder<f64>); 4] = [
    (write_f32_normal, write_f64_normal),
    (write_f32_f16,    write_f64_f16),
    (write_f32_normal, write_f64_f32),
    (write_f32_f16,    write_f64_f32),
];
static FLOAT_DECODERS: [(FloatDecoder<f32>, FloatDecoder<f64>); 4] = [
    (read_f32_normal, read_f64_normal),
    (read_f32_f16,    read_f64_f16),
    (read_f32_normal, read_f64_f32),
    (read_f32_f16,    read_f64_f32),
];
static FLOAT_SIZES: [(usize, usize); 4] = [
    // (size_of::<f32>(), size_of::<f64>()),
    // (size_of::<u16>(), size_of::<u16>()),
    // (size_of::<f32>(), size_of::<f32>()),
    // (size_of::<u16>(), size_of::<f32>()),
    (4, 8),
    (2, 2),
    (4, 4),
    (2, 4),
];

#[inline(always)]
pub fn float_encoder(float_enc: FloatEncoding) -> (FloatEncoder<f32>, FloatEncoder<f64>) {
    unsafe { *FLOAT_ENCODERS.get_unchecked(float_enc as usize) }
}

#[inline(always)]
pub fn float_decoder(float_enc: FloatEncoding) -> (FloatDecoder<f32>, FloatDecoder<f64>) {
    unsafe { *FLOAT_DECODERS.get_unchecked(float_enc as usize) }
}

#[inline(always)]
pub fn float_sizes(float_enc: FloatEncoding) -> (usize, usize) {
    unsafe { *FLOAT_SIZES.get_unchecked(float_enc as usize) }
}

fn write_f32_normal(w: &mut Write, v: f32) -> Result<(), IoError> {
    w.write_f32::<LittleEndian>(v)
}
fn write_f64_normal(w: &mut Write, v: f64) -> Result<(), IoError> {
    w.write_f64::<LittleEndian>(v)
}
fn write_f32_f16(w: &mut Write, v: f32) -> Result<(), IoError> {
    w.write_u16::<LittleEndian>(f16::from_f32(v).as_bits())
}
fn write_f64_f16(w: &mut Write, v: f64) -> Result<(), IoError> {
    w.write_u16::<LittleEndian>(f16::from_f64(v).as_bits())
}
fn write_f64_f32(w: &mut Write, v: f64) -> Result<(), IoError> {
    w.write_f32::<LittleEndian>(v as f32)
}

fn read_f32_normal(r: &mut Read) -> Result<f32, IoError> {
    r.read_f32::<LittleEndian>()
}
fn read_f64_normal(r: &mut Read) -> Result<f64, IoError> {
    r.read_f64::<LittleEndian>()
}
fn read_f32_f16(r: &mut Read) -> Result<f32, IoError> {
    r.read_u16::<LittleEndian>().map(|v| f32::from(f16::from_bits(v)))
}
fn read_f64_f16(r: &mut Read) -> Result<f64, IoError> {
    r.read_u16::<LittleEndian>().map(|v| f64::from(f16::from_bits(v)))
}
fn read_f64_f32(r: &mut Read) -> Result<f64, IoError> {
    r.read_f32::<LittleEndian>().map(|v| v as f64)
}
