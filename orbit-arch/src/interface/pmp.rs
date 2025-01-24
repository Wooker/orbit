pub trait Pmp<Permission, Range> {
    type Result<T>;

    fn write_addr(&self, reg: usize, addr: usize) -> Self::Result<()>;
    fn read_addr(&self, reg: usize, addr: usize) -> Self::Result<usize>;
    fn write_cfg(
        &self,
        reg: usize,
        index: usize,
        range: Range,
        permission: Permission,
        locked: bool,
    ) -> Self::Result<()>;
    fn read_cfg(&self, reg: usize, index: usize) -> Self::Result<usize>;
    fn clear_cfg(&self, reg: usize, index: usize) -> Self::Result<()>;
}
