use std::sync::Arc;
use failure::*;
use structopt::StructOpt;
use log::*;

use std::*;
use std::collections::VecDeque;
use std::io::{Read};
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
    let signal_buffer = RingBuffer::<u8>::new(1000000);
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
            let mag: u8 = (i*i+q*q).sqrt().round() as u8;

            // trace!("got magnitude: {}", mag);
            sig_producer.push(mag).expect("unable to push magnitude");
        }
    });

    // demodulator
    let packet_buffer = RingBuffer::<Vec<u8>>::new(1000);
    let (mut pak_producer, pak_consumer) = packet_buffer.split();

    let demodulator = std::thread::spawn(move || {
        let mut m: VecDeque<u8> = vec![0; 16].into();

        loop {
            if let Some(s) = sig_consumer.pop() {
                m.pop_front();
                m.push_back(s);
            }

            /* First check of relations between the first 10 samples
            * representing a valid preamble. We don't even investigate further
            * if this simple test is not passed. */
            if !(m[0] > m[1] &&
                m[1] < m[2] &&
                m[2] > m[3] &&
                m[3] < m[0] &&
                m[4] < m[0] &&
                m[5] < m[0] &&
                m[6] < m[0] &&
                m[7] > m[8] &&
                m[8] < m[9] &&
                m[9] > m[6])
            {
                // if (Modes.debug & MODES_DEBUG_NOPREAMBLE &&
                //     m[j] > MODES_DEBUG_NOPREAMBLE_LEVEL)
                //     dumpRawMessage("Unexpected ratio among first 10 samples",
                //         msg, m, j);
                continue;
            }


            /* The samples between the two spikes must be < than the average
            * of the high spikes level. We don't test bits too near to
            * the high levels as signals can be out of phase so part of the
            * energy can be in the near samples. */
            let high = (((m[0] as i32 + m[2] as i32 + m[7] as i32 + m[9] as i32))/6) as u8;
            if m[4] >= high || m[5] >= high {
                trace!("Too high level in samples between 3 and 6 {:?}", m);
                // if (Modes.debug & MODES_DEBUG_NOPREAMBLE &&
                //     m[j] > MODES_DEBUG_NOPREAMBLE_LEVEL)
                //     dumpRawMessage(
                //         "Too high level in samples between 3 and 6",
                //         msg, m, j);
                // ;
            }

            /* Similarly samples in the range 11-14 must be low, as it is the
            * space between the preamble and real data. Again we don't test
            * bits too near to high levels, see above. */
            if m[11] >= high ||
                m[12] >= high ||
                m[13] >= high ||
                m[14] >= high
            {
                trace!("Too high level in samples between 10 and 15 {:?}", m);
                // if (Modes.debug & MODES_DEBUG_NOPREAMBLE &&
                //     m[j] > MODES_DEBUG_NOPREAMBLE_LEVEL)
                //     dumpRawMessage(
                //         "Too high level in samples between 10 and 15",
                //         msg, m, j);
                continue;
            }

            debug!("detected preamble!");

            let mut packet_samples: [u8; 224] = [0; 224];
            while sig_consumer.len() < 256 {
                trace!("packet buffer underrun, waiting");
                std::thread::sleep(time::Duration::from_millis(10));
            }

            sig_consumer.read_exact(&mut packet_samples).unwrap();

            // decode bits from pulses
            let mut bits: [u8; 112] = [0; 112];
            for i in (0..packet_samples.len()).step_by(2) {
                let low = packet_samples[i];
                let high = packet_samples[i+1];
                let mut delta = low as i32 - high as i32;
                if delta < 0 {
                    delta = -1 * delta;
                }

                if i > 0 && delta < 256 {
                    bits[i/2] = bits[i/2-1];
                }
                if low > high {
                    bits[i/2] = 1;
                } else {
                    bits[i/2] = 0;
                }

            }

            /* Pack bits into bytes */
            let mut msg: [u8; 14] = [0; 14];
            for i in (0..bits.len()).step_by(8) {
                msg[i/8] =
                    bits[i]<<7 | 
                    bits[i+1]<<6 | 
                    bits[i+2]<<5 | 
                    bits[i+3]<<4 | 
                    bits[i+4]<<3 | 
                    bits[i+5]<<2 | 
                    bits[i+6]<<1 | 
                    bits[i+7];
            }


            debug!("got packet: {:?}", hex::encode(msg));

            match adsb::parse_binary(&msg) {
                Ok((message, _)) => {
                    info!("mode-s message {} => {:#?}", hex::encode(msg), message);
                    // if let adsb::MessageKind::ADSBMessage{crc: true, kind, ..} = message.kind {
                    //     info!("ads-b message {} => {:#?}", hex::encode(msg), kind);
                    // }
                } ,
                Err(error) => error!("error parsing ads-b frame {:#?}", error),
            }
        }

        
    });




    // decoder 

        

    sig_thread.join().unwrap();
    Ok(())
}