use std::ffi::{CStr, c_int, c_short};

use libloading::Symbol;

use crate::{GetFeatures, LibSensors, error::{Error, Result}, features::Feature, ffi::{self, sensors_bus_id, sensors_chip_name}, utils::{invert_res_opt, ptr_to_ref, try_cstr}};

unsafe fn get_feature_raw<'lib>(
    fun: &Symbol<'lib, GetFeatures>,
    lib: &'lib LibSensors,
    chip: &'lib sensors_chip_name,
    index: &mut c_int
) -> Option<Result<Feature<'lib>>> {
    unsafe { ptr_to_ref(fun(chip, index)) }.unwrap()
        .map(|f| Feature::new(lib, chip, f))
}

pub struct Chip<'lib> {
    lib: &'lib LibSensors,
    raw: &'lib sensors_chip_name,
    // wrapped data
    prefix: &'lib CStr,
    bus: BusId
}
impl<'lib> Chip<'lib> {
    pub fn new(lib: &'lib LibSensors, raw: &'lib sensors_chip_name) -> Result<Self> {
        Ok(Self {
            lib, raw,
            // SAFETY: raw is valid for the lifetime 'lib (matching the lifetime of prefix)
            //  It is valid for reads up to the NUL terminator, coming from libsensors itself.
            //  FIXME: technically nothing prevents raw.prefix from being longer than isize::MAX
            prefix: unsafe { try_cstr(raw.prefix).expect("chip prefix was null") },
            bus: BusId::try_from(raw.bus)?
        })
    }

    pub fn get_name(&self) -> Result<Option<&'lib CStr>> {
        let fun = self.lib._sensors_get_adapter_name()?;
        // SAFETY: I can call sensors_get_adapter_name at any time. There are no safety requirements
        //  The passed pointer trivially lives as long as fun & it isn't stored by fun.
        let raw = unsafe { fun(&self.raw.bus) };
        // SAFETY: We are assuming that libsensors returns a correct C-string.
        //  This means that it is contains a NUL and is valid for reads until the first NUL.
        //  sensors_get_adapter_name also returns a pointer from libsensors internal storage,
        //   which means that it lives until sensors_cleanup (i.e. the lifetime of self.lib).
        //  libsensors (probably!) doesn't mutate the adapter name for its lifetime (TODO: Verify this)
        //  FIXME: Technically there is nothing preventing strlen(raw) > isize::MAX, even though it is very unlikely.
        Ok(unsafe { try_cstr(raw) })
    }

    pub fn get_prefix(&self) -> &'lib CStr {
        self.prefix
    }

    pub fn get_bus_id(&self) -> BusId {
        self.bus
    }

    pub fn get_feature(&self, index: c_int) -> Result<Option<Feature<'lib>>> {
        self.lib._sensors_get_features()
            .map_err(Into::into)
            .and_then(|sym| 
                invert_res_opt(
                    unsafe { get_feature_raw(&sym, self.lib, self.raw, &mut index.clone()) }
                )
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
    type Item = Result<Feature<'lib>>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { get_feature_raw(&self.fun, self.lib, self.chip, &mut self.index) }
    }
}


#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::IntoStaticStr)]
/// The type of a bus, excluding any wildcard values
pub enum BusType {
    I2C = 0,
    ISA = 1,
    PCI = 2,
    SPI = 3,
    VIRTUAL = 4,
    ACPI = 5,
    HID = 6,
    MDIO = 7,
    SCSI = 8
}
impl TryFrom<c_short> for BusType {
    type Error = Error;

    fn try_from(value: c_short) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            ffi::SENSORS_BUS_TYPE_I2C     => Self::I2C,
            ffi::SENSORS_BUS_TYPE_ISA     => Self::ISA,
            ffi::SENSORS_BUS_TYPE_PCI     => Self::PCI,
            ffi::SENSORS_BUS_TYPE_SPI     => Self::SPI,
            ffi::SENSORS_BUS_TYPE_VIRTUAL => Self::VIRTUAL,
            ffi::SENSORS_BUS_TYPE_ACPI    => Self::ACPI,
            ffi::SENSORS_BUS_TYPE_HID     => Self::HID,
            ffi::SENSORS_BUS_TYPE_MDIO    => Self::MDIO,
            ffi::SENSORS_BUS_TYPE_SCSI    => Self::SCSI,
            x => return Err(Error::UnexpectedWildcard(x as i64))
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The Id of a bus hosting a Chip.
/// Note that this struct does not support wildcard values
pub struct BusId {
    pub type_: BusType,
    pub nr: c_short
}
impl TryFrom<sensors_bus_id> for BusId {
    type Error = Error;

    fn try_from(value: sensors_bus_id) -> std::result::Result<Self, Self::Error> {
        if value.nr == ffi::SENSORS_BUS_NR_ANY || value.nr == ffi::SENSORS_BUS_NR_IGNORE {
            return Err(Error::UnexpectedWildcard(value.nr as i64))
        }
        Ok(Self {
            type_: BusType::try_from(value.type_)?,
            nr: value.nr
        })
    }
}