#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[doc(hidden)]
pub mod __reexports {
    pub use cstr::cstr;
}

#[macro_export]
macro_rules! mprint {
    ($fmt:expr $(, $args:expr)*) => {
	mprint($crate::__reexports::cstr!($fmt).as_ptr()  $(, $args)*)
    };
}
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
