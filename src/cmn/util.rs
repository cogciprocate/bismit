
use ocl::{Buffer, OclPrm};

pub fn buffer_uid<T: OclPrm>(buf: &Buffer<T>) -> u64 {
    buf.core().as_ptr() as u64
}