use std::ffi::{CStr, c_double};

use crate::{LibSensors, error::{Result, SensorsError}, ffi::{sensors_chip_name, sensors_subfeature}};



pub struct Subfeature<'lib> {
    lib: &'lib LibSensors,
    chip: &'lib sensors_chip_name,
    raw: &'lib sensors_subfeature
}
impl<'lib> Subfeature<'lib> {
    pub(crate) fn new(raw: &'lib sensors_subfeature, chip: &'lib sensors_chip_name, lib: &'lib LibSensors) -> Self {
        Self { lib, chip, raw }
    }

    pub fn get_name(&self) -> Option<&CStr> {
        let raw = self.raw.name;
        if raw.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(raw) })
        }
    }

    pub fn get_value(&self) -> Result<c_double> {
        let fun = self.lib._sensors_get_value()?;
        
        let mut value: c_double = c_double::NAN;
        SensorsError::convert_cint(
            // SAFETY: I dunno what to say, there aren't really any concerns here.
            //  *mut c_double isn't stored anywhere, self.chip exists so long as this struct instance does.
            unsafe { fun(self.chip, self.raw.number, &mut value) }
        )?;
        Ok(value)
    }

    pub fn set_value(&self, value: c_double) -> Result<()> {
        let fun = self.lib._sensors_set_value()?;
        SensorsError::convert_cint(
            unsafe { fun(self.chip, self.raw.number, value) }
        )?;
        Ok(())
    }
}