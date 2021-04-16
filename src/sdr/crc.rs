use super::mode_s;

/* Parity table for MODE S Messages.
 * The table contains 112 elements, every element corresponds to a bit set
 * in the message, starting from the first bit of actual data after the
 * preamble.
 *
 * For messages of 112 bit, the whole table is used.
 * For messages of 56 bits only the last 56 elements are used.
 *
 * The algorithm is as simple as xoring all the elements in this table
 * for which the corresponding bit on the message is set to 1.
 *
 * The latest 24 elements in this table are set to 0 as the checksum at the
 * end of the message should not affect the computation.
 *
 * Note: this function can be used with DF11 and DF17, other modes have
 * the CRC xored with the sender address as they are reply to interrogations,
 * but a casual listener can't split the address from the checksum.
*/

const MODES_CHECKSUM_TABLE: [u32; 112] = [
    0x3935ea, 0x1c9af5, 0xf1b77e, 0x78dbbf, 0xc397db, 0x9e31e9, 0xb0e2f0, 0x587178, 0x2c38bc,
    0x161c5e, 0x0b0e2f, 0xfa7d13, 0x82c48d, 0xbe9842, 0x5f4c21, 0xd05c14, 0x682e0a, 0x341705,
    0xe5f186, 0x72f8c3, 0xc68665, 0x9cb936, 0x4e5c9b, 0xd8d449, 0x939020, 0x49c810, 0x24e408,
    0x127204, 0x093902, 0x049c81, 0xfdb444, 0x7eda22, 0x3f6d11, 0xe04c8c, 0x702646, 0x381323,
    0xe3f395, 0x8e03ce, 0x4701e7, 0xdc7af7, 0x91c77f, 0xb719bb, 0xa476d9, 0xadc168, 0x56e0b4,
    0x2b705a, 0x15b82d, 0xf52612, 0x7a9309, 0xc2b380, 0x6159c0, 0x30ace0, 0x185670, 0x0c2b38,
    0x06159c, 0x030ace, 0x018567, 0xff38b7, 0x80665f, 0xbfc92b, 0xa01e91, 0xaff54c, 0x57faa6,
    0x2bfd53, 0xea04ad, 0x8af852, 0x457c29, 0xdd4410, 0x6ea208, 0x375104, 0x1ba882, 0x0dd441,
    0xf91024, 0x7c8812, 0x3e4409, 0xe0d800, 0x706c00, 0x383600, 0x1c1b00, 0x0e0d80, 0x0706c0,
    0x038360, 0x01c1b0, 0x00e0d8, 0x00706c, 0x003836, 0x001c1b, 0xfff409, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000,
];

pub fn modes_checksum(frame: &Vec<u8>) -> u32 {
    let mut crc: u32 = 0;
    let offset: usize = match frame.len() == mode_s::MODES_LONG_MSG_BYTES {
        true => 0,
        false => (112 - 56),
    };

    for j in 0..(frame.len() * 8) {
        let byte = j / 8;
        let bit = j % 8;
        let bitmask = 1 << (7 - bit);

        /* If bit is set, xor with corresponding table entry. */
        if frame[byte] & bitmask != 0 {
            crc ^= MODES_CHECKSUM_TABLE[j + offset];
        }
    }
    return crc; /* 24 bit checksum. */
}

// bit repair of frame
pub fn modes_repair_single_bit(frame: &Vec<u8>) -> Option<Vec<u8>> {
    for j in 0..(frame.len() * 8) {
        let byte = j / 8;
        let bitmask = 1 << (7 - (j % 8));

        let mut new_frame = frame.to_vec();
        new_frame[byte] ^= bitmask; /* Flip j-th bit. */

        // check crc24 checksum
        let crc: u32 = ((new_frame[new_frame.len() - 3] as u32) << 16)
            | ((new_frame[new_frame.len() - 2] as u32) << 8)
            | (new_frame[new_frame.len() - 1] as u32);
        let crc2: u32 = modes_checksum(&new_frame);

        if crc == crc2 {
            return Some(new_frame);
        }
    }

    None
}

