use std::io::Write;
use std::io::Error as IoError;
use std::error::Error;
use std::fmt;

use rustc_serialize_crate::Encoder;

use byteorder::WriteBytesExt;

use leb128;

use float::*;

pub type EncodingResult<T> = Result<T, EncodingError>;


/// An error that can be produced during encoding.
#[derive(Debug)]
pub enum EncodingError {
    /// An error originating from the underlying `Writer`.
    IoError(IoError),
    /// An object could not be encoded with the given size limit.
    ///
    /// This error is returned before any bytes are written to the
    /// output `Writer`.
    SizeLimit,
}

/// An Encoder that encodes values directly into a Writer.
///
/// This struct should not be used often.
/// For most cases, prefer the `encode_into` function.
pub struct EncoderWriter<'a, W: 'a> {
    writer: &'a mut W,
    write_f32: FloatEncoder<f32>,
    write_f64: FloatEncoder<f64>,
}

pub struct SizeChecker {
    pub size_limit: u64,
    pub written: u64,
    float_size_f32: usize,
    float_size_f64: usize,
}

fn wrap_io(err: IoError) -> EncodingError {
    EncodingError::IoError(err)
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            EncodingError::IoError(ref err) => write!(f, "IoError: {}", err),
            EncodingError::SizeLimit => write!(f, "SizeLimit")
        }
    }
}

impl Error for EncodingError {
    fn description(&self) -> &str {
        match *self {
            EncodingError::IoError(ref err) => Error::description(err),
            EncodingError::SizeLimit => "the size limit for decoding has been reached"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            EncodingError::IoError(ref err)     => err.cause(),
            EncodingError::SizeLimit => None
        }
    }
}

impl <'a, W: Write> EncoderWriter<'a, W> {
    pub fn new(w: &'a mut W, float_enc: FloatEncoding) -> EncoderWriter<'a, W> {
        let (write_f32, write_f64) = float_encoder(float_enc);
        EncoderWriter {
            writer: w,
            write_f32: write_f32,
            write_f64: write_f64,
        }
    }

    fn write_unsigned<T: Into<u64>>(&mut self, v: T) -> EncodingResult<()> {
        leb128::write::unsigned(&mut self.writer, v.into()).map(|_| ()).map_err(wrap_io)
    }

    fn write_signed<T: Into<i64>>(&mut self, v: T) -> EncodingResult<()> {
        leb128::write::signed(&mut self.writer, v.into()).map(|_| ()).map_err(wrap_io)
    }
}

impl SizeChecker {
    pub fn new(limit: u64, float_enc: FloatEncoding) -> SizeChecker {
        let (float_size_f32, float_size_f64) = float_sizes(float_enc);
        SizeChecker {
            size_limit: limit,
            written: 0,
            float_size_f32: float_size_f32,
            float_size_f64: float_size_f64,
        }
    }

    fn add_raw(&mut self, size: usize) -> EncodingResult<()> {
        self.written += size as u64;
        if self.written <= self.size_limit {
            Ok(())
        } else {
            Err(EncodingError::SizeLimit)
        }
    }

    /*fn add_value<T>(&mut self, _: T) -> EncodingResult<()> {
        use std::mem::size_of;
        self.add_raw(size_of::<T>())
    }*/

    fn add_value_unsigned<T: Into<u64>>(&mut self, t: T) -> EncodingResult<()> {
        let mut v: Vec<u8> = vec![];
        match leb128::write::unsigned(&mut v, t.into()) {
            Ok(n) => self.add_raw(n),
            Err(e) => Err(wrap_io(e))
        }
    }

    fn add_value_signed<T: Into<i64>>(&mut self, t: T) -> EncodingResult<()> {
        let mut v: Vec<u8> = vec![];
        match leb128::write::signed(&mut v, t.into()) {
            Ok(n) => self.add_raw(n),
            Err(e) => Err(wrap_io(e))
        }
    }
}

impl<'a, W: Write> Encoder for EncoderWriter<'a, W> {
    type Error = EncodingError;

