//! Data copying between tract types.

#![allow(dead_code)]

use ocl::{Buffer, /*EventList*/};
use ::{TractDims, CmnResult};


trait CopyInto {

}


// pub struct Multitract {

// }


// pub struct TractTerminal {

// }


// pub struct TractDestination {

// }


pub struct TractTerminalOclBuffer<'b> {
    buf: &'b Buffer<u8>,
    dims: &'b TractDims,
}

impl<'b> TractTerminalOclBuffer<'b> {
    pub fn new(buf: &'b Buffer<u8>, dims: &'b TractDims) -> Self {
        TractTerminalOclBuffer {
            buf: buf,
            dims: dims,
        }
    }

    pub fn buf_mut(&mut self) -> &'b Buffer<u8> {
        self.buf
    }

    pub fn copy_into_slice(&'b self, mut tt_slice: TractTerminalSlice, offs: usize)
                -> CmnResult<()> {

        // [TODO]: BRING BACK: `.block(false).ewait(wait_events).enew(new_events).offset(...)`
        self.buf.cmd().write(tt_slice.slice_mut()).offset(offs).block(true)
            .enq().map_err(|e| e.into())
    }
}


pub struct TractTerminalSlice<'b> {
    slice: &'b mut [u8],
    dims: &'b TractDims,
}

impl<'b> TractTerminalSlice<'b> {
    pub fn new(slice: &'b mut [u8], dims: &'b TractDims) -> Self {
        TractTerminalSlice {
            slice: slice,
            dims: dims,
        }
    }

    #[inline]
    pub fn slice_mut(&mut self) -> &mut [u8] {
        self.slice
    }

    pub fn copy_into_ocl_buffer(&mut self, mut tt_buf: TractTerminalOclBuffer, offs: usize)
                -> CmnResult<()> {
        tt_buf.buf_mut().read(self.slice).offset(offs).block(true)
            .enq().map_err(|e| e.into())
    }
}

