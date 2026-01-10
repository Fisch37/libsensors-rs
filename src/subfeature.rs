use std::ffi::{CStr, c_double};

use crate::{LibSensors, error::{Result, SensorsError}, feature::FeatureType, ffi::{self, sensors_chip_name, sensors_subfeature}};


#[derive(Debug)]
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

    pub fn can_get(&self) -> bool { 
        self.raw.flags & ffi::SENSORS_MODE_R != 0
    }

    pub fn can_set(&self) -> bool {
        self.raw.flags & ffi::SENSORS_MODE_W != 0
    }
}


/// Feature-independent enum for commonly used subtypes
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericSubfeature {
    Input,
    Min,
    Max,
}
impl GenericSubfeature {
    pub fn to_primitive(self, feature_type: FeatureType) -> Option<ffi::sensors_subfeature_type::Type> {
        use ffi::sensors_subfeature_type::*;
        Some(match self {
            // can you see me doing this for every possible subfeature?
            // no. no you cannot. because i will not.
            Self::Input => {
                match feature_type {
                    FeatureType::In => SENSORS_SUBFEATURE_IN_INPUT,
                    FeatureType::Fan => SENSORS_SUBFEATURE_FAN_INPUT,
                    FeatureType::Temp => SENSORS_SUBFEATURE_FAN_INPUT,
                    FeatureType::Power => SENSORS_SUBFEATURE_POWER_INPUT,
                    FeatureType::Energy => SENSORS_SUBFEATURE_ENERGY_INPUT,
                    FeatureType::Current => SENSORS_SUBFEATURE_CURR_INPUT,
                    FeatureType::Humidity => SENSORS_SUBFEATURE_HUMIDITY_INPUT,
                    _ => return None
                }
            },
            Self::Min => {
                match feature_type {
                    FeatureType::In => SENSORS_SUBFEATURE_IN_MIN,
                    FeatureType::Fan => SENSORS_SUBFEATURE_FAN_MIN,
                    FeatureType::Temp => SENSORS_SUBFEATURE_TEMP_MIN,
                    FeatureType::Power => SENSORS_SUBFEATURE_POWER_MIN,
                    FeatureType::Current => SENSORS_SUBFEATURE_CURR_MIN,
                    _ => return None
                }
            },
            Self::Max => {
                match feature_type {
                    FeatureType::In => SENSORS_SUBFEATURE_IN_MAX,
                    FeatureType::Fan => SENSORS_SUBFEATURE_FAN_MAX,
                    FeatureType::Temp => SENSORS_SUBFEATURE_TEMP_MAX,
                    FeatureType::Power => SENSORS_SUBFEATURE_POWER_MAX,
                    FeatureType::Current => SENSORS_SUBFEATURE_CURR_MAX,
                    _ => return None
                }
            },
        })
    }
}