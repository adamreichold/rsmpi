//! Describing data
//!
//! The core function of MPI is getting data from point A to point B (where A and B are e.g. single
//! processes, multiple processes, the filesystem, ...). It offers facilities to describe that data
//! (layout in memory, behavior under certain operators) that go beyound a start address and a
//! number of bytes.
//!
//! An MPI datatype describes a memory layout and semantics (e.g. in a collective reduce
//! operation). There are several pre-defined `SystemDatatype`s which directly correspond to Rust
//! primitive types, such as `MPI_DOUBLE` and `f64`. A direct relationship between a Rust type and
//! an MPI datatype is covered by the `EquivalentDatatype` trait. Starting from the
//! `SystemDatatype`s, the user can build various `UserDatatype`s, e.g. to describe the layout of a
//! struct (which should then implement `EquivalentDatatype`) or to intrusively describe parts of
//! an object in memory like all elements below the diagonal of a dense matrix stored in row-major
//! order.
//!
//! A `Buffer` describes a specific piece of data in memory that MPI should operate on. In addition
//! to specifying the datatype of the data. It knows the address in memory where the data begins
//! and how many instances of the datatype are contained in the data. The `Buffer` trait is
//! implemented for slices that contain types implementing `EquivalentDatatype`.
//!
//! In order to use arbitrary datatypes to describe the contents of a slice, the `View` type is
//! provided. However, since it can be used to instruct the underlying MPI implementation to
//! rummage around arbitrary parts of memory, its constructors are currently marked unsafe.
//!
//! # Unfinished features
//!
//! - **4.1.2**: Datatype constructors, `MPI_Type_create_hvector()`, `MPI_Type_indexed()`,
//! `MPI_Type_create_hindexed()`, `MPI_Type_create_indexed_block()`,
//! `MPI_Type_create_hindexed_block()`, `MPI_Type_create_struct()`
//! - **4.1.3**: Subarray datatype constructors, `MPI_Type_create_subarray()`,
//! - **4.1.4**: Distributed array datatype constructors, `MPI_Type_create_darray()`
//! - **4.1.5**: Address and size functions, `MPI_Get_address()`, `MPI_Aint_add()`,
//! `MPI_Aint_diff()`, `MPI_Type_size()`, `MPI_Type_size_x()`
//! - **4.1.7**: Extent and bounds of datatypes: `MPI_Type_get_extent()`,
//! `MPI_Type_get_extent_x()`, `MPI_Type_create_resized()`
//! - **4.1.8**: True extent of datatypes, `MPI_Type_get_true_extent()`,
//! `MPI_Type_get_true_extent_x()`
//! - **4.1.10**: Duplicating a datatype, `MPI_Type_dup()`
//! - **4.1.11**: `MPI_Get_elements()`, `MPI_Get_elements_x()`
//! - **4.1.13**: Decoding a datatype, `MPI_Type_get_envelope()`, `MPI_Type_get_contents()`
//! - **4.2**: Pack and unpack, `MPI_Pack()`, `MPI_Unpack()`, `MPI_Pack_size()`
//! - **4.3**: Canonical pack and unpack, `MPI_Pack_external()`, `MPI_Unpack_external()`,
//! `MPI_Pack_external_size()`

use std::{mem};

use libc::{c_void};

use ::Count;
use ffi;
use ffi::MPI_Datatype;

pub mod traits;

/// Can identify as an `MPI_Datatype`
pub trait RawDatatype {
    unsafe fn raw(&self) -> MPI_Datatype;
}

impl<'a, D: RawDatatype> RawDatatype for &'a D {
    unsafe fn raw(&self) -> MPI_Datatype {
        (*self).raw()
    }
}

/// A system datatype, e.g. `MPI_FLOAT`
///
/// # Standard section(s)
///
/// 3.2.2
#[derive(Copy, Clone)]
pub struct SystemDatatype(MPI_Datatype);

impl RawDatatype for SystemDatatype {
    unsafe fn raw(&self) -> MPI_Datatype {
        self.0
    }
}

/// A direct equivalence exists between the implementing type and an MPI datatype
///
/// # Standard section(s)
///
/// 3.2.2
pub trait EquivalentDatatype {
    type Out: RawDatatype;
    fn equivalent_datatype() -> Self::Out;
}

macro_rules! equivalent_system_datatype {
    ($rstype:path, $mpitype:path) => (
        impl EquivalentDatatype for $rstype {
            type Out = SystemDatatype;
            fn equivalent_datatype() -> Self::Out { SystemDatatype($mpitype) }
        }
    )
}

equivalent_system_datatype!(f32, ffi::RSMPI_FLOAT);
equivalent_system_datatype!(f64, ffi::RSMPI_DOUBLE);

equivalent_system_datatype!(i8, ffi::RSMPI_INT8_T);
equivalent_system_datatype!(i16, ffi::RSMPI_INT16_T);
equivalent_system_datatype!(i32, ffi::RSMPI_INT32_T);
equivalent_system_datatype!(i64, ffi::RSMPI_INT64_T);

equivalent_system_datatype!(u8, ffi::RSMPI_UINT8_T);
equivalent_system_datatype!(u16, ffi::RSMPI_UINT16_T);
equivalent_system_datatype!(u32, ffi::RSMPI_UINT32_T);
equivalent_system_datatype!(u64, ffi::RSMPI_UINT64_T);

