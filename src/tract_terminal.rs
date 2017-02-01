//! Data copying between tract types.

#![allow(dead_code)]

use std::ops::Range;
use ocl::core::{ClWaitList, ClEventPtrNew};
use ocl::{Buffer, EventList, Event};
use ::{TractDims, Result as CmnResult};
// use map::ExecutionGraph;


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


// Option<&'b ClWaitList>

/// An OpenCL buffer backed target.
pub struct OclBufferTarget<'b> {
    buf: &'b Buffer<u8>,
    offset: usize,
    dims: TractDims,
    events: Option<&'b mut EventList>,
    event: Option<Event>,
}

impl<'b> OclBufferTarget<'b> {
    pub fn new(buf: &'b Buffer<u8>, offset: Range<u32>, dims: TractDims,
            mut events: Option<&'b mut EventList>, store_event: bool) -> CmnResult<Self>
    {
        // [TODO]: Ensure buffer is sufficient size to handle offset range.
        // ~ debug_assert_eq!(buf.len(), dims.to_len());
        let event = if store_event { Some(Event::empty()) } else { None };

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
            event: event,
        })
    }

    pub fn copy_from_ocl_buffer(&'b mut self, source: OclBufferSource)
            -> CmnResult<&'b mut OclBufferTarget>
    {
        // let mut ev = match self.event {
        //     Some(ev) => ev,
        //     None => Event::empty(),
        // };

        let mut ev = Event::empty();

        source.buf().cmd().copy(self.buf, self.offset, self.dims.to_len())
            .offset(source.offset)
            .ewait_opt(source.events().map(|evl| evl as &ClWaitList))
            .enew_opt(if self.events.is_some() || self.event.is_some()
                { Some(&mut ev as &mut ClEventPtrNew) } else { None })
            .enq()?;

        if let Some(ref mut evl) = self.events {
            evl.push(ev.clone());
        }

        if self.event.is_some() {
            self.event = Some(ev);
        }

        Ok(self)
    }

    pub fn copy_from_slice_buffer(&'b mut self, source: SliceBufferSource)
            -> CmnResult<&'b mut OclBufferTarget>
    {
        let mut ev = Event::empty();

        // self.axns.states.cmd().write(tract.frame()).offset(axn_range.start as usize)
        //     .block(false).ewait(wait_events).enew(new_events).enq().unwrap();
        self.buf().write(source.slice())
            .offset(self.offset)
            .block(false)
            .ewait_opt(source.events().map(|e| e as &ClWaitList))
            .enew_opt(if self.events.is_some() || self.event.is_some()
                { Some(&mut ev as &mut ClEventPtrNew) } else { None })
            .enq()?;

        if let Some(ref mut evl) = self.events {
            evl.push(ev.clone());
        }

        if self.event.is_some() {
            self.event = Some(ev);
        }

        Ok(self)
    }

    pub fn copy_from_slice_buffer_v2(&'b mut self, source: SliceBufferSource, wait_list: Option<&ClWaitList>)
            -> CmnResult<Event>
    {
        let mut ev = Event::empty();

        // self.axns.states.cmd().write(tract.frame()).offset(axn_range.start as usize)
        //     .block(false).ewait(wait_events).enew(new_events).enq().unwrap();
        self.buf().write(source.slice())
            .offset(self.offset)
            .block(false)
            // .ewait_opt(source.events().map(|e| e as &ClWaitList))
            .ewait_opt(wait_list)
            .enew_opt(if self.events.is_some() || self.event.is_some()
                { Some(&mut ev as &mut ClEventPtrNew) } else { None })
            .enq()?;

        // if let Some(ref mut evl) = self.events {
        //     evl.push(ev.clone());
        // }

        // if self.event.is_some() {
        //     self.event = Some(ev);
        // }

        Ok(ev)
    }

    #[inline] pub fn buf(&mut self) -> &'b Buffer<u8> { self.buf }
    #[inline] pub fn offset(&self) -> usize { self.offset }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    #[inline] pub fn events(&mut self) -> &mut Option<&'b mut EventList> { &mut self.events }
    #[inline] pub fn event(&self) -> Option<Event> { self.event.clone() }
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
    event: Option<Event>,
}

impl<'b> SliceBufferTarget<'b> {
    pub fn new(slice: &'b mut [u8], dims: TractDims, mut events: Option<&'b mut EventList>,
            store_event: bool) -> CmnResult<Self>
    {
        // debug_assert_eq!(slice.len(), dims.to_len());
        let event = if store_event { Some(Event::empty()) } else { None };

        if let Some(ref mut events) = events {
            events.clear_completed()?
        }

        Ok(SliceBufferTarget {
            slice: slice,
            dims: dims,
            events: events,
            event: event,
        })
    }

    pub fn copy_from_ocl_buffer(&'b mut self, source: OclBufferSource)
            -> CmnResult<&'b mut SliceBufferTarget>
    {
        let mut ev = Event::empty();

        unsafe {
            source.buf().cmd().read_async(self.slice)
                .offset(source.offset())
                .ewait_opt(source.events().map(|e| e as &ClWaitList))
                .enew_opt(if self.events.is_some() || self.event.is_some()
                    { Some(&mut ev as &mut ClEventPtrNew) } else { None })
                .enq()?;
        }

        if let Some(ref mut evl) = self.events {
            evl.push(ev.clone());
        }

        if self.event.is_some() {
            self.event = Some(ev);
        }

        Ok(self)
    }

    pub fn copy_from_ocl_buffer_v2(&'b mut self, source: OclBufferSource, wait_list: Option<&ClWaitList>)
            -> CmnResult<Event>
    {
        let mut ev = Event::empty();

        unsafe {
            source.buf().cmd().read_async(self.slice)
                .offset(source.offset())
                // .ewait_opt(source.events().map(|e| e as &ClWaitList))
                .ewait_opt(wait_list)
                .enew(&mut ev as &mut ClEventPtrNew)
                .enq()?;
        }

        // if let Some(ref mut evl) = self.events {
        //     evl.push(ev.clone());
        // }

        // if self.event.is_some() {
        //     self.event = Some(ev);
        // }

        Ok(ev)
    }

    #[inline] pub fn slice(&mut self) -> &mut [u8] { self.slice }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    #[inline] pub fn events(&mut self) -> &mut Option<&'b mut EventList> { &mut self.events }
}

