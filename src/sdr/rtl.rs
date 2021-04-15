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

pub const RTL_SDR_BUFFER_SIZE: usize = 512000;

pub struct RadioConfig {
    device_index: u8,
    sample_rate: u32,
    center_freq: u32,
    ppm: i32,
}

impl RadioConfig {
    pub fn mode_s(device_index: u8) -> RadioConfig {
        RadioConfig {
            device_index: device_index,
            sample_rate: 2_000_000,
            center_freq: 1_090_000_000,
            ppm: 0,
        }
    }
}

pub struct Radio {
    consumer: Consumer<u8>,
    waker: Arc<Mutex<Option<Waker>>>,
    ctl: rtlsdr_mt::Controller,
}

impl Radio {
    pub fn open(cfg: RadioConfig) -> Radio {
        debug!("starting rtl-sdr with device-id {}", cfg.device_index);

        // setup iq sample buffer
        let iq_buffer = RingBuffer::<u8>::new(12 * RTL_SDR_BUFFER_SIZE);
        let (mut iq_producer, iq_consumer) = iq_buffer.split();

        // setup waker slot
        let shared_waker_slot = Arc::new(Mutex::new(Option::<Waker>::None));

        let (mut ctl, mut reader) = rtlsdr_mt::open(cfg.device_index.into()).unwrap();
        ctl.enable_agc().unwrap();
        ctl.set_ppm(cfg.ppm).unwrap();
        ctl.set_sample_rate(cfg.sample_rate).unwrap();
        ctl.set_center_freq(cfg.center_freq).unwrap();

        let rtl_shared_waker_slot = shared_waker_slot.clone();

        task::spawn_blocking(move || {
            reader
                .read_async(12, RTL_SDR_BUFFER_SIZE as u32, |bytes| {
                    trace!("got buffer from rtl-sdr iq");
                    iq_producer.push_slice(bytes);

                    let mut guard = rtl_shared_waker_slot.lock().unwrap();
                    if let Some(waker) = &*guard {
                        waker.wake_by_ref();
                    }
                    *guard = Option::<Waker>::None;
                })
                .unwrap();
            debug!("rtl-sdr reader thread finished");
            ()
        });

        Radio {
            consumer: iq_consumer,
            waker: shared_waker_slot,
            ctl: ctl,
        }
    }
}

impl AsyncRead for Radio {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        trace!("rtl-sdr AsyncRead poll_read");
        if self.consumer.is_empty() {
            *self.get_mut().waker.lock().unwrap() = Some(cx.waker().clone());
            return Poll::Pending;
        }

        let mut remaining = buf.initialize_unfilled();
        let n = self.get_mut().consumer.pop_slice(&mut remaining);
        buf.advance(n);
        trace!("rtl-sdr AsyncRead wrote {} into buf", n);
        Poll::Ready(Ok(()))
    }
}

impl Drop for Radio {
    fn drop(&mut self) {
        &self.ctl.cancel_async_read();
        trace!("rtl-sdr reader thread canceled");
    }
}

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

impl From<&[u8; 2]> for IQ {
    fn from(item: &[u8; 2]) -> Self {
        IQ {
            i: item[0],
            q: item[1],
        }
    }
}

