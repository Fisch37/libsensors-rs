#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// use std::ffi::{c_char, c_int, c_short};

// #[repr(C)]
// pub struct sensors_chip_name {
//     pub prefix: *mut c_char,
//     pub bus: sensors_bus_id,
//     pub addr: c_int,
//     pub path: *mut c_char
// }

// #[repr(C)]
// pub struct sensors_bus_id {
//     pub type_: c_short,
//     pub nr: c_short
// }

// #[repr(C)]
// pub struct sensors_feature {
//     pub name: *mut c_char,
//     pub number: c_int,
//     pub type_: sensors_feature_type,
//     // libsensors internal
//     first_subfeature: c_int,
//     padding1: c_int
// }

// #[repr(C)]
// pub enum sensors_feature_type {
// 	SENSORS_FEATURE_IN		= 0x00,
// 	SENSORS_FEATURE_FAN		= 0x01,
// 	SENSORS_FEATURE_TEMP		= 0x02,
// 	SENSORS_FEATURE_POWER		= 0x03,
// 	SENSORS_FEATURE_ENERGY		= 0x04,
// 	SENSORS_FEATURE_CURR		= 0x05,
// 	SENSORS_FEATURE_HUMIDITY	= 0x06,
// 	SENSORS_FEATURE_MAX_MAIN,
// 	SENSORS_FEATURE_VID		= 0x10,
// 	SENSORS_FEATURE_INTRUSION	= 0x11,
// 	SENSORS_FEATURE_MAX_OTHER,
// 	SENSORS_FEATURE_BEEP_ENABLE	= 0x18,
// 	SENSORS_FEATURE_MAX,
// 	SENSORS_FEATURE_UNKNOWN		= i32::MAX,
// } 
