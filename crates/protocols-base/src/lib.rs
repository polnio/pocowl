pub trait WaylandProtocol<T> {
    fn call(&self, state: &mut T, opcode: u16, buf: &mut &[u8]) -> u32;
}
