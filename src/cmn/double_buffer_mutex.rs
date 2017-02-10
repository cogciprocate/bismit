
////// [TEMPORARY]:
#![allow(dead_code)]


use std::default::Default;


pub struct DoubleBufferMutex<T> {
    buffers: Vec<T>,
}

impl<T: Default + Clone> DoubleBufferMutex<T> {
    pub fn new(buffer_len: usize) -> DoubleBufferMutex<T> {
        DoubleBufferMutex {
            buffers: vec![Default::default(); buffer_len * 2],
        }
    }
}

