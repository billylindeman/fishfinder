use log::*;
use ringbuf::{Consumer, RingBuffer};
use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use tokio::{
    io::{AsyncRead, ReadBuf},
    task,
};

#[repr(C)]
pub struct IQ {
    pub i: u8,
    pub q: u8,
}

impl IQ {
    pub fn magnitude(&self) -> u8 {
        let i: f32 = (self.i as i16 - 127 as i16).into();
        let q: f32 = (self.i as i16 - 127 as i16).into();
        let mag: u8 = (i * i + q * q).sqrt().round() as u8;
        return mag;
    }
}

pub struct IQMagnitudeReader<T: AsyncRead> {
    reader: T,
}

impl<T: AsyncRead> AsyncRead for IQMagnitudeReader<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        trace!("iqmagnituderader poll");

        //   let mut remaining = buf.initialize_unfilled();
        //   buf.advance(n);

        Poll::Ready(Ok(()))
    }
}
