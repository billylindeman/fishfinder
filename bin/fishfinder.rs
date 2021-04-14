use failure::*;
use log::*;
use structopt::StructOpt;

use fishfinder::*;
use fishfinder::{SignalSink, SignalSrc, SignalTransform};

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
                let sdr = sdr::FileSDR { path: path };
                sdr.produce(scope)
            }
            _ => {
                let sdr = sdr::RtlSDR { device_id: 0 };
                sdr.produce(scope)
            }
        };

        let signal_src = sdr::decode::ConvertIQToMagnitude::new().transform(scope, iq_sample_src);
        let frame_src = sdr::decode::ModeSFrameDetector::new().transform(scope, signal_src);

        sdr::decode::ModeSFrameDecoder::new().consume(frame_src);
    })
    .unwrap();

    // decoder
    Ok(())
}

