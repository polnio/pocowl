use anyhow::Result;
use byteorder::{NativeEndian, ReadBytesExt as _};
use std::io::Read as _;

#[derive(Debug)]
pub struct WaylandMessage {
    pub object_id: u32,
    pub opcode: u16,
    pub data: Vec<u8>,
}
impl WaylandMessage {
    pub fn new(object_id: u32, opcode: u16, data: Vec<u8>) -> Self {
        Self {
            object_id,
            opcode,
            data,
        }
    }

    pub fn from_raw(buf: &mut &[u8]) -> Result<Self> {
        const HEADER_LEN: usize = 8;
        if buf.len() < HEADER_LEN {
            return Err(anyhow::anyhow!("Invalid message: {} bytes", buf.len()));
        }
        let object_id = buf.read_u32::<NativeEndian>().unwrap();
        let opcode = buf.read_u16::<NativeEndian>().unwrap();
        let mut len = buf.read_u16::<NativeEndian>().unwrap();
        if len < 8 {
            return Err(anyhow::anyhow!(
                "length must be at least 8 bytes, got {}",
                len
            ));
        }
        len -= 8;

        let mut data = vec![0; len as usize];
        let m = buf.read(&mut data)?;
        if m != len as usize {
            return Err(anyhow::anyhow!(
                "length bigger than message size: {} > {} bytes",
                len,
                m
            ));
        }
        Ok(WaylandMessage {
            object_id,
            opcode,
            data,
        })
    }
    pub fn to_raw(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(self.data.len() + 8);
        vec.extend(self.object_id.to_ne_bytes());
        vec.extend(self.opcode.to_ne_bytes());
        vec.extend((self.data.len() as u16).to_ne_bytes());
        vec.extend(self.data.clone());
        vec
    }
}
