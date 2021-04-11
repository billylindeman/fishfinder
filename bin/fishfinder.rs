use std::sync::Arc;
use failure::*;
use structopt::StructOpt;
use log::*;

use fishfinder::*;

#[derive(StructOpt)]
#[structopt(name = "fishfinder", about = "ads-b tracker for rtl-sdr")]
struct Cli {
    #[structopt(short, long)]
    path: Option<String>,
}


fn start_sdr<T: sdr::SDR>(sdr: Arc<T>) -> Result<(), Error> {
    sdr.init()?;

    let handle = std::thread::spawn(move || {
        sdr.run().expect("error running sdr");
    });

    handle.join().expect("error waiting for sdr thread");
    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    log::set_max_level(LevelFilter::Trace);

    let args = Cli::from_args();


    match args.path {
        Some(path) => {
            let sdr = Arc::new(sdr::FileSDR{path: path});
            start_sdr(sdr)?;
        }
        _ =>  {
            let sdr = Arc::new(sdr::RtlSDR{device_id: 0});
            start_sdr(sdr)?;
        }
    }

    Ok(())
}