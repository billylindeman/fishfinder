use std::result::*;
use std::fs::File;
use std::io::Read;
use std::io::BufReader;
use std::*;

use log::*;
use failure::*;


pub trait SDR: Send + Sync + 'static {
    fn init(&self) -> Result<(), Error>;
    fn run(&self) -> Result<(), Error>;
    fn close(&self) -> Result<(), Error>;
}
 

pub struct FileSDR {
    pub path: String
}

impl SDR for FileSDR {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }
    fn run(&self) -> Result<(), Error> {
        debug!("starting FileSDR with {}", self.path);

        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        let mut buf: [u8; 256] = [0; 256];
        loop {
            match reader.read(&mut buf) {
                Ok(_) => {
                    debug!("read samples from dump {:?}", buf);
                    thread::sleep(time::Duration::from_millis(500));
                },
                Err(e) => {
                    error!("got error: {:?}", e);
                    break;
                }
            }
        }

        Ok(())
    }
    fn close(&self) -> Result<(), Error> {
        Err(format_err!("not implemented"))
    }
}



pub struct RtlSDR {
    pub device_id: u8,
}

impl SDR for RtlSDR {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }
    fn run(&self) -> Result<(), Error> {
        debug!("starting rtl-sdr with device-id {}", self.device_id);
       
        let (mut ctl, mut reader) = rtlsdr_mt::open(0).unwrap();

        ctl.enable_agc().unwrap();
        ctl.set_ppm(0).unwrap();
        ctl.set_center_freq(1_090_000_000).unwrap();

        reader.read_async(4, 32768, |bytes| {
            debug!("got buffer from rtl-sdr iq = [{}{}]", bytes[0], bytes[1])
        }).unwrap();

        Ok(())
    }
    fn close(&self) -> Result<(), Error> {
        Err(format_err!("not implemented"))
    }
}
