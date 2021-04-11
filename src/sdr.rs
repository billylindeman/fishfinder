use std::result::*;
use std::fs::File;
use std::sync::Arc;
use std::io::BufReader;
use std::*;

use log::*;
use failure::*;



const SDR_SAMPLES: usize = 256000;

pub trait SDR: Send + Sync + 'static {
    fn init(&self) -> Result<(), Error>;
    fn run(&mut self) -> Result<(), Error>;
    fn close(&self) -> Result<(), Error>;
}
 

pub struct FileSDR {
    pub path: String,
    pub producer: ringbuf::Producer<u8>
}

impl SDR for FileSDR {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), Error> {
        debug!("starting FileSDR with {}", self.path);

        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        loop {
            match self.producer.read_from(&mut reader, Some(500)) {
                Ok(count) => trace!("read {} samples from dump", count),
                Err(e) => {
                    error!("error reading from dump: {:?}", e); 
                    break;
                }
            }
            thread::sleep(time::Duration::from_millis(50))
        }

        Ok(())
    }
    fn close(&self) -> Result<(), Error> {
        Err(format_err!("not implemented"))
    }
}



pub struct RtlSDR {
    pub device_id: u8,
    pub producer: ringbuf::Producer<u8>
}

impl SDR for RtlSDR {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), Error> {
        debug!("starting rtl-sdr with device-id {}", self.device_id);
       
        let (mut ctl, mut reader) = rtlsdr_mt::open(0).unwrap();

        ctl.enable_agc().unwrap();
        ctl.set_ppm(0).unwrap();
        ctl.set_sample_rate(2000000).unwrap();
        ctl.set_center_freq(1_090_000_000).unwrap();

        reader.read_async(4, 32768, |bytes| {
            trace!("got buffer from rtl-sdr iq");
            self.producer.push_slice(bytes);
        }).unwrap();

        Ok(())
    }
    fn close(&self) -> Result<(), Error> {
        Err(format_err!("not implemented"))
    }
}
