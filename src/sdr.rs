use std::result::*;

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





        Ok(())
    }
    fn close(&self) -> Result<(), Error> {
        Err(format_err!("not implemented"))
    }
}

