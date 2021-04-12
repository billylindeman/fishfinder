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
        
        let iq_sample_src = match args.path {
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


        let signal_src = decode::ConvertIQToMagnitude::new()
            .transform(scope, iq_sample_src);
        let frame_src = decode::ModeSFrameDetector::new()
            .transform(scope, signal_src);

        decode::ModeSFrameDecoder::new().consume(frame_src);
    }).unwrap();


    // decoder 
    Ok(())
}