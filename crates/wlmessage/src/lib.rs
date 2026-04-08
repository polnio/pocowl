use anyhow::Result;
use pocowl_wlstream::WaylandStream;
use tokio::io::AsyncReadExt as _;

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
    pub fn to_raw(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(self.data.len() + 8);
        vec.extend(self.object_id.to_ne_bytes());
        vec.extend(self.opcode.to_ne_bytes());
        vec.extend((self.data.len() as u16 + 8).to_ne_bytes());
        vec.extend(self.data.clone());
        vec
    }
    pub async fn read(stream: &mut WaylandStream) -> Result<Option<Self>> {
        let object_id = match stream.read_u32_le().await {
            Ok(id) => id,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let opcode = stream.read_u16_le().await?;
        let mut len = stream.read_u16_le().await?;
        if len < 8 {
            anyhow::bail!("length must be at least 8 bytes, got {}", len);
        }
        len -= 8;

        let mut data = vec![0; len as usize];
        let m = stream.read_exact(&mut data).await?;
        assert_eq!(m, len as usize);
        Ok(Some(WaylandMessage {
            object_id,
            opcode,
            data,
        }))
    }
}
