use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
pub struct RegionUniq<'a, T>(pub(crate) &'a mut T);

impl<'a, T: Sync> RegionUniq<'a, T> {
    pub fn shared(self) -> RegionRes<T> {
        // .as_mut_ptr() not in stable
        unsafe { RegionRes(NonNull::new_unchecked(self.0 as *mut T)) }
    }
}

impl<'a, T> Deref for RegionUniq<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T> DerefMut for RegionUniq<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

pub struct RegionRes<T: Sync>(pub(crate) NonNull<T>);

unsafe impl<T: Sync> Sync for RegionRes<T> {}

impl<T: Sync> Deref for RegionRes<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.as_ptr() }
    }
}

// investige how this compares to #[derive(Clone)]
impl<T: Sync> core::clone::Clone for RegionRes<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

pub struct RegionUniqSlice<'a, T>(pub(crate) &'a mut [T]);

impl<'a, T: Sync> RegionUniqSlice<'a, T> {
    pub fn shared(self) -> RegionResSlice<T> {
        unsafe { RegionResSlice(NonNull::new_unchecked(self.0.as_mut_ptr()), self.0.len()) }
    }
    pub fn zero_slice(&self) -> RegionResSlice<T> {
        unsafe { RegionResSlice(NonNull::new_unchecked(self.0.as_ptr() as *mut _), 0) }
    }
}

impl<'a, T> Deref for RegionUniqSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T> DerefMut for RegionUniqSlice<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

pub struct RegionResSlice<T: Sync>(pub(crate) NonNull<T>, pub(crate) usize);

unsafe impl<T: Sync> Sync for RegionResSlice<T> {}

impl<T: Sync> Deref for RegionResSlice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.0.as_ptr(), self.1) }
    }
}

impl<T: Sync> DerefMut for RegionResSlice<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.0.as_mut(), self.1) }
    }
}

// again investigate #[derive(Clone)]
impl<T: Sync> core::clone::Clone for RegionResSlice<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}
