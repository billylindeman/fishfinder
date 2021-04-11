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

