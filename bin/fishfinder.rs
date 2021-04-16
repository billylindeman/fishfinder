use failure::*;
use log::*;
use std::error::Error;
use structopt::StructOpt;

use fishfinder::sdr::{dsp, mode_s, rtl};
use std::pin::Pin;
use tokio::io::AsyncRead;
use tokio_stream::{Stream, StreamExt};
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(StructOpt)]
#[structopt(name = "fishfinder", about = "ads-b tracker for rtl-sdr")]
struct Cli {
    #[structopt(short, long)]
    path: Option<String>,
}

fn create_stream<T: 'static + AsyncRead + Sized>(
    iq_sample_src: T,
) -> Pin<Box<dyn Stream<Item = adsb::Message>>> {
    let magnitude_src = dsp::IQMagnitudeReader::new(iq_sample_src);

    let mode_s_frame_stream = FramedRead::with_capacity(
        magnitude_src,
        mode_s::FrameDecoder::new(),
        rtl::RTL_SDR_BUFFER_SIZE,
    );

    let adsb_stream = mode_s_frame_stream
        .filter_map(|f| f.ok())
        .filter_map(|frame| match frame.valid() {
            true => Some(frame),
            false => frame.try_repair(),
        })
        .filter_map(|f| f.parse());

    return Box::pin(adsb_stream);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::set_max_level(LevelFilter::Trace);
    let args = Cli::from_args();

    let mut stream = match args.path {
        Some(path) => create_stream(tokio::fs::File::open(path).await?),
        //_ => panic!("panik"),
        _ => create_stream(rtl::Radio::open(rtl::RadioConfig::mode_s(0))),
    };

    while let Some(frame) = stream.next().await {
        info!("got frame: {:?}", frame);
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

