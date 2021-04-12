use std::*;
use std::sync::atomic::AtomicBool;
use log::*;
use ringbuf::*;

use crossbeam::utils::*;
use crate::*;

const MODES_PREAMBLE: usize = 8;
const MODES_MAGNITUDE_CAPACITY: usize = 1000000;

pub struct ModeS {}


pub struct ConvertIQToMagnitude{
    pub closed: AtomicBool 
}

impl ConvertIQToMagnitude {
    pub fn new() -> ConvertIQToMagnitude {
        ConvertIQToMagnitude{closed: AtomicBool::from(false)}
    }
}

impl SignalTransform<u8, u8> for ConvertIQToMagnitude {
    fn transform(&self, src: &mut Consumer<u8>) -> Consumer<u8> {
        let (mut sig_producer, sig_consumer) = RingBuffer::new(MODES_MAGNITUDE_CAPACITY).split();

        debug!("starting magnitude vector thread");
        crossbeam::scope(|scope| {

        })
        thread::spawn(move || {
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