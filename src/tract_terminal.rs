//! Data copying between tract types.

#![allow(dead_code)]

use std::ops::Range;
use ocl::core::{ClWaitList, ClEventPtrNew};
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



/// An OpenCL buffer backed source.
pub struct OclBufferSource<'b> {
    buf: &'b Buffer<u8>,
    offset: usize,
    dims: TractDims,
    events: Option<&'b EventList>,
}

impl<'b> OclBufferSource<'b> {
    pub fn new(buf: &'b Buffer<u8>, offset: Range<u32>, dims: TractDims,
            events: Option<&'b EventList>,) -> CmnResult<Self>
    {
        // [TODO]: Ensure buffer is sufficient size to handle offset range.
        // ~ debug_assert_eq!(buf.len(), dims.to_len());

        if dims.to_len() != offset.len() as usize {
            return Err(format!(" dims.to_len(): {} != offset.len(): \
            {}, (offset range: '{:?}').", dims.to_len(), offset.len(), offset).into())
        }

        Ok(OclBufferSource {
            buf: buf,
            dims: dims,
            offset: offset.start as usize,
            events: events,
        })
    }

    #[inline] pub fn buf(&self) -> &'b Buffer<u8> { self.buf }
    #[inline] pub fn offset(&self) -> usize { self.offset }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    #[inline] pub fn events(&self) -> &Option<&'b EventList> { &self.events }
}



/// An OpenCL buffer backed target.
pub struct OclBufferTarget<'b> {
    buf: &'b Buffer<u8>,
    offset: usize,
    dims: TractDims,
    events: Option<&'b mut EventList>,
}

impl<'b> OclBufferTarget<'b> {
    pub fn new(buf: &'b Buffer<u8>, offset: Range<u32>, dims: TractDims,
            mut events: Option<&'b mut EventList>) -> CmnResult<Self>
    {
        // [TODO]: Ensure buffer is sufficient size to handle offset range.
        // ~ debug_assert_eq!(buf.len(), dims.to_len());

        if dims.to_len() != offset.len() as usize {
            return Err(format!(" dims.to_len(): {} != offset.len(): \
            {}, (offset range: '{:?}').", dims.to_len(), offset.len(), offset).into())
        }

        if let Some(ref mut events) = events {
            events.clear_completed()?
        }

        Ok(OclBufferTarget {
            buf: buf,
            dims: dims,
            offset: offset.start as usize,
            events: events,
        })
    }

    pub fn copy_from_ocl_buffer<'e>(&mut self, source: OclBufferSource) -> CmnResult<()> {
        let mut en = Event::empty();

        source.buf().cmd().copy(self.buf, self.offset, self.dims.to_len())
            .offset(source.offset)
            .ewait_opt(source.events().map(|e| e as &ClWaitList))
            .enew_opt(if self.events.is_some() { Some(&mut en as &mut ClEventPtrNew) } else { None })
            .enq()?;

        if let Some(ref mut evl) = self.events {
            evl.push(en);
        }

        Ok(())
    }

    pub fn copy_from_slice_buffer<'e>(&mut self, source: SliceBufferSource) -> CmnResult<()> {
        let mut en = Event::empty();

        // self.axns.states.cmd().write(tract.frame()).offset(axn_range.start as usize)
        //     .block(false).ewait(wait_events).enew(new_events).enq().unwrap();
        self.buf().write(source.slice())
            .offset(self.offset)
            .block(false)
            .ewait_opt(source.events().map(|e| e as &ClWaitList))
            .enew_opt(if self.events.is_some() { Some(&mut en as &mut ClEventPtrNew) } else { None })
            .enq()?;

        if let Some(ref mut evl) = self.events {
            evl.push(en);
        }

        Ok(())
    }

    #[inline] pub fn buf(&mut self) -> &'b Buffer<u8> { self.buf }
    #[inline] pub fn offset(&self) -> usize { self.offset }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    #[inline] pub fn events(&mut self) -> &mut Option<&'b mut EventList> { &mut self.events }
}



/// A `Vec` (or array) backed source.
pub struct SliceBufferSource<'b> {
    slice: &'b [u8],
    dims: TractDims,
    events: Option<&'b EventList>,
}

impl<'b> SliceBufferSource<'b> {
    pub fn new(slice: &'b [u8], dims: TractDims, events: Option<&'b EventList>) -> CmnResult<Self> {
        // debug_assert_eq!(slice.len(), dims.to_len());

        Ok(SliceBufferSource {
            slice: slice,
            dims: dims,
            events: events,
        })
    }

    #[inline] pub fn slice(&self) -> &[u8] { self.slice }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    #[inline] pub fn events(&self) -> &Option<&'b EventList> { &self.events }
}



/// A `Vec` (or array) backed target.
pub struct SliceBufferTarget<'b> {
    slice: &'b mut [u8],
    dims: TractDims,
    events: Option<&'b mut EventList>,
}

impl<'b> SliceBufferTarget<'b> {
    pub fn new(slice: &'b mut [u8], dims: TractDims, mut events: Option<&'b mut EventList>)
            -> CmnResult<Self>
    {
        // debug_assert_eq!(slice.len(), dims.to_len());

        if let Some(ref mut events) = events {
            events.clear_completed()?
        }

        Ok(SliceBufferTarget {
            slice: slice,
            dims: dims,
            events: events,
        })
    }

    pub fn copy_from_ocl_buffer<'e>(&mut self, source: OclBufferSource) -> CmnResult<()> {
        let mut en = Event::empty();

        unsafe {
            source.buf().cmd().read_async(self.slice)
                .offset(source.offset())
                .ewait_opt(source.events().map(|e| e as &ClWaitList))
                .enew_opt(if self.events.is_some() { Some(&mut en as &mut ClEventPtrNew) } else { None })
                .enq()?;
        }

        if let Some(ref mut evl) = self.events {
            evl.push(en);
        }

        Ok(())
    }

    #[inline] pub fn slice(&mut self) -> &mut [u8] { self.slice }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    #[inline] pub fn events(&mut self) -> &mut Option<&'b mut EventList> { &mut self.events }
}

