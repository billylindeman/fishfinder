use log::*;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

#[repr(C)]
pub struct IQ {
    pub i: u8,
    pub q: u8,
}

impl IQ {
    pub fn magnitude(&self) -> u8 {
        let i: f32 = (self.i as i16 - 127 as i16).into();
        let q: f32 = (self.q as i16 - 127 as i16).into();
        let mag: u8 = (i * i + q * q).sqrt().round() as u8;
        return mag;
    }
}

pub struct IQMagnitudeReader<T: AsyncRead> {
    inner: T,
}

impl<T: AsyncRead> IQMagnitudeReader<T> {
    pin_utils::unsafe_pinned!(inner: T);

    pub fn new(inner: T) -> IQMagnitudeReader<T> {
        IQMagnitudeReader { inner: inner }
    }
}

impl<T: AsyncRead> AsyncRead for IQMagnitudeReader<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        trace!("IQMagnitudeReader poll_read");

        let Self { inner } = unsafe { self.get_unchecked_mut() };
        let inner = unsafe { Pin::new_unchecked(inner) };

        // create a buffer that is 2 * the size of our upstream buffer
        // [i, q] -> [m]
        let mut inner_buf = vec![0u8; buf.remaining() * 2];
        let mut inner_bytebuf = ReadBuf::new(&mut inner_buf);

        match inner.poll_read(cx, &mut inner_bytebuf) {
            Poll::Pending => {
                // cx.waker() gets scheduled by inner impl
                return Poll::Pending;
            }
            Poll::Ready(Ok(())) => {
                let filled = inner_bytebuf.filled();
                trace!(
                    "IQMagnitudeReader got {} iq-samples, calculating magnitudes",
                    filled.len()
                );
                let ptr = filled.as_ptr() as *const IQ;
                let iq = unsafe { std::slice::from_raw_parts::<IQ>(ptr, filled.len() / 2) };

                let magnitude_samples: Vec<u8> = iq.iter().map(|iq| iq.magnitude()).collect();

                let dst = buf.initialize_unfilled_to(magnitude_samples.len());
                dst.copy_from_slice(&magnitude_samples);
                buf.advance(magnitude_samples.len());

                trace!(
                    "IQMagnitudeReader wrote {} magnitudes into buf",
                    magnitude_samples.len()
                );
                inner_bytebuf.clear();
            }
            Poll::Ready(e) => return Poll::Ready(e),
        }

        Poll::Ready(Ok(()))
    }
}
