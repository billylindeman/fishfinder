use binwrite::{BinWrite, WriterOption};
use bitfield::{Bit, BitRange};
use bytes::{BufMut, BytesMut};
use log::*;
use std::io::{Result, Write};

use adsb::ICAOAddress;
use tokio_util::codec;

// MessageType refers to the GDL90 Message identifier
// (byte 1 in a payload)
enum MessageType {
    Heartbeat = 0,
    Initialization = 2,
    UplinkDataOut = 7, // UAT Uplink (TIS-B,FIS-B)
    HeightAboveTerrain = 9,
    OwnshipReport = 10,
    OwnshipGeometricAltitude = 11,
    TrafficReport = 20,
    BasicReport = 30,
    LongReport = 31,
    // Foreflight Extended Specification
    // https://www.foreflight.com/connect/spec
    Foreflight = 0x65,
}

#[derive(Clone, Copy, Debug)]
pub struct HeartbeatStatus {
    //Byte 1
    //Bit 7: GPS Pos Valid
    //Bit 6: Maint Req'd
    //Bit 5: IDENT
    //Bit 4: Addr Type
    //Bit 3: GPS Batt Low
    //Bit 2: RATCS
    //Bit 1: reserved
    //Bit 0: UAT Initialized
    uat_initialized: bool,
    ratcs: bool,
    gps_batt_low: bool,
    addr_type: bool,
    ident: bool,
    maintainance_required: bool,
    gps_pos_valid: bool,
    //Byte 2
    //Bit 7: Time Stamp (MS bit)
    //Bit 6: CSA Requested
    //Bit 5: CSA Not Available
    //Bit 4: reserved
    //Bit 3: reserved
    //Bit 2: reserved
    //Bit 1: reserved
    //Bit 0: UTC OK
    utc_ok: bool,
    csa_not_available: bool,
    csa_not_requested: bool,
    timestamp_msb: bool,
}

impl Default for HeartbeatStatus {
    fn default() -> HeartbeatStatus {
        HeartbeatStatus {
            uat_initialized: true,
            ratcs: false,
            gps_batt_low: false,
            addr_type: false,
            ident: false,
            maintainance_required: false,
            gps_pos_valid: false,
            utc_ok: false,
            csa_not_available: false,
            csa_not_requested: false,
            timestamp_msb: false,
        }
    }
}

impl BinWrite for HeartbeatStatus {
    fn write_options<W: Write>(&self, writer: &mut W, _options: &WriterOption) -> Result<()> {
        let mut b1 = 0u8;
        b1.set_bit(0, self.uat_initialized);
        b1.set_bit(2, self.ratcs);
        b1.set_bit(3, self.gps_batt_low);
        b1.set_bit(4, self.addr_type);
        b1.set_bit(5, self.ident);
        b1.set_bit(6, self.maintainance_required);
        b1.set_bit(7, self.gps_pos_valid);

        let mut b2 = 0u8;
        b2.set_bit(0, self.utc_ok);
        b2.set_bit(5, self.csa_not_available);
        b2.set_bit(6, self.csa_not_requested);
        b2.set_bit(7, self.timestamp_msb);
        writer.write(&[b1, b2])?;

        Ok(())
    }
}

#[derive(BinWrite, Clone, Copy, Debug)]
pub struct Heartbeat {
    status: HeartbeatStatus,
    timestamp: u16,
    msg_counts: u16,
}

impl Default for Heartbeat {
    fn default() -> Heartbeat {
        Heartbeat {
            status: HeartbeatStatus::default(),
            timestamp: 0,
            msg_counts: 0,
        }
    }
}

#[derive(BinWrite, Clone, Copy, Debug)]
pub struct OwnshipGeometricAltitude {}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum TrafficAlertStatus {
    NoAlert = 0,
    Alert = 1,
}

