use core::cell::UnsafeCell;

pub struct ID<T>
where
    T: Sized,
{
    id: UnsafeCell<T>,
}

unsafe impl<T: Send + Sized> Sync for ID<T> {}

impl<T> ID<T>
where
    T: Sized + Copy,
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
