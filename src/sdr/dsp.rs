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

impl<T: AsyncRead> IQMagnitudeReader<T> {
    pin_utils::unsafe_pinned!(reader: T);

    pub fn new(inner: T) -> IQMagnitudeReader<T> {
        IQMagnitudeReader { reader: inner }
    }
}

impl<T: AsyncRead> AsyncRead for IQMagnitudeReader<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        trace!("iqmagnituderader poll");
        let Self { reader } = unsafe { self.get_unchecked_mut() };
        let inner = unsafe { Pin::new_unchecked(reader) };

        let mut inner_buf = [0u8; 256000];
        let mut inner_bytebuf = ReadBuf::new(&mut inner_buf);
        match inner.poll_read(cx, &mut inner_bytebuf) {
            Poll::Pending => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(Ok(())) => {
                trace!("IQMagnitudeReader got samples, converting");
                let filled = inner_bytebuf.filled();
                let ptr = filled.as_ptr() as *const IQ;
                let iq = unsafe { std::slice::from_raw_parts::<IQ>(ptr, filled.len() / 2) };

                let magnitude_samples: Vec<u8> = iq.iter().map(|iq| iq.magnitude()).collect();
                let dst = buf.initialize_unfilled_to(magnitude_samples.len());
                dst.copy_from_slice(&magnitude_samples);
                buf.advance(magnitude_samples.len());
                inner_bytebuf.clear();
            }
            Poll::Ready(e) => return Poll::Ready(e),
        }
        //   let mut remaining = buf.initialize_unfilled();
        //   buf.advance(n);

        Poll::Ready(Ok(()))
    }
}
