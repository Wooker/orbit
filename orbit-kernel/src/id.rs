use core::{cell::UnsafeCell, ops::Add};

pub trait CanID: Sized + Copy {}
impl<T: Sized + Copy> CanID for T {}

pub struct ID<T>
where
    T: CanID,
{
    id: UnsafeCell<T>,
}

unsafe impl<T: CanID> Sync for ID<T> {}

impl<T> ID<T>
where
    T: CanID + Add<Output = T>,
{
    pub const fn new(id: T) -> Self {
        Self {
            id: UnsafeCell::new(id),
        }
    }

    pub fn get_id(&self) -> T {
        unsafe { *self.id.get() }
    }

    pub fn set(&self, value: T) {
        unsafe { *self.id.get() = value };
    }
}
