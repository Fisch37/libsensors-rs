use std::{ffi::{c_char, c_double, c_int}, fmt::Display, os::raw::c_void, ptr, result::Result as StdResult, sync::atomic::{AtomicBool, Ordering as MemOrdering}};
use libloading::{Library, Symbol};
use log::warn;
use crate::{chips::Chip, error::SensorsError, utils::{GLibCFree, invert_res_opt, ptr_to_ref}};

use self::error::{Error, Result};

pub mod chips;
pub mod error;
pub mod features;
pub mod subfeature;
mod ffi;
mod utils;

#[derive(Debug)]
pub enum LoadingError {
    Init(Error),
    AlreadyInitialised
}
impl From<Error> for LoadingError {
    fn from(value: Error) -> Self {
        LoadingError::Init(value)
    }
}
impl Display for LoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Init(e) => write!(f, "{e}"),
            Self::AlreadyInitialised => write!(f, "Already initialised")
        }
    }
}
impl std::error::Error for LoadingError { }

static LIBSENSORS_DOES_NOT_EXIST: AtomicBool = AtomicBool::new(true);
type LibLoadingResult<T> = StdResult<T, libloading::Error>;
type SymbolResult<'lib, T> = LibLoadingResult<Symbol<'lib, T>>;

pub(crate) type GetDetectedChips = unsafe extern "C" fn(*const ffi::sensors_chip_name, *mut c_int) -> *const ffi::sensors_chip_name;
pub(crate) type GetFeatures = unsafe extern "C" fn(*const ffi::sensors_chip_name, *mut c_int) -> *const ffi::sensors_feature;
pub(crate) type GetAllSubfeatures = unsafe extern "C" fn(*const ffi::sensors_chip_name, *const ffi::sensors_feature, *mut c_int) -> *const ffi::sensors_subfeature;

/// A handle to an initialized libsensors environment.
/// Note that only one of these may exist at the same time during the lifetime of a program!
/// libsensors also makes no claims as to thread safety, so creating two instances in different threads is also forbidden!
#[derive(Debug)]
pub struct LibSensors {
    inner: Library,
}
impl LibSensors {
    /// Initialises Libsensors and returns a handle to it.
    /// 
    /// Note that no two instances of this struct can exist at the same time.
    /// Trying to create an instance while another exists will raise an error.
    /// Furthermore, if two threads race (one dropping an instance, another creating it),
    /// there is no guarantee that the second thread will not encounter a duplication error,
    /// even if their timings were perfect.
    /// If you do this, you should create proper synchronisation around the threads.
    pub fn init() -> StdResult<Self, LoadingError> {
        // Acquire/Release is necessary here.
        // Acquire guarantees nobody stores, while we're reading.
        // Release guarantees nobody reads, while we're storing.
        if LIBSENSORS_DOES_NOT_EXIST.fetch_and(false, MemOrdering::AcqRel) {
            unsafe { Library::new("libsensors.so.5") }
                .map_err(Into::into)
                .and_then(|inner| {
                    SensorsError::convert_cint(
                        unsafe { inner.get::<unsafe extern "C" fn(*mut c_void) -> c_int>(c"sensors_init")?(ptr::null_mut()) }
                    ).map(|_| LibSensors { inner })
                    .map_err(Into::into)
                    // fetch_and above asserts that no two threads can be in this side of the if-stament at the same time.
                    // Therefore we have guarantee, that at this point, LIBSENSORS_DOES_NOT_EXIST is false, so we can simply set it true.
                    // (Using Relaxed here is fine, as we don't guarantee that this call succeeds, even if no LibSensors object exists)
                    .inspect_err(|_| LIBSENSORS_DOES_NOT_EXIST.store(true, MemOrdering::Relaxed))
                })
                .map_err(LoadingError::Init)
        } else {
            Err(LoadingError::AlreadyInitialised)
        }
    }

    fn close_inner(&self) -> LibLoadingResult<()> {
        unsafe { self.inner.get::<unsafe extern "C" fn()>(c"sensors_cleanup") }
            .map(|f| unsafe { f() })
    }

    pub fn close(self) -> LibLoadingResult<()> {
        self.close_inner()
    }

