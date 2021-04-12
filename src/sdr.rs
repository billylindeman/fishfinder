use std::*;
use log::*;
use std::result::*;
use failure::*;
use crossbeam::thread;
use std::fs::File;
use std::io::BufReader;
use ringbuf::RingBuffer;

use crate::*;

const IQ_SAMPLE_CAPACITY: usize = 2000000;

pub struct FileSDR {
    pub path: String,
}

impl<'env> SignalSrc<'env, u8> for FileSDR {
    fn produce(&self, scope: &thread::Scope<'env>) -> ringbuf::Consumer<u8> {
        debug!("starting FileSDR with {}", self.path);

        // setup iq sample buffer
        let iq_buffer = RingBuffer::<u8>::new(IQ_SAMPLE_CAPACITY);
        let (mut iq_producer, iq_consumer) = iq_buffer.split();

        let file = File::open(&self.path).unwrap();
        let mut reader = BufReader::new(file);

        scope.spawn(move |_|{
            loop {
                match iq_producer.read_from(&mut reader, Some(500)) {
                    Ok(count) => trace!("read {} samples from dump", count),
                    Err(e) => {
                        error!("error reading from dump: {:?}", e); 
                        break;
                    }
                }
                std::thread::sleep(time::Duration::from_millis(10))
            }
            
        });

        iq_consumer
    }
}

pub struct RtlSDR {
    pub device_id: u8,
}

impl<'env> SignalSrc<'env, u8> for RtlSDR {
    fn produce(&self, scope: &thread::Scope<'env>) -> ringbuf::Consumer<u8> {
        debug!("starting rtl-sdr with device-id {}", self.device_id);
       
        // setup iq sample buffer
        let iq_buffer = RingBuffer::<u8>::new(IQ_SAMPLE_CAPACITY);
        let (mut iq_producer, iq_consumer) = iq_buffer.split();

        let id = self.device_id as u32;
        scope.spawn(move |_| {
            let (mut ctl, mut reader) = rtlsdr_mt::open(id).unwrap();

            ctl.enable_agc().unwrap();
            ctl.set_ppm(0).unwrap();
            ctl.set_sample_rate(2000000).unwrap();
            ctl.set_center_freq(1_090_000_000).unwrap();

            reader.read_async(12, 32768, |bytes| {
                // trace!("got buffer from rtl-sdr iq");
                iq_producer.push_slice(bytes);
            }).unwrap();


        });
        
        iq_consumer
    }
}
  