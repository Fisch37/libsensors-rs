use std::{borrow::Borrow, ffi::c_void, ops::Deref};



/// Converts a raw pointer into a safe reference, valid for a given lifetime (inferred from usage).
/// 
/// Returns an Err(()) if the pointer is not aligned, an Ok(None) if the pointer is null,
/// or an Ok(Some(ref)) with the correct reference.
/// 
/// # Safety
/// The caller must guarantee that this pointer is actually valid for the given lifetime
/// and that it is either null or that the data it points to is valid for the given type of the resulting reference.
/// The caller must also guarantee that the data behind ptr is not mutated for the entire lifetime of the reference.
pub unsafe fn ptr_to_ref<'a, T>(ptr: *const T) -> Result<Option<&'a T>, ()> {
    if ptr.is_null() {
        Ok(None)
    } else if !ptr.is_aligned() {
        Err(())
    } else {
        // Caller has guaranteed that the reference is valid for the lifetime 'a
        // and contains valid data. 
        Ok(Some(unsafe { &*ptr }))
    }
}

pub type GLibCFree = unsafe extern "C" fn(*mut c_void);
/// A wrapper struct to ensure that the pointer inside it is freed.
pub struct GLibCBox<T> {
    ptr: *mut T,
    free: GLibCFree
}
impl<T> GLibCBox<T> {
    /// Construct a [`GLibCBox`] from a pointer and a freeing function.
    /// 
    /// # Safety
    /// The caller must guarantee that `free` can be called with `ptr` when the struct goes out of scope.
    pub unsafe fn from_raw(ptr: *mut T, free: GLibCFree) -> Self {
        Self { ptr, free }
    }
}
impl<T> Drop for GLibCBox<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: When this struct was created the caller guaranteed
            //  that self.free is safe to call for self.ptr
            unsafe { (self.free)(self.ptr as *mut c_void) }
        }
    }
}
impl<T> Deref for GLibCBox<T> {
    type Target = *mut T;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}
impl<T> Borrow<*mut T> for GLibCBox<T> {
    fn borrow(&self) -> &*mut T {
        &*self
    }
}
impl<T> AsRef<*mut T> for GLibCBox<T> {
    fn as_ref(&self) -> &*mut T {
        &*self
    }
}