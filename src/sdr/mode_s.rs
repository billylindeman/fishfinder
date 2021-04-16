use bytes::{Buf, BytesMut};
use log::*;
use tokio_util::codec;

use super::crc;

pub const MODES_PREAMBLE_BITS: usize = 8;
pub const MODES_SHORT_MSG_BITS: usize = 56;
pub const MODES_LONG_MSG_BITS: usize = 112;
pub const MODES_SHORT_MSG_BYTES: usize = MODES_SHORT_MSG_BITS / 8;
pub const MODES_LONG_MSG_BYTES: usize = MODES_LONG_MSG_BITS / 8;

pub type FrameBits = [u8; MODES_LONG_MSG_BITS];
pub type FrameSamples = [u8; MODES_LONG_MSG_BITS * 2];
pub struct Frame(Vec<u8>);

pub struct FrameDecoder {}

impl FrameDecoder {
    pub fn new() -> FrameDecoder {
        FrameDecoder {}
    }

    fn detect_preamble(m: &[u8; MODES_PREAMBLE_BITS * 2]) -> bool {
        /* First check of relations between the first 10 samples
         * representing a valid preamble. We don't even investigate further
         * if this simple test is not passed. */
        if !(m[0] > m[1]
            && m[1] < m[2]
            && m[2] > m[3]
            && m[3] < m[0]
            && m[4] < m[0]
            && m[5] < m[0]
            && m[6] < m[0]
            && m[7] > m[8]
            && m[8] < m[9]
            && m[9] > m[6])
        {
            // trace!("Unexpected ratio among first 10 samples");
            return false;
        }

        /* The samples between the two spikes must be < than the average
         * of the high spikes level. We don't test bits too near to
         * the high levels as signals can be out of phase so part of the
         * energy can be in the near samples. */
        let high = ((m[0] as i32 + m[2] as i32 + m[7] as i32 + m[9] as i32) / 6) as u8;
        if m[4] >= high || m[5] >= high {
            trace!("Too high level in samples between 3 and 6 {:?}", m);
            return false;
        }

        /* Similarly samples in the range 11-14 must be low, as it is the
         * space between the preamble and real data. Again we don't test
         * bits too near to high levels, see above. */
        if m[11] >= high || m[12] >= high || m[13] >= high || m[14] >= high {
            trace!("Too high level in samples between 10 and 15 {:?}", m);
            return false;
        }

        return true;
    }

    fn demodulate_samples_to_bits(frame_samples: FrameSamples) -> FrameBits {
        // decode bits from pulses
        let mut bits: FrameBits = [0; 112];
        for i in (0..frame_samples.len()).step_by(2) {
            let low = frame_samples[i];
            let high = frame_samples[i + 1];
            let mut delta = low as i32 - high as i32;
            if delta < 0 {
                delta = -1 * delta;
            }
            if i > 0 && delta < 256 {
                bits[i / 2] = bits[i / 2 - 1];
            }
            if low > high {
                bits[i / 2] = 1;
            } else {
                bits[i / 2] = 0;
            }
        }

        return bits;
    }

    fn pack_bits(bits: FrameBits) -> Frame {
        /* Pack bits into bytes */
        let mut frame_bytes = [0; MODES_LONG_MSG_BYTES];
        for i in (0..bits.len()).step_by(8) {
            frame_bytes[i / 8] = bits[i] << 7
                | bits[i + 1] << 6
                | bits[i + 2] << 5
                | bits[i + 3] << 4
                | bits[i + 4] << 3
                | bits[i + 5] << 2
                | bits[i + 6] << 1
                | bits[i + 7];
        }

        let msgtype = frame_bytes[0] >> 3;
        let msglen = match msgtype {
            16 | 17 | 19 | 20 | 21 => MODES_LONG_MSG_BYTES,
            _ => MODES_SHORT_MSG_BYTES,
        };

        Frame(frame_bytes[0..msglen].into())
    }
}

impl codec::Decoder for FrameDecoder {
    type Item = Frame;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // consume buffer looking for a preamble
        loop {
            if src.len() < (MODES_PREAMBLE_BITS + MODES_LONG_MSG_BITS) * 2 {
                // Not enough data
                return Ok(None);
            }

            let mut preamble: [u8; (MODES_PREAMBLE_BITS * 2)] = Default::default();
            preamble.clone_from_slice(&src[0..(MODES_PREAMBLE_BITS * 2)]);

            if FrameDecoder::detect_preamble(&preamble) {
                // We slide the buffer window 1 sample at a time until this function detects a preamble
                break;
            }

            src.advance(1);
        }

        // We have a valid preamble, read full sized frame
        let mut frame_samples: [u8; MODES_LONG_MSG_BITS * 2] = [0; MODES_LONG_MSG_BITS * 2];
        let s = &src[(MODES_PREAMBLE_BITS * 2)..((MODES_PREAMBLE_BITS + MODES_LONG_MSG_BITS) * 2)];
        frame_samples.clone_from_slice(s);

        let frame_bits = FrameDecoder::demodulate_samples_to_bits(frame_samples);
        let frame = FrameDecoder::pack_bits(frame_bits);

        // advance the buffer by the preamble and length of the actual decoded frame
        src.advance((MODES_PREAMBLE_BITS * 2) + (MODES_LONG_MSG_BITS * 2));

        Ok(Some(frame))
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let valid = match self.valid() {
            true => "valid",
            false => "invalid",
        };
        write!(f, "({},{})", hex::encode(&self.0), valid)
    }
}

impl Frame {
    pub fn valid(&self) -> bool {
        let frame_bytes = &self.0;
        let crc: u32 = ((frame_bytes[frame_bytes.len() - 3] as u32) << 16)
            | ((frame_bytes[frame_bytes.len() - 2] as u32) << 8)
            | (frame_bytes[frame_bytes.len() - 1] as u32);
        let crc2: u32 = crc::modes_checksum(&frame_bytes);
        let valid = crc == crc2;
        trace!("crc: {:#x} crc2:{:#x} match: {}", crc, crc2, valid);
        valid
    }

    pub fn try_repair(&self) -> Option<Frame> {
        if let Some(repaired_frame) = crc::modes_repair_single_bit(&self.0) {
            info!(
                "repaired frame {} => {}",
                hex::encode(&self.0),
                hex::encode(&repaired_frame)
            );
            return Some(Frame(repaired_frame));
        }

        None
    }

    pub fn parse(&self) -> Option<adsb::Message> {
        match adsb::parse_binary(&self.0) {
            Ok((message, _)) => Some(message),
            Err(error) => {
                error!("error parsing ads-b frame {:#?}", error);
                None
            }
        }
    }
}

