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
    let (iq_producer, mut iq_consumer) = iq_buffer.split();

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
    let signal_buffer = RingBuffer::<i8>::new(1000000);
    let (mut sig_producer, mut sig_consumer) = signal_buffer.split();
    let sig_thread = std::thread::spawn(move || {
        debug!("starting magnitude vector thread");
        loop {
            let mut iq: [u8; 2] = [0; 2]; 
            if iq_consumer.is_empty() {
                continue;
            }
            iq_consumer.pop_slice(&mut iq);

            let i: f32 = (iq[0] as i16 - 127 as i16).into();
            let q: f32 = (iq[1] as i16 - 127 as i16).into();
            let mag: i8 = (i*i+q*q).sqrt().round() as i8;

            trace!("got magnitude: {}", mag);
            sig_producer.push(mag).expect("unable to push magnitude");
        }
    });

    // demodulator
    let packet_buffer = RingBuffer::<Vec<u8>>::new(1000);
    let (mut pak_producer, pak_consumer) = packet_buffer.split();




    // decoder 

    sig_thread.join().unwrap();
    Ok(())
}