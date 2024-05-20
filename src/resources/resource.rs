use core::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use esp_hal::peripheral::{Peripheral, PeripheralRef};

#[derive(Clone)]
pub struct Resource<'res, T> {
    inner: T,
    _lifetime: PhantomData<&'res mut T>,
}

impl<'res, T> Resource<'res, T> {
    #[inline]
    pub fn new(inner: T) -> Self {
        // let inner = inner.into_ref();
        Self {
            inner,
            _lifetime: PhantomData,
        }
    }

    pub fn inner(self) -> T {
        self.inner
    }
}

// impl<'res, T> Deref for Resource<'res, T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl<'res, T> DerefMut for Resource<'res, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

// impl<'r, T> From<PeripheralRef<'r, T>> for Resource<'r, T>
// where
//     T: Peripheral<P = T>,
// {
//     fn from(mut value: PeripheralRef<'r, T>) -> Self {
//         let inner = value.clone_unchecked();
//         Self {
//             inner: value,
//             _lifetime: PhantomData,
//         }
//     }
// }
