use std::ffi::{CStr, c_int};

use libloading::Symbol;

use crate::{GetFeatures, LibSensors, error::Result, features::Feature, ffi::sensors_chip_name, utils::ptr_to_ref};

unsafe fn get_feature_raw<'lib>(
    fun: &Symbol<'lib, GetFeatures>,
    lib: &'lib LibSensors,
    chip: &'lib sensors_chip_name,
    index: &mut c_int
) -> Option<Feature<'lib>> {
    unsafe { ptr_to_ref(fun(chip, index)) }.unwrap()
        .map(|f| Feature::new(lib, chip, f))
}

pub struct Chip<'lib> {
    lib: &'lib LibSensors,
    raw: &'lib sensors_chip_name
}
impl<'lib> Chip<'lib> {
    pub fn new(lib: &'lib LibSensors, raw: &'lib sensors_chip_name) -> Self {
        Self { lib, raw }
    }

    pub fn get_name(&self) -> Result<Option<&CStr>> {
        let fun = self.lib._sensors_get_adapter_name()?;
        let raw = unsafe { fun(&self.raw.bus) };
        Ok(
            if raw.is_null() {
                None
            } else {
                Some(unsafe { CStr::from_ptr(raw) })
            }
        )
    }

    pub fn get_prefix(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.raw.prefix) }
    }

    pub fn get_feature(&self, index: c_int) -> Result<Option<Feature<'lib>>> {
        self.lib._sensors_get_features()
            .map_err(Into::into)
            .map(|sym| 
                unsafe { get_feature_raw(&sym, self.lib, self.raw, &mut index.clone()) }
            )
    }

    pub fn get_features(&self) -> Result<FeatureIterator<'lib>> {
        self.lib._sensors_get_features()
            .map_err(Into::into)
            .map(|sym| FeatureIterator::new(self.lib, sym, self.raw))
    }
}

pub struct FeatureIterator<'lib> {
    lib: &'lib LibSensors,
    fun: Symbol<'lib, GetFeatures>,
    chip: &'lib sensors_chip_name,
    index: c_int
}
impl<'lib> FeatureIterator<'lib> {
    pub fn new(lib: &'lib LibSensors, fun: Symbol<'lib, GetFeatures>, chip: &'lib sensors_chip_name) -> Self {
        Self { lib, fun, chip, index: 0 }
    }
}
impl<'lib> Iterator for FeatureIterator<'lib> {
    type Item = Feature<'lib>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { get_feature_raw(&self.fun, self.lib, self.chip, &mut self.index) }
    }
}