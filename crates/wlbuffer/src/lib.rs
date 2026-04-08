const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug, Clone)]
pub struct WaylandBuffer {
    pub data: Vec<u8>,
    pub stride: usize,
}
impl WaylandBuffer {
    pub fn new(width: usize, height: usize, stride: usize) -> Self {
        let data = vec![0; width * height * BYTES_PER_PIXEL];
        Self { data, stride }
    }
    pub fn slice(&self) -> WaylandBufferSlice<'_> {
        WaylandBufferSlice {
            data: &self.data,
            stride: self.stride,
        }
    }
    pub fn width(&self) -> usize {
        self.stride / BYTES_PER_PIXEL
    }
    pub fn height(&self) -> usize {
        self.data.len() / self.stride
    }
}

pub struct WaylandBufferSlice<'a> {
    data: &'a [u8],
    stride: usize,
}
