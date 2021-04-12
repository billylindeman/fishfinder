use std::*;
use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::collections::VecDeque;
use log::*;
use ringbuf::*;


use crossbeam_utils::thread;
use crate::*;

const MODES_PREAMBLE_BITS: usize = 8;
const MODES_LONG_MSG_BITS: usize = 112;
const MODES_LONG_MSG_BYTES: usize = 14;
const MODES_MAGNITUDE_CAPACITY: usize = 1000000;
const MODES_FRAME_CAPACITY: usize = 1024;

pub struct ConvertIQToMagnitude{
    pub closed: AtomicBool, 
}

impl ConvertIQToMagnitude {
    pub fn new() -> ConvertIQToMagnitude {
        ConvertIQToMagnitude{
            closed: AtomicBool::from(false),
        }
    }
}

impl<'env> SignalTransform<'env,u8, u8> for ConvertIQToMagnitude {
    fn transform<'b>(&self, scope: &thread::Scope<'env>, mut src: Consumer<u8>) -> Consumer<u8> {
        let (mut sig_producer, sig_consumer) = RingBuffer::new(MODES_MAGNITUDE_CAPACITY).split();

        debug!("starting magnitude vector thread");
        scope.spawn(move |_| {
            loop {
                let mut iq: [u8; 2] = [0; 2]; 
                if src.is_empty() {
                    continue;
                }
                src.pop_slice(&mut iq);

                let i: f32 = (iq[0] as i16 - 127 as i16).into();
                let q: f32 = (iq[1] as i16 - 127 as i16).into();
                let mag: u8 = (i*i+q*q).sqrt().round() as u8;

                // trace!("got magnitude: {}", mag);
                sig_producer.push(mag).expect("unable to push magnitude");
            }
        });
        
        return sig_consumer;
    }
}


type ModeSFrame = [u8; MODES_LONG_MSG_BYTES];

pub struct ModeSFrameDetector{
    pub closed: AtomicBool, 
}

impl<'env> ModeSFrameDetector {
    pub fn new() -> ModeSFrameDetector {
        ModeSFrameDetector{
            closed: AtomicBool::from(false),
        }
    }

    fn detect_preamble(m: &VecDeque<u8>) -> bool {
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
                // trace!("Unexpected ratio among first 10 samples");
                return false;
            }


            /* The samples between the two spikes must be < than the average
            * of the high spikes level. We don't test bits too near to
            * the high levels as signals can be out of phase so part of the
            * energy can be in the near samples. */
            let high = (((m[0] as i32 + m[2] as i32 + m[7] as i32 + m[9] as i32))/6) as u8;
            if m[4] >= high || m[5] >= high {
                trace!("Too high level in samples between 3 and 6 {:?}", m);
                return false;
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
                return false;
            }

            return true;
    }
}

impl<'env> SignalTransform<'env, u8, ModeSFrame> for ModeSFrameDetector {
    fn transform(&self, scope: &thread::Scope<'env>, mut src: Consumer<u8>) -> Consumer<ModeSFrame> {
        let (mut frame_producer, frame_consumer) = RingBuffer::<ModeSFrame>::new(MODES_FRAME_CAPACITY).split();

        scope.spawn(move |_| {
            debug!("starting mode-s frame detector thread");
            let mut m: VecDeque<u8> = vec![0; MODES_PREAMBLE_BITS * 2].into();
            loop {
                if let Some(s) = src.pop() {
                    m.pop_front();
                    m.push_back(s);
                }else {
                    std::thread::sleep(time::Duration::from_millis(10));
                }

                if !ModeSFrameDetector::detect_preamble(&m) {
                    continue;
                }
                debug!("detected preamble!");

                let mut frame_samples: [u8; MODES_LONG_MSG_BITS * 2] = [0; MODES_LONG_MSG_BITS * 2];
                while src.len() < 256 {
                    trace!("packet buffer underrun, waiting");
                }
                src.read_exact(&mut frame_samples).unwrap();

                // decode bits from pulses
                let mut bits: [u8; 112] = [0; 112];
                for i in (0..frame_samples.len()).step_by(2) {
                    let low = frame_samples[i];
                    let high = frame_samples[i+1];
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
                let mut frame_bytes: ModeSFrame = [0; 14];
                for i in (0..bits.len()).step_by(8) {
                    frame_bytes[i/8] =
                        bits[i]<<7 | 
                        bits[i+1]<<6 | 
                        bits[i+2]<<5 | 
                        bits[i+3]<<4 | 
                        bits[i+4]<<3 | 
                        bits[i+5]<<2 | 
                        bits[i+6]<<1 | 
                        bits[i+7];
                }

                frame_producer.push(frame_bytes).expect("error pushing mode-s frame");
                // m = vec![0; MODES_PREAMBLE_BITS * 2].into();

           }
        });

        return frame_consumer;
    }
}

pub struct ModeSFrameDecoder{
    pub closed: AtomicBool, 
}

impl<'env> ModeSFrameDecoder {
    pub fn new() -> ModeSFrameDecoder {
        ModeSFrameDecoder{
            closed: AtomicBool::from(false),
        }
    }
}

impl<'env> SignalSink<'env, ModeSFrame> for ModeSFrameDecoder {
    fn consume(&self, mut src: Consumer<ModeSFrame>) {
        loop {
            if let Some(frame) = src.pop() {
                match adsb::parse_binary(&frame) {
                    Ok((message, _)) => {
                        info!("mode-s message {} => {:#?}", hex::encode(frame), message);
                        // if let adsb::MessageKind::ADSBMessage{crc: true, kind, ..} = message.kind {
                        //     info!("ads-b message {} => {:#?}", hex::encode(msg), kind);
                        // }
                    } ,
                    Err(error) => error!("error parsing ads-b frame {:#?}", error),
                }

            }else {
                std::thread::sleep_ms(10);
            }

        }

    }
}

