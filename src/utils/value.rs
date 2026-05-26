use anyhow::Result;
use byteorder::NativeEndian;
use byteorder::ReadBytesExt as _;
use fixed::types::I24F8;
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
        let alen = align(len as usize, 4);
        let mut bytes = vec![0; alen];
        let n = buf.read(&mut bytes)?;
        if n != alen {
            return Err(anyhow::anyhow!(
                "length bigger than message size: {} > {} bytes",
                alen,
                n
            ));
        }
        bytes.truncate(len as usize);
        let cstr = CString::from_vec_with_nul(bytes)?;
        let s = cstr.into_string()?;
        Ok(s)
    }
    fn to_raw(self) -> Vec<u8> {
        let cstr = CString::new(self).expect("Failed to convert string to CString");
        let bytes = cstr.into_bytes_with_nul();
        let len = bytes.len() as u32;
        let real_len = align(len as usize + 4, 4);
        let mut vec = Vec::with_capacity(real_len);
        vec.extend(len.to_ne_bytes());
        vec.extend(bytes);
        vec.extend(vec![0; real_len - vec.len()]);
        vec
    }
}

impl WaylandValue for I24F8 {
    fn from_raw(buf: &mut &[u8]) -> Result<Self> {
        let mut bytes = [0; _];
        let n = buf.read(&mut bytes)?;
        if n != 4 {
            anyhow::bail!("Failed to read 4 bytes");
        }
        Ok(I24F8::from_ne_bytes(bytes))
    }

    fn to_raw(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl WaylandValue for () {
    fn from_raw(_: &mut &[u8]) -> Result<Self> {
        Ok(())
    }

    fn to_raw(self) -> Vec<u8> {
        vec![]
    }
}

impl<T> WaylandValue for Option<T>
where
    T: WaylandValue + Clone,
{
    fn from_raw(buf: &mut &[u8]) -> Result<Self> {
        let data = T::from_raw(buf)?;
        // FIXME: Find a way to lookup instead of cloning
        let bytes = data.clone().to_raw();
        // SAFETY: transmute is safe because the bytes are 32 bit aligned
        let u32bytes = unsafe { std::mem::transmute::<&[u8], &[u32]>(&bytes) };
        Ok(u32bytes.iter().any(|&x| x != 0).then_some(data))
    }
    fn to_raw(self) -> Vec<u8> {
        match self {
            Some(val) => val.to_raw(),
            None => vec![0; 4],
        }
    }
}
