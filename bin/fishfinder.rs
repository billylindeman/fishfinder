use std::sync::Arc;
use failure::*;
use structopt::StructOpt;
use log::*;

use ringbuf::*;

use fishfinder::*;

#[derive(StructOpt)]
#[structopt(name = "fishfinder", about = "ads-b tracker for rtl-sdr")]
struct Cli {
    #[structopt(short, long)]
    path: Option<String>,
}


fn start_sdr<T: sdr::SDR>(mut sdr: T) -> Result<(), Error> {
    sdr.init()?;

    std::thread::spawn(move || {
        sdr.run().expect("error running sdr");
    });

    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    log::set_max_level(LevelFilter::Trace);

    let args = Cli::from_args();


    // setup iq sample buffer
    let iq_buffer = RingBuffer::<u8>::new(2000000);
    let (iq_producer, mut consumer) = iq_buffer.split();

    match args.path {
        Some(path) => {
            let sdr = sdr::FileSDR{path: path, producer: iq_producer};
            start_sdr(sdr)?;
        }
        _ =>  {
            let sdr = sdr::RtlSDR{device_id: 0, producer: iq_producer};
            start_sdr(sdr)?;
        }
    }

    // magnitude vector processor
    loop {
        match consumer.pop() {
            Some(v) => {
                trace!("got val {}", v);
            }
            None => {},
        }
    }

    // preamble detector 

    // decoder 

    Ok(())
}