    pub fn get_chip<'lib>(&'lib self, mut index: c_int) -> Result<Option<Chip<'lib>>> {
        let fun = self._sensors_get_detected_chips()?;
        let raw = unsafe { fun(ptr::null(), &mut index) };
        invert_res_opt(
            unsafe { ptr_to_ref(raw) }.expect("get_chip: ptr not aligned")
                .map(|c| Chip::new(self, c))
        )
    }

    pub fn get_chips<'lib>(&'lib self) -> Result<ChipIterator<'lib>> {
        self._sensors_get_detected_chips()
            .map(|s| ChipIterator::new(self, s))
            .map_err(Into::into)
    }

    // -----------------------------------------
    //             Library functions
    // -----------------------------------------

    pub(crate) fn _free(&self) -> SymbolResult<'_, GLibCFree> {
        unsafe { self.inner.get(c"free") }
    }

    pub(crate) fn _sensors_get_adapter_name(&self) -> SymbolResult<'_, unsafe extern "C" fn(*const ffi::sensors_bus_id) -> *const c_char> {
        unsafe { self.inner.get(c"sensors_get_adapter_name") }
    }

    pub(crate) fn _sensors_get_label(&self) -> SymbolResult<'_, unsafe extern "C" fn(*const ffi::sensors_chip_name, *const ffi::sensors_feature) -> *mut c_char> { 
        unsafe { self.inner.get(c"sensors_get_label") }
    }

    pub(crate) fn _sensors_get_value(&self) -> SymbolResult<'_, unsafe extern "C" fn(*const ffi::sensors_chip_name, c_int, *mut c_double) -> c_int> {
        unsafe { self.inner.get(c"sensors_get_value") }
    }

    pub(crate) fn _sensors_set_value(&self) -> SymbolResult<'_, unsafe extern "C" fn(*const ffi::sensors_chip_name, c_int, c_double) -> c_int> {
        unsafe { self.inner.get(c"sensors_set_value") }
    }

    pub(crate) fn _sensors_get_detected_chips(&self) -> SymbolResult<'_, GetDetectedChips> {
        unsafe { self.inner.get(c"sensors_get_detected_chips") }
    }

    pub(crate) fn _sensors_get_features(&self) -> SymbolResult<'_, GetFeatures> {
        unsafe { self.inner.get(c"sensors_get_features") }
    }

    pub(crate) fn _sensors_get_all_subfeatures(&self) -> SymbolResult<'_, GetAllSubfeatures> {
        unsafe { self.inner.get(c"sensors_get_all_subfeatures") }
    }

    pub(crate) fn _sensors_get_subfeature(&self) -> SymbolResult<'_, unsafe extern "C" fn(*const ffi::sensors_chip_name, *const ffi::sensors_feature, ffi::sensors_subfeature_type::Type) -> *const ffi::sensors_subfeature> {
        unsafe { self.inner.get(c"sensors_get_subfeature") }
    }

    pub(crate) fn _sensors_strerror(&self) -> SymbolResult<'_, unsafe extern "C" fn(c_int) -> *const c_char> {
        unsafe { self.inner.get(c"sensors_strerror") }
    }
}
impl Drop for LibSensors {
    fn drop(&mut self) {
        if let Err(e) = self.close_inner() {
            warn!("Failed to load sensors_cleanup: {e}")
        }
    }
}

pub struct ChipIterator<'lib> {
    lib: &'lib LibSensors,
    // loading the GetDetectedChips eagerly means we don't have to worry about LibLoading errors during iteration.
    fun: Symbol<'lib, GetDetectedChips>,
    index: c_int
}
impl<'lib> ChipIterator<'lib> {
    fn new(lib: &'lib LibSensors, fun: Symbol<'lib, GetDetectedChips>) -> Self {
        ChipIterator { lib, fun, index: 0 }
    }
}
impl<'lib> Iterator for ChipIterator<'lib> {
    type Item = Result<Chip<'lib>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = unsafe { (self.fun)(ptr::null(), &mut self.index) };
        unsafe { ptr_to_ref(ptr) }.unwrap().map(|c| Chip::new(self.lib, c))
    }
}
