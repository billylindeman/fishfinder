use std::sync::Arc;
use failure::*;
use structopt::StructOpt;

use fishfinder::*;

#[derive(StructOpt)]
#[structopt(name = "fishfinder", about = "ads-b tracker for rtl-sdr")]
struct Cli {
    #[structopt(help = "path")]
    path: String,
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
    let args = Cli::from_args();

    let sdr = Arc::new(sdr::FileSDR{path: args.path});
    start_sdr(sdr)?;


    Ok(())
}