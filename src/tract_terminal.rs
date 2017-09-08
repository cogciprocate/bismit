//! Data copying between tract types.

#![allow(dead_code, unused_imports)]

use std::ops::Range;
use ocl::core::{ClWaitListPtr, ClNullEventPtr};
use ocl::builders::{ClWaitListPtrEnum, ClNullEventPtrEnum};
use ocl::{Buffer, EventList, Event, Queue, FutureReadGuard, FutureWriteGuard};
use ::{TractDims, Result as CmnResult};
// use map::ExecutionGraph;


trait CopyFrom {}



pub enum SourceKind {
    Reader(FutureReadGuard<u8>),
}

pub struct TerminalSource {
    kind: SourceKind,
}



pub enum TargetKind {
    Writer(FutureWriteGuard<u8>),
}

pub struct TerminalTarget {
    kind: TargetKind,
}

// impl TerminalTarget {


//     fn read_from_source(&mut self, src: TerminalSource) -> CmnResult<Event> {
//         let mut ev = Event::empty();

//         match self.kind {
//             TargetKind::Writer(writer) => {
//                 self.buf.write(src)
//                 .offset(self.offset)
//                 .ewait_opt(wait_list)
//                 // .enew_opt(ev.as_mut())
//                 // .enew_opt(if self.events.is_some() || self.event.is_some()
//                 //     { Some(&mut ev) } else { None })
//                 .enew(&mut ev)
//                 .enq()?;
//             }
//         }

//         Ok(ev)
//     }
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
        // * TODO: Ensure buffer is sufficient size to handle offset range.
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

    #[inline] pub fn buf(&self) -> &Buffer<u8> { self.buf }
    #[inline] pub fn offset(&self) -> usize { self.offset }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    #[inline] pub fn events(&self) -> Option<&EventList> { self.events }
}


// Option<&'b ClWaitListPtr>

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
        // * TODO: Ensure buffer is sufficient size to handle offset range.
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

    pub fn copy_from_ocl_buffer(&mut self, source: OclBufferSource)
            -> CmnResult<&'b mut OclBufferTarget>
    {
        let mut ev = Event::empty();

        source.buf().cmd().copy(self.buf, Some(self.offset), Some(self.dims.to_len()))
            .offset(source.offset)
            .ewait_opt(source.events().clone())
            .enew_opt(if self.events.is_some() || self.event.is_some()
                { Some(&mut ev) } else { None })
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

        unsafe {
            self.buf.write(source.slice())
                .offset(self.offset)
                .block(false)
                .ewait_opt(source.events())
                .enew_opt(if self.events.is_some() || self.event.is_some()
                    { Some(&mut ev) } else { None })
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

    pub fn copy_from_slice_buffer_v2<'e, Ewl>(&'e mut self, source: SliceBufferSource,
            wait_list: Option<Ewl>) -> CmnResult<Event>
            // wait_list: Option<Ewl>) -> CmnResult<Event>
            where Ewl: Into<ClWaitListPtrEnum<'e>>
    {
        // let mut ev = if self.events.is_some() || self.event.is_some() {
        //     Some(Event::empty())
        // } else {
        //     None
        // };

        let mut ev = Event::empty();

        unsafe {
            self.buf.write(source.slice())
                .offset(self.offset)
                .block(false)
                .ewait_opt(wait_list)
                // .enew_opt(ev.as_mut())
                // .enew_opt(if self.events.is_some() || self.event.is_some()
                //     { Some(&mut ev) } else { None })
                .enew(&mut ev)
                .enq()?;
        }

        Ok(ev)
    }

    #[inline] pub fn buf(&mut self) -> &Buffer<u8> { self.buf }
    #[inline] pub fn offset(&self) -> usize { self.offset }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    // #[inline] pub fn events(&mut self) -> Option<&mut EventList> { self.events.clone() }
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
    #[inline] pub fn events(&self) -> Option<&EventList> { self.events }
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

        let slice = unsafe { ::std::slice::from_raw_parts_mut(self.slice.as_mut_ptr(),
            self.slice.len()) };

        unsafe {
            source.buf.cmd().read(slice)
                .block(false)
                .offset(source.offset())
                .ewait_opt(source.events())
                .enew_opt(if self.events.is_some() || self.event.is_some()
                    { Some(&mut ev) } else { None })
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

    pub fn copy_from_ocl_buffer_v2<'e, Ewl>(&mut self, source: OclBufferSource,
            wait_list: Option<Ewl>, read_queue: Option<&Queue>)
            -> CmnResult<Event>
            where Ewl: Into<ClWaitListPtrEnum<'e>>
    {
        // let mut ev = Event::empty();

        // let mut ev = if self.events.is_some() || self.event.is_some() {
        //     Some(Event::empty())
        // } else {
        //     None
        // };

        let mut ev = Event::empty();

        let slice = unsafe { ::std::slice::from_raw_parts_mut(self.slice.as_mut_ptr(),
            self.slice.len()) };

        {
            let mut cmd = source.buf.cmd().read(slice)
                .offset(source.offset())
                .ewait_opt(wait_list)
                // .enew(&mut ev);
                // .enew_opt(ev.as_mut());
                .enew(&mut ev);

            if let Some(rq) = read_queue {
                cmd = cmd.queue(rq);
            }

            cmd.enq()?;
        }

        Ok(ev)
    }

    #[inline] pub fn slice(&mut self) -> &mut [u8] { self.slice }
    #[inline] pub fn dims(&self) -> &TractDims { &self.dims }
    // #[inline] pub fn events(&mut self) -> Option<&mut EventList> { self.events }
}

