pub trait PortSystem<'s> {
    fn read() -> &'s [u8];
    fn write(buf: &'s [u8]);
}