/// A user defined MPI datatype
///
/// # Standard section(s)
///
/// 4
pub struct UserDatatype(MPI_Datatype);

impl UserDatatype {
    /// Constructs a new datatype by concatenating `count` repetitions of `oldtype`
    ///
    /// # Examples
    /// See `examples/contiguous.rs`
    ///
    /// # Standard section(s)
    ///
    /// 4.1.2
    pub fn contiguous<D: RawDatatype>(count: Count, oldtype: D) -> UserDatatype {
        let mut newtype: MPI_Datatype = unsafe { mem::uninitialized() };
        unsafe {
            ffi::MPI_Type_contiguous(count, oldtype.raw(), &mut newtype as *mut MPI_Datatype);
            ffi::MPI_Type_commit(&mut newtype as *mut MPI_Datatype);
        }
        UserDatatype(newtype)
    }

    /// Construct a new datatype out of `count` blocks of `blocklength` elements of `oldtype`
    /// concatenated with the start of consecutive blocks placed `stride` elements apart.
    ///
    /// # Examples
    /// See `examples/vector.rs`
    ///
    /// # Standard section(s)
    ///
    /// 4.1.2
    pub fn vector<D: RawDatatype>(count: Count, blocklength: Count, stride: Count, oldtype: D) -> UserDatatype {
        let mut newtype: MPI_Datatype = unsafe { mem::uninitialized() };
        unsafe {
            ffi::MPI_Type_vector(count, blocklength, stride, oldtype.raw(),
                &mut newtype as *mut MPI_Datatype);
            ffi::MPI_Type_commit(&mut newtype as *mut MPI_Datatype);
        }
        UserDatatype(newtype)
    }
}

impl RawDatatype for UserDatatype {
    unsafe fn raw(&self) -> MPI_Datatype {
        self.0
    }
}

impl Drop for UserDatatype {
    fn drop(&mut self) {
        unsafe {
            ffi::MPI_Type_free(&mut self.0 as *mut MPI_Datatype);
        }
        assert_eq!(self.0, ffi::RSMPI_DATATYPE_NULL);
    }
}

/// Something that has an associated datatype
// TODO: merge this into Buffer, maybe?
pub trait Datatype {
    type Out: RawDatatype;
    fn datatype(&self) -> Self::Out;
}

impl<T> Datatype for T where T: EquivalentDatatype {
    type Out = <T as EquivalentDatatype>::Out;
    fn datatype(&self) -> Self::Out { <T as EquivalentDatatype>::equivalent_datatype() }
}

impl<T> Datatype for [T] where T: EquivalentDatatype {
    type Out = <T as EquivalentDatatype>::Out;
    fn datatype(&self) -> Self::Out { <T as EquivalentDatatype>::equivalent_datatype() }
}

/// A buffer is a region in memory that starts at `send_address()` (or `receive_address()`) and
/// contains `count()` copies of `datatype()`.
pub trait Buffer: Datatype {
    fn count(&self) -> Count;
    unsafe fn send_address(&self) -> *const c_void;
    unsafe fn receive_address(&mut self) -> *mut c_void;
}

impl<T> Buffer for T where T: EquivalentDatatype {
    fn count(&self) -> Count { 1 }
    unsafe fn send_address(&self) -> *const c_void { mem::transmute(self as *const T) }
    unsafe fn receive_address(&mut self) -> *mut c_void { mem::transmute(self as *mut T) }
}

impl<T> Buffer for [T] where T: EquivalentDatatype {
    fn count(&self) -> Count {
        self.len() as Count // FIXME: this should be a checked cast.
    }
    unsafe fn send_address(&self) -> *const c_void { mem::transmute(self.as_ptr()) }
    unsafe fn receive_address(&mut self) -> *mut c_void { mem::transmute(self.as_mut_ptr()) }
}

/// A buffer with a user specified count and datatype
///
/// # Safety
///
/// Views can be used to instruct the underlying MPI library to rummage around at arbitrary
/// locations in memory. This might be controlled later on using datatype bounds an slice lengths
/// but for now, all View constructors are marked `unsafe`.
pub struct View<'a, 'b, T: 'a, D: 'b>
where D: RawDatatype {
    datatype: &'b D,
    count: Count,
    buffer: &'a mut [T]
}

impl<'a, 'b, T: 'a, D: 'b> View<'a, 'b, T, D>
where D: RawDatatype {
    /// Return a view of the slice `buffer` containing `count` instances of MPI datatype
    /// `datatype`.
    ///
    /// # Examples
    /// See `examples/contiguous.rs`, `examples/vector.rs`
    pub unsafe fn with_count_and_datatype(buffer: &'a mut [T], count: Count, datatype: &'b D) -> View<'a, 'b, T, D> {
        View { datatype: datatype, count: count, buffer: buffer }
    }
}

impl<'a, 'b, T: 'a, D: 'b> Datatype for View<'a, 'b, T, D>
where D: RawDatatype {
    type Out = &'b D;
    fn datatype(&self) -> Self::Out { self.datatype }
}

impl<'a, 'b, T: 'a, D: 'b> Buffer for View<'a, 'b, T, D>
where D: RawDatatype {
    fn count(&self) -> Count { self.count }
    unsafe fn send_address(&self) -> *const c_void { mem::transmute(self.buffer.as_ptr()) }
    unsafe fn receive_address(&mut self) -> *mut c_void { mem::transmute(self.buffer.as_mut_ptr()) }
}