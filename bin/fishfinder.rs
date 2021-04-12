use failure::*;
use structopt::StructOpt;
use log::*;


use fishfinder::*;
use fishfinder::{SignalSrc, SignalTransform, SignalSink};

#[derive(StructOpt)]
#[structopt(name = "fishfinder", about = "ads-b tracker for rtl-sdr")]
struct Cli {
    #[structopt(short, long)]
    path: Option<String>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    log::set_max_level(LevelFilter::Trace);

    let args = Cli::from_args();

    crossbeam::thread::scope(move |scope| {
        // setup iq sample buffer
        
        let mut iq_consumer = match args.path {
            Some(path) => {
                let sdr = sdr::FileSDR{path: path};
                sdr.produce(scope)
            }
            _ =>  {
                // let sdr = sdr::RtlSDR{device_id: 0, producer: iq_producer};
                // start_sdr(sdr);
                panic!("not implemented")
            }
        };


        let magnitude = decode::ConvertIQToMagnitude::new();
        let signal_consumer = magnitude.transform(scope, iq_consumer);

        // let frame_detector = decode::ModeSFrameDetector::new();
        // let frame_consumer = frame_detector.transform(&mut signal_consumer);
    }).unwrap();


    // decoder 
    Ok(())
}