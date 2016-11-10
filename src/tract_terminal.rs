//! Data copying between tract types.

#![allow(dead_code)]

// use std::ops::Range;
// use std::ops::Deref;
use ocl::core::{self, ClWaitList, ClEventPtrNew};
use ocl::{Buffer, EventList, Event};
use ::{TractDims, CmnResult};


trait CopyFrom {

}


// pub struct Multitract {

// }


// pub struct TractTerminal {

// }


// pub struct TractDestination {

// }


pub struct TractTerminalOclBuffer<'b> {
    buf: &'b Buffer<u8>,
    offset: usize,
    dims: TractDims,
    events: Option<&'b mut EventList>,
}

impl<'b> TractTerminalOclBuffer<'b> {
    pub fn new(buf: &'b Buffer<u8>, offset: usize, dims: TractDims,
            events: Option<&'b mut EventList>,) -> Self
    {
        // debug_assert_eq!(buf.len(), dims.to_len());

        TractTerminalOclBuffer {
            buf: buf,
            dims: dims,
            offset: offset,
            events: events,
        }
    }

    // pub fn copy_to_slice<'e>(&'b self, mut tt_slice: TractTerminalSlice,
    //         wait_events: Option<&'e EventList>) -> CmnResult<()>
    // {
    //     // [TODO]: BRING BACK: `.block(false).ewait(wait_events).enew(new_events).offset(...)`
    //     unsafe { self.buf.cmd().read_async(tt_slice.slice())
    //         .offset(self.offset)
    //         .ewait_opt(wait_events.map(|e| e as &ClWaitList))
    //         // .enew_opt(match tt_slice.events() {
    //         //         &mut Some(e) => Some(e as &mut ClEventPtrNew),
    //         //         &mut None => None,
    //         //     })
    //         .enq().map_err(|e| e.into()) }
    // }

    pub fn copy_from_ocl_buffer<'e>(&mut self, mut tt_buf: TractTerminalOclBuffer,
        wait_events: Option<&'e EventList>) -> CmnResult<()>
    {
        // let evl = match self.events {
        //     Some(ref mut evl) => {
        //         let wlp = (*evl) as *mut EventList as *mut _ as *mut ClEventPtrNew;
        //         &mut (*wlp) as &mut ClEventPtrNew
        //     },
        //     None =>
        // }
        let mut en = Event::empty();

        try!(tt_buf.buf().cmd().copy(self.buf, self.offset, self.dims.to_len())
            .offset(tt_buf.offset)
            .ewait_opt(wait_events.map(|e| e as &ClWaitList))
            .enew_opt(if self.events.is_some() { Some(&mut en as &mut ClEventPtrNew) } else { None })
            .enq());

        if let Some(ref mut evl) = self.events {
            evl.push(en);
        }
        // try!(check_len(self.mem_len, len, offset));
        // core::enqueue_copy_buffer::<u8>(self.buf.default_queue(),
        //         self.buf.core_as_ref(), tt_buf.buf().core_as_ref(),
        //         tt_buf.offset(), self.offset, tt_buf.dims().to_len(),
        //         wait_events.map(|e| e as &ClWaitList),
        //         self.events.map(|e| e as &mut ClEventPtrNew))
        //     .map_err(|e| e.into())
        Ok(())
    }

    pub fn buf(&mut self) -> &'b Buffer<u8> {
        self.buf
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn dims(&self) -> &TractDims {
        &self.dims
    }
}



pub struct TractTerminalSlice<'b> {
    slice: &'b mut [u8],
    dims: TractDims,
    events: Option<&'b mut EventList>,
}

impl<'b> TractTerminalSlice<'b> {
    pub fn new(slice: &'b mut [u8], dims: TractDims, events: Option<&'b mut EventList>) -> Self {
        // debug_assert_eq!(slice.len(), dims.to_len());

        TractTerminalSlice {
            slice: slice,
            dims: dims,
            events: events,
        }
    }

    // pub fn copy_to_ocl_buffer(&mut self, mut tt_buf: TractTerminalOclBuffer)
    //         -> CmnResult<()>
    // {
    //     tt_buf.buf().write(self.slice).offset(tt_buf.offset()).block(true)
    //         .enq().map_err(|e| e.into())
    // }

    pub fn copy_from_ocl_buffer<'e>(&mut self, mut tt_buf: TractTerminalOclBuffer,
            wait_events: Option<&'e EventList>) -> CmnResult<()>
    {
        // .enew_opt(if self.events.is_some() { Some(&mut en as &mut ClEventPtrNew) } else { None })

        // if let Some(ref mut evl) = self.events {
        let mut en = Event::empty();

        unsafe {
            try!(tt_buf.buf().cmd().read_async(self.slice)
                .offset(tt_buf.offset())
                .ewait_opt(wait_events.map(|e| e as &ClWaitList))
                // .enew(&mut en)
                .enew_opt(if self.events.is_some() { Some(&mut en as &mut ClEventPtrNew) } else { None })
                .enq());
        }

        if let Some(ref mut evl) = self.events {
            evl.push(en);
        }
        // } else {
        //     unsafe {
        //         try!(tt_buf.buf().cmd().read_async(self.slice)
        //             .offset(tt_buf.offset())
        //             .ewait_opt(wait_events.map(|e| e as &ClWaitList))
        //             .enq());
        //     }
        // }

        Ok(())
    }

    #[inline]
    pub fn slice(&mut self) -> &mut [u8] {
        self.slice
    }

    #[inline]
    pub fn dims(&self) -> &TractDims {
        &self.dims
    }

    #[inline]
    pub fn events(&mut self) -> &mut Option<&'b mut EventList> {
        &mut self.events
    }

    pub fn clear_completed_events(&mut self) -> CmnResult<()> {
        if let Some(ref mut events) = self.events {
            events.clear_completed().map_err(|e| e.into())
        } else {
            Ok(())
        }
    }
}

