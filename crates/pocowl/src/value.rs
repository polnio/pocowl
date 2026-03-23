use anyhow::Result;
use byteorder::NativeEndian;
use byteorder::ReadBytesExt as _;
use std::ffi::CString;
use std::io::Read as _;

fn align(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

pub trait WaylandValue: Sized {
    fn from_raw(buf: &mut &[u8]) -> Result<Self>;
    fn to_raw(self) -> Vec<u8>;
}
impl WaylandValue for u32 {
    fn from_raw(buf: &mut &[u8]) -> Result<Self> {
        buf.read_u32::<NativeEndian>().map_err(anyhow::Error::from)
    }
    fn to_raw(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}
impl WaylandValue for i32 {
    fn from_raw(buf: &mut &[u8]) -> Result<Self> {
        buf.read_i32::<NativeEndian>().map_err(anyhow::Error::from)
    }
    fn to_raw(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}
impl WaylandValue for String {
    fn from_raw(buf: &mut &[u8]) -> Result<Self> {
        let len = buf.read_u32::<NativeEndian>()?;
        let mut bytes = vec![0; len as usize];
        let n = buf.read(&mut bytes)?;
        if n != len as usize {
            return Err(anyhow::anyhow!(
                "length bigger than message size: {} > {} bytes",
                len,
                n
            ));
        }
        let cstr = CString::from_vec_with_nul(bytes)?;
        let s = cstr.into_string()?;
        Ok(s)
    }
    fn to_raw(self) -> Vec<u8> {
        let cstr = CString::new(self).expect("Failed to convert string to CString");
        let bytes = cstr.into_bytes();
        let len = bytes.len() as u32;
        let real_len = align(len as usize + 4, 4);
        let mut vec = Vec::with_capacity(real_len);
        println!("len: {}, real-len: {}", len, real_len);
        vec.extend(len.to_ne_bytes());
        vec.extend(bytes);
        vec.extend(vec![0; real_len - vec.len()]);
        vec
    }
}
