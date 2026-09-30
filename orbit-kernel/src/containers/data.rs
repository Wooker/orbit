#[derive(Debug, Clone, Copy)]
pub struct DataContainer {
    data: usize,
}
impl<'d> DataContainer {
    pub fn new(data: usize) -> Self {
        Self { data }
    }

    pub fn data(&self) -> usize {
        self.data
    }
}

pub trait DataAuto<'s> {
    #[inline(never)]
    fn to_data_container(&self) -> DataContainer
    where
        Self: Sized,
    {
        DataContainer::new(self as *const Self as usize)
    }
}
