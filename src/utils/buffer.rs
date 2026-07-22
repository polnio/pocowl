#[derive(Debug, Clone)]
pub struct WaylandBuffer {
    pub data: Vec<u32>,
    pub stride: usize,
}
impl WaylandBuffer {
    pub fn new(width: usize, height: usize, stride: usize) -> Self {
        let data = vec![0; width * height];
        Self { data, stride }
    }
    pub fn slice(&self) -> WaylandBufferSlice<'_> {
        WaylandBufferSlice {
            data: &self.data,
            stride: self.stride,
        }
    }
    pub fn width(&self) -> usize {
        self.stride
    }
    pub fn height(&self) -> usize {
        self.data.len() / self.stride
    }
}

pub struct WaylandBufferSlice<'a> {
    data: &'a [u32],
    stride: usize,
}
