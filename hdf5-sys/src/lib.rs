#![allow(non_camel_case_types, non_snake_case, dead_code, deprecated)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_safety_doc))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cognitive_complexity))]

macro_rules! extern_static {
    ($dest:ident, $src:ident) => {
        extern "C" {
            static $src: id_t;
        }
        pub static $dest: &'static id_t = unsafe { &$src };
    };
}

#[cfg(all(feature = "mpio", not(h5_have_parallel)))]
compile_error!("Enabling \"mpio\" feature requires HDF5 library built with MPI support");

pub mod h5;
pub mod h5a;
pub mod h5ac;
pub mod h5c;
pub mod h5d;
pub mod h5e;
pub mod h5f;
pub mod h5fd;
pub mod h5g;
pub mod h5i;
pub mod h5l;
pub mod h5mm;
pub mod h5o;
pub mod h5p;
pub mod h5r;
pub mod h5s;
pub mod h5t;
pub mod h5z;

#[cfg(hdf5_1_8_15)]
pub mod h5pl;

#[allow(non_camel_case_types)]
mod internal_prelude {
    pub use crate::h5::{
        haddr_t, hbool_t, herr_t, hsize_t, hssize_t, htri_t, H5_ih_info_t, H5_index_t,
        H5_iter_order_t,
    };
    pub use crate::h5i::hid_t;
    pub use crate::h5t::H5T_cset_t;
    pub use libc::{int64_t, off_t, size_t, ssize_t, time_t, uint32_t, uint64_t, FILE};
    pub use std::os::raw::{
        c_char, c_double, c_float, c_int, c_long, c_longlong, c_uchar, c_uint, c_ulong,
        c_ulonglong, c_void,
    };
}

#[doc(hidden)]
macro_rules! check_and_emit {
    ($flag:ident) => {
        if cfg!($flag) {
            println!("cargo:rustc-cfg={}", stringify!($flag));
        }
    };
}

#[doc(hidden)]
pub fn emit_cfg_flags() {
    check_and_emit!(hdf5_1_10_5);
    check_and_emit!(h5_have_direct);
    check_and_emit!(h5_have_parallel);
    check_and_emit!(h5_have_threadsafe);
}

#[cfg(test)]
mod tests {
    use super::h5p::H5P_CLS_ROOT;
    use crate::h5::{H5open, H5get_libversion, H5close};
    use std::os::raw::c_uint;

    #[test]
    pub fn test_smoke() {
        unsafe {
            H5open();
            let mut majnum:c_uint = 0;
            let mut minnum:c_uint = 0;
            let mut relnum:c_uint = 0;
            let result = H5get_libversion(&mut majnum, &mut minnum, &mut relnum);
            println!("Call to: H5get_libversion returned: {}", result);
            println!("H5 library version: {}.{}.{}", majnum, minnum, relnum);
            H5close();
            assert!(*H5P_CLS_ROOT > 0);
        }
    }
}
