
use ocl::{Buffer, OclPrm};

pub fn buffer_uid<T: OclPrm>(buf: &Buffer<T>) -> u64 {
    unsafe { buf.core_as_ref().as_ptr() as u64 }
}