    fn emit_nil(&mut self) -> EncodingResult<()> {
        Ok(())
    }
    fn emit_usize(&mut self, v: usize) -> EncodingResult<()> {
        self.write_unsigned(v as u64)
    }
    fn emit_u64(&mut self, v: u64) -> EncodingResult<()> {
        self.write_unsigned(v)
    }
    fn emit_u32(&mut self, v: u32) -> EncodingResult<()> {
        self.write_unsigned(v)
    }
    fn emit_u16(&mut self, v: u16) -> EncodingResult<()> {
        self.write_unsigned(v)
    }
    fn emit_u8(&mut self, v: u8) -> EncodingResult<()> {
        self.writer.write_u8(v).map_err(wrap_io)
    }
    fn emit_isize(&mut self, v: isize) -> EncodingResult<()> {
        self.write_signed(v as i64)
    }
    fn emit_i64(&mut self, v: i64) -> EncodingResult<()> {
        self.write_signed(v)
    }
    fn emit_i32(&mut self, v: i32) -> EncodingResult<()> {
        self.write_signed(v)
    }
    fn emit_i16(&mut self, v: i16) -> EncodingResult<()> {
        self.write_signed(v)
    }
    fn emit_i8(&mut self, v: i8) -> EncodingResult<()> {
        self.writer.write_i8(v).map_err(wrap_io)
    }
    fn emit_bool(&mut self, v: bool) -> EncodingResult<()> {
        self.writer.write_u8(if v {1} else {0}).map_err(wrap_io)
    }
    fn emit_f64(&mut self, v: f64) -> EncodingResult<()> {
        //self.writer.write_f64::<BigEndian>(v).map_err(wrap_io)
        (self.write_f64)(&mut self.writer, v).map_err(wrap_io)
    }
    fn emit_f32(&mut self, v: f32) -> EncodingResult<()> {
        //self.writer.write_f32::<BigEndian>(v).map_err(wrap_io)
        (self.write_f32)(&mut self.writer, v).map_err(wrap_io)
    }
    fn emit_char(&mut self, v: char) -> EncodingResult<()> {
        // TODO: change this back once unicode works
        //let mut cbuf = [0; 4];
        //let sz = v.encode_utf8(&mut cbuf[..]).unwrap_or(0);
        //let ptr = &cbuf[..sz];
        //self.writer.write_all(ptr).map_err(EncodingError::IoError)

        let mut inter = String::with_capacity(1);
        inter.push(v);
        self.writer.write_all(inter.as_bytes()).map_err(EncodingError::IoError)
    }
    fn emit_str(&mut self, v: &str) -> EncodingResult<()> {
        try!(self.emit_usize(v.len()));
        self.writer.write_all(v.as_bytes()).map_err(EncodingError::IoError)
    }
    fn emit_enum<F>(&mut self, __: &str, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_enum_variant<F>(&mut self, _: &str, v_id: usize, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        try!(self.write_unsigned(v_id as u64));
        f(self)
    }
    fn emit_enum_variant_arg<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_enum_struct_variant<F>(&mut self,
                                   _: &str,
                                   _: usize,
                                   _: usize,
                                   f: F)
                                   -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_enum_struct_variant_field<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_tuple<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_tuple_struct<F>(&mut self, _: &str, len: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        self.emit_tuple(len, f)
    }
    fn emit_tuple_struct_arg<F>(&mut self, f_idx: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        self.emit_tuple_arg(f_idx, f)
    }
    fn emit_option<F>(&mut self, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_option_none(&mut self) -> EncodingResult<()> {
        self.writer.write_u8(0).map_err(wrap_io)
    }
    fn emit_option_some<F>(&mut self, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        try!(self.writer.write_u8(1).map_err(wrap_io));
        f(self)
    }
    fn emit_seq<F>(&mut self, len: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        try!(self.emit_usize(len));
        f(self)
    }
    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_map<F>(&mut self, len: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        try!(self.emit_usize(len));
        f(self)
    }
    fn emit_map_elt_key<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_map_elt_val<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()>
    {
        f(self)
    }

}

impl Encoder for SizeChecker {
    type Error = EncodingError;

    fn emit_nil(&mut self) -> EncodingResult<()> {
        Ok(())
    }
    fn emit_usize(&mut self, v: usize) -> EncodingResult<()> {
        self.add_value_unsigned(v as u64)
    }
    fn emit_u64(&mut self, v: u64) -> EncodingResult<()> {
        self.add_value_unsigned(v)
    }
    fn emit_u32(&mut self, v: u32) -> EncodingResult<()> {
        self.add_value_unsigned(v)
    }
    fn emit_u16(&mut self, v: u16) -> EncodingResult<()> {
        self.add_value_unsigned(v)
    }
    fn emit_u8(&mut self, _: u8) -> EncodingResult<()> {
        self.add_value_unsigned(0 as u8)
    }
    fn emit_isize(&mut self, v: isize) -> EncodingResult<()> {
        self.add_value_signed(v as i64)
    }
    fn emit_i64(&mut self, v: i64) -> EncodingResult<()> {
        self.add_value_signed(v)
    }
    fn emit_i32(&mut self, v: i32) -> EncodingResult<()> {
        self.add_value_signed(v)
    }
    fn emit_i16(&mut self, v: i16) -> EncodingResult<()> {
        self.add_value_signed(v)
    }
    fn emit_i8(&mut self, v: i8) -> EncodingResult<()> {
        self.add_value_signed(v)
    }
    fn emit_bool(&mut self, _: bool) -> EncodingResult<()> {
        self.add_value_unsigned(0 as u8)
    }
    fn emit_f64(&mut self, _: f64) -> EncodingResult<()> {
        let bytes = self.float_size_f64;
        self.add_raw(bytes)
    }
    fn emit_f32(&mut self, _: f32) -> EncodingResult<()> {
        let bytes = self.float_size_f32;
        self.add_raw(bytes)
    }
    fn emit_char(&mut self, v: char) -> EncodingResult<()> {
        self.add_raw(v.len_utf8())
    }
    fn emit_str(&mut self, v: &str) -> EncodingResult<()> {
        self.add_value_unsigned(v.len() as u64)?;
        self.add_raw(v.len())
    }
    fn emit_enum<F>(&mut self, __: &str, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_enum_variant<F>(&mut self, _: &str, v_id: usize, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        self.add_value_unsigned(v_id as u32)?;
        f(self)
    }
    fn emit_enum_variant_arg<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_enum_struct_variant<F>(&mut self,
                                   _: &str,
                                   _: usize,
                                   _: usize,
                                   f: F)
                                   -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_enum_struct_variant_field<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_tuple<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_tuple_struct<F>(&mut self, _: &str, len: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        self.emit_tuple(len, f)
    }
    fn emit_tuple_struct_arg<F>(&mut self, f_idx: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        self.emit_tuple_arg(f_idx, f)
    }
    fn emit_option<F>(&mut self, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_option_none(&mut self) -> EncodingResult<()> {
        self.add_value_unsigned(0 as u8)
    }
    fn emit_option_some<F>(&mut self, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        try!(self.add_value_unsigned(1 as u8));
        f(self)
    }
    fn emit_seq<F>(&mut self, len: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        try!(self.emit_usize(len));
        f(self)
    }
    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_map<F>(&mut self, len: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        try!(self.emit_usize(len));
        f(self)
    }
    fn emit_map_elt_key<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }
    fn emit_map_elt_val<F>(&mut self, _: usize, f: F) -> EncodingResult<()>
        where F: FnOnce(&mut SizeChecker) -> EncodingResult<()>
    {
        f(self)
    }

}
