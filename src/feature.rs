use std::{error::Error as StdError, ffi::{CStr, CString, c_char, c_int}, fmt::Display, result::Result as StdResult};

use libloading::Symbol;

use crate::{GetAllSubfeatures, LibSensors, error::{Error, Result}, ffi::{self, sensors_chip_name, sensors_feature, sensors_subfeature, sensors_subfeature_type}, subfeature::Subfeature, utils::{GLibCBox, ptr_to_ref}};

#[derive(Debug)]
pub enum GetLabelError {
    GetLabelFailed,
    LibSensors(crate::error::Error)
}
impl Display for GetLabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (pre, e): (&'static str, Option<&dyn StdError>) = match self {
            Self::GetLabelFailed => ("GetLabelFailed", None),
            Self::LibSensors(e) => ("LibSensors", Some(e))
        };
        if let Some(e) = e {
            write!(f, "GetLabelError({}: {})", pre, e)
        } else {
            write!(f, "GetLabelError({})", pre)
        }
    }
}
impl StdError for GetLabelError { }

#[derive(Debug)]
pub struct Feature<'lib> {
    lib: &'lib LibSensors,
    chip: &'lib sensors_chip_name,
    raw: &'lib sensors_feature,
    type_: FeatureType
}
impl<'lib> Feature<'lib> {
    pub fn new(lib: &'lib LibSensors, chip: &'lib sensors_chip_name, raw: &'lib sensors_feature) -> Result<Self> {
        Ok(Self {
            lib, chip, raw,
            type_: FeatureType::from_repr(raw.type_).ok_or(Error::UnexpectedWildcard(raw.type_ as i64))?
        })
    }

    pub fn get_type(&self) -> FeatureType {
        self.type_
    }

    pub fn get_subfeature_by_type(&self, type_: sensors_subfeature_type::Type) -> Result<Option<Subfeature<'lib>>> {
        self.lib._sensors_get_subfeature()
            .map(|sym| {
                unsafe { ptr_to_ref(sym(self.chip, self.raw, type_)) }
                    .expect("get_subfeature_by_type: ptr is misaligned")
            })
            .map(|raw_opt| raw_opt.map(|raw| Subfeature::new(raw, self.chip, self.lib)))
            .map_err(Into::into)
    }

    pub fn get_subfeature(&self, mut index: c_int) -> Result<Option<&'lib sensors_subfeature>> {
        self.lib._sensors_get_all_subfeatures()
            .map(|sym|
                unsafe { ptr_to_ref(sym(self.chip, self.raw, &mut index)) }.unwrap()
            )
            .map_err(Into::into)
    }

    pub fn get_subfeatures(&self) -> Result<SubfeatureIterator<'lib>> {
        self.lib._sensors_get_all_subfeatures()
            .map(|sym| SubfeatureIterator::new(sym, self.chip, self.raw, self.lib))
            .map_err(Into::into)
    }


    /// Gets the label of this feature and returns the raw allocation.
    /// 
    /// The resulting GLibCBox contains a maybe-null pointer that is guaranteed to point to a valid c-string.
    /// GLibCBox also guarantees that the allocated memory will be correctly disposed when the box goes out of scope.
    fn get_label_extremely_raw(&self) -> Result<GLibCBox<c_char>> {
        // Note that this function is not unsafe, even though we return unsafe stuff.
        // This is because every safety guarantee required by the functions we use is accounted for.
        let get_label = self.lib._sensors_get_label()?;
        let free = self.lib._free()?;
        // SAFETY:
        //  We must pass in valid chip and feature pointers (we do, because references).
        //  The resulting ptr was allocated by libsensors' free function (which is what we are passing into GLibCBox here)
        //  Using GLibCBox also guarantees that the pointer is freed when it goes out of scope (GLibC's free function cannot error)
        Ok(unsafe { GLibCBox::from_raw(get_label(self.chip, self.raw), *free) })
    }

    /// Get the label for this feature.
    /// 
    /// Note that this function returns a [`CString`].
    /// If you want a [`String`], use [`Self::get_label`] instead.
    pub fn get_label_raw(&self) -> Result<Option<CString>> {
        self.get_label_extremely_raw()
            .map(|raw| {
                if raw.is_null() {
                    None
                } else {
                    // SAFETY:
                    //  - Libsensors guarantees us that the returned pointer is either null or points to a valid c-string.
                    //    - contains null terminator, valid for reads up to the null terminator
                    //  - we own that memory now and Libsensors won't modify it either.
                    //     (Nor do we, as it will be freed after to_owned)
                    //  - TODO: Technically we can't know whether strlen(raw) <= isize::MAX
                    Some(unsafe { CStr::from_ptr(*raw) }.to_owned())
                }
            })
    }

    pub fn get_label(&self) -> StdResult<String, GetLabelError> {
        self.get_label_raw()
            .map_err(GetLabelError::LibSensors)?
            .ok_or(GetLabelError::GetLabelFailed)?
            .into_string()
            .map_err(|e| GetLabelError::LibSensors(e.utf8_error().into()))
    }

    pub fn get_name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.raw.name) }
    }
}

pub struct SubfeatureIterator<'lib> {
    lib: &'lib LibSensors,
    fun: Symbol<'lib, GetAllSubfeatures>,
    chip: &'lib sensors_chip_name,
    feature: &'lib sensors_feature,
    index: c_int
}
impl<'lib> SubfeatureIterator<'lib> {
    pub fn new(fun: Symbol<'lib, GetAllSubfeatures>, chip: &'lib sensors_chip_name, feature: &'lib sensors_feature, lib: &'lib LibSensors) -> Self {
        Self { lib, fun, chip, feature, index: 0 }
    }
}
impl<'lib> Iterator for SubfeatureIterator<'lib> {
    type Item = Subfeature<'lib>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { ptr_to_ref((self.fun)(self.chip, self.feature, &mut self.index)) }.unwrap()
            .map(|raw| Subfeature::new(raw, self.chip, self.lib))
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::FromRepr)]
pub enum FeatureType {
    In = 0,
    Fan = 1,
    Temp = 2,
    Power = 3,
    Energy = 4,
    Current = 5,
    Humidity = 6,
    MaxMain = ffi::sensors_feature_type::SENSORS_FEATURE_MAX_MAIN,
    Vid = 0x10,
    Intrusion = 0x11,
    MaxOther = ffi::sensors_feature_type::SENSORS_FEATURE_MAX_OTHER,
    BeepEnable = 0x18,
    Max = ffi::sensors_feature_type::SENSORS_FEATURE_MAX
}