fn tas_to_u8(a: &TrafficAlertStatus) -> u8 {
    *a as u8
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum TrafficAddressType {
    ADSBWithIcao = 0,
    ADSBWithSelfAssigned = 1,
    TISBWithIcao = 2,
    TISBWithTrackFile = 3,
    SurfaceVehicle = 4,
    GroundStationBeacon = 5,
}
fn address_type_to_u8(a: &TrafficAddressType) -> u8 {
    *a as u8
}

#[derive(BinWrite, Clone, Copy, Debug)]
pub struct Traffic {
    #[binwrite(preprocessor(tas_to_u8))]
    traffic_alert_status: TrafficAlertStatus, //Traffic Alert Status
    #[binwrite(preprocessor(address_type_to_u8))]
    address_type: TrafficAddressType,
    participant_address: [u8; 3],
    latitude: [u8; 3],  //24bit signed fraction
    longitude: [u8; 3], //24bit signed fraction
    altitude: u16,
}

// Foreflight Extended Specification
// https://www.foreflight.com/connect/spec
#[derive(BinWrite, Clone, Debug)]
pub struct ForeflightIdentify {
    pub version: u8,
    #[binwrite(big)]
    pub serial_number: u64,
    #[binwrite(preprocessor(string_to_sized_vec(8)))]
    pub device_name: String,
    #[binwrite(preprocessor(string_to_sized_vec(16)))]
    pub device_name_long: String,
    #[binwrite(big)]
    pub capabilities: u32,
}

fn string_to_sized_vec(len: usize) -> impl Fn(&String) -> Vec<u8> {
    move |s| {
        let mut b = vec![0; len];
        for (dst, src) in b.iter_mut().zip(s.as_bytes().iter()) {
            *dst = *src
        }
        b
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ForeflightAHRS {}

#[derive(Clone, Debug)]
pub enum Message {
    Heartbeat(Heartbeat),
    OwnshipReport(Traffic),
    OwnshipGeometricAltitude(OwnshipGeometricAltitude),
    TrafficReport(Traffic),
    ForeflightIdentify(ForeflightIdentify),
}

pub struct Encoder {}
impl Encoder {
    pub fn new() -> Encoder {
        Encoder {}
    }
}

impl codec::Encoder<Message> for Encoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<()> {
        let id: u8;
        let mut frame = Vec::<u8>::new();

        match item {
            Message::Heartbeat(msg) => {
                id = MessageType::Heartbeat as u8;
                msg.write(&mut frame)?;
            }
            Message::OwnshipReport(msg) => {
                id = MessageType::OwnshipReport as u8;
                msg.write(&mut frame)?;
            }
            Message::OwnshipGeometricAltitude(msg) => {
                id = MessageType::OwnshipGeometricAltitude as u8;
                msg.write(&mut frame)?;
            }
            Message::TrafficReport(msg) => {
                id = MessageType::TrafficReport as u8;
                msg.write(&mut frame)?;
            }
            Message::ForeflightIdentify(msg) => {
                id = MessageType::Foreflight as u8;
                frame.extend_from_slice(&[0]); //foreflight sub msg id
                msg.write(&mut frame)?;
            }
        }

        let crc = fcs_crc16_compute(&frame);

        dst.put_slice(&[0x7E]);
        dst.put_slice(&[id]);
        dst.put_slice(&frame);
        dst.put_slice(&(crc.to_le_bytes()));
        dst.put_slice(&[0x7E]);

        let b: &[u8] = &dst;
        trace!("sent bytes: {:X?}", b);
        trace!("len: {}", frame.len());

        Ok(())
    }
}

const FCS_CRC16_TABLE: [u16; 256] = [
    0x0000, 0x1021, 0x2042, 0x3063, 0x4084, 0x50a5, 0x60c6, 0x70e7, 0x8108, 0x9129, 0xa14a, 0xb16b,
    0xc18c, 0xd1ad, 0xe1ce, 0xf1ef, 0x1231, 0x0210, 0x3273, 0x2252, 0x52b5, 0x4294, 0x72f7, 0x62d6,
    0x9339, 0x8318, 0xb37b, 0xa35a, 0xd3bd, 0xc39c, 0xf3ff, 0xe3de, 0x2462, 0x3443, 0x0420, 0x1401,
    0x64e6, 0x74c7, 0x44a4, 0x5485, 0xa56a, 0xb54b, 0x8528, 0x9509, 0xe5ee, 0xf5cf, 0xc5ac, 0xd58d,
    0x3653, 0x2672, 0x1611, 0x0630, 0x76d7, 0x66f6, 0x5695, 0x46b4, 0xb75b, 0xa77a, 0x9719, 0x8738,
    0xf7df, 0xe7fe, 0xd79d, 0xc7bc, 0x48c4, 0x58e5, 0x6886, 0x78a7, 0x0840, 0x1861, 0x2802, 0x3823,
    0xc9cc, 0xd9ed, 0xe98e, 0xf9af, 0x8948, 0x9969, 0xa90a, 0xb92b, 0x5af5, 0x4ad4, 0x7ab7, 0x6a96,
    0x1a71, 0x0a50, 0x3a33, 0x2a12, 0xdbfd, 0xcbdc, 0xfbbf, 0xeb9e, 0x9b79, 0x8b58, 0xbb3b, 0xab1a,
    0x6ca6, 0x7c87, 0x4ce4, 0x5cc5, 0x2c22, 0x3c03, 0x0c60, 0x1c41, 0xedae, 0xfd8f, 0xcdec, 0xddcd,
    0xad2a, 0xbd0b, 0x8d68, 0x9d49, 0x7e97, 0x6eb6, 0x5ed5, 0x4ef4, 0x3e13, 0x2e32, 0x1e51, 0x0e70,
    0xff9f, 0xefbe, 0xdfdd, 0xcffc, 0xbf1b, 0xaf3a, 0x9f59, 0x8f78, 0x9188, 0x81a9, 0xb1ca, 0xa1eb,
    0xd10c, 0xc12d, 0xf14e, 0xe16f, 0x1080, 0x00a1, 0x30c2, 0x20e3, 0x5004, 0x4025, 0x7046, 0x6067,
    0x83b9, 0x9398, 0xa3fb, 0xb3da, 0xc33d, 0xd31c, 0xe37f, 0xf35e, 0x02b1, 0x1290, 0x22f3, 0x32d2,
    0x4235, 0x5214, 0x6277, 0x7256, 0xb5ea, 0xa5cb, 0x95a8, 0x8589, 0xf56e, 0xe54f, 0xd52c, 0xc50d,
    0x34e2, 0x24c3, 0x14a0, 0x0481, 0x7466, 0x6447, 0x5424, 0x4405, 0xa7db, 0xb7fa, 0x8799, 0x97b8,
    0xe75f, 0xf77e, 0xc71d, 0xd73c, 0x26d3, 0x36f2, 0x0691, 0x16b0, 0x6657, 0x7676, 0x4615, 0x5634,
    0xd94c, 0xc96d, 0xf90e, 0xe92f, 0x99c8, 0x89e9, 0xb98a, 0xa9ab, 0x5844, 0x4865, 0x7806, 0x6827,
    0x18c0, 0x08e1, 0x3882, 0x28a3, 0xcb7d, 0xdb5c, 0xeb3f, 0xfb1e, 0x8bf9, 0x9bd8, 0xabbb, 0xbb9a,
    0x4a75, 0x5a54, 0x6a37, 0x7a16, 0x0af1, 0x1ad0, 0x2ab3, 0x3a92, 0xfd2e, 0xed0f, 0xdd6c, 0xcd4d,
    0xbdaa, 0xad8b, 0x9de8, 0x8dc9, 0x7c26, 0x6c07, 0x5c64, 0x4c45, 0x3ca2, 0x2c83, 0x1ce0, 0x0cc1,
    0xef1f, 0xff3e, 0xcf5d, 0xdf7c, 0xaf9b, 0xbfba, 0x8fd9, 0x9ff8, 0x6e17, 0x7e36, 0x4e55, 0x5e74,
    0x2e93, 0x3eb2, 0x0ed1, 0x1ef0,
];

pub fn fcs_crc16_compute(data: &[u8]) -> u16 {
    let mut ret = 0u16;
    for i in 0..data.len() {
        ret = (FCS_CRC16_TABLE[(ret >> 8) as usize] ^ (ret << 8)) ^ (data[i] as u16);
    }
    ret
}
