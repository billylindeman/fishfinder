use failure::*;
use log::*;
use std::error::Error;
use structopt::StructOpt;

use fishfinder::sdr::{dsp, modes, rtl};
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(StructOpt)]
#[structopt(name = "fishfinder", about = "ads-b tracker for rtl-sdr")]
struct Cli {
    #[structopt(short, long)]
    path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::set_max_level(LevelFilter::Trace);
    let args = Cli::from_args();

    let my_async_read = rtl::Radio::open(rtl::RadioConfig::mode_s(0));
    let my_iq_read = dsp::IQMagnitudeReader::new(my_async_read);

    let mut my_stream_of_bytes = FramedRead::with_capacity(
        my_iq_read,
        modes::FrameDecoder::new(),
        rtl::RTL_SDR_BUFFER_SIZE,
    );

    let mut adsb_stream = my_stream_of_bytes
        .filter_map(|f| f.ok())
        .filter_map(|frame| match frame.valid() {
            true => Some(frame),
            false => frame.try_repair(),
        });

    while let Some(frame) = adsb_stream.next().await {
        info!("got frame: {}", frame);
    }

    trace!("stream ended");

    Ok(())
}

//
//fn main() -> Result<(), Box<dyn std::error::Error>> {
//    crossbeam::thread::scope(move |scope| {
//        // setup iq sample buffer
//        let iq_sample_src = match args.path {
//            Some(path) => {
//                let sdr = sdr::FileSDR { path: path };
//                sdr.produce(scope)
//            }
//            _ => {
//                let sdr = sdr::RtlSDR { device_id: 0 };
//                sdr.produce(scope)
//            }
//        };
//
//        let signal_src = sdr::decode::ConvertIQToMagnitude::transform(scope, iq_sample_src);
//        let frame_src = sdr::decode::ModeSFrameDetector::transform(scope, signal_src);
//
//        sdr::decode::ModeSFrameDecoder::consume(frame_src);
//    })
//    .unwrap();
//
//    // decoder
//    Ok(())
//}

