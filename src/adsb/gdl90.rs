use binwrite::{BinWrite, WriterOption};
use bitfield::{Bit, BitRange};
use bytes::BytesMut;
use std::io::{Result, Write};

use adsb::ICAOAddress;
// MessageType refers to the GDL90 Message identifier
// (byte 1 in a payload)
enum MessageType {
    Heartbeat = 0,
    Initialization = 2,
    UplinkDataOut = 7,
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

impl BinWrite for HeartbeatStatus {
    fn write_options<W: Write>(&self, writer: &mut W, options: &WriterOption) -> Result<()> {
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

#[derive(BinWrite)]
pub struct Heartbeat {
    status: HeartbeatStatus,
    timestamp: u16,
    msg_counts: u16,
}

#[derive(BinWrite)]
pub struct Ownship {}

#[derive(BinWrite)]
pub struct OwnshipGeometricAltitude {}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TrafficAlertStatus {
    NoAlert = 0,
    Alert = 1,
}

fn tas_to_u8(a: &TrafficAlertStatus) -> u8 {
    *a as u8
}

#[derive(Clone, Copy)]
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

#[derive(BinWrite)]
pub struct Traffic {
    #[binwrite(preprocessor(tas_to_u8))]
    traffic_alert_status: TrafficAlertStatus, //Traffic Alert Status
    #[binwrite(preprocessor(address_type_to_u8))]
    address_type: TrafficAddressType,
    participant_address: [u8; 3],
    latitude: u32,
    longitude: u32,
    altitude: u16,
}

#[derive(BinWrite)]
pub struct ForeflightIdentify {
    version: u8,
    serial_number: u64,
    device_name: [u8; 8],
    device_name_long: [u8; 16],
    capabilities: u8,
}

pub struct ForeflightAHRS {}

pub enum Message {
    Heartbeat(Heartbeat),
    OwnshipReport(Ownship),
    OwnshipGeometricAltitude(OwnshipGeometricAltitude),
    TrafficReport(Traffic),
    // Foreflight Extended Specification
    // https://www.foreflight.com/connect/spec
    ForeflightIdentify(ForeflightIdentify),
}

impl BinWrite for Message {
    fn write_options<W: Write>(&self, writer: &mut W, options: &WriterOption) -> Result<()> {
        writer.write(&[0x7E])?;

        let id: u8;
        let mut frame = Vec::<u8>::new();

        match self {
            Message::Heartbeat(msg) => {
                id = MessageType::Heartbeat as u8;
                msg.write_options(&mut frame, options)?;
            }
            Message::OwnshipReport(msg) => {
                id = MessageType::OwnshipReport as u8;
                msg.write_options(&mut frame, options)?;
            }
            Message::OwnshipGeometricAltitude(msg) => {
                id = MessageType::OwnshipGeometricAltitude as u8;
                msg.write_options(&mut frame, options)?;
            }
            Message::TrafficReport(msg) => {
                id = MessageType::TrafficReport as u8;
                msg.write_options(&mut frame, options)?;
            }
            Message::ForeflightIdentify(msg) => {
                id = MessageType::Foreflight as u8;
                msg.write_options(&mut frame, options)?;
            }
        }

        let mut digest = crc16::State::<crc16::CCITT_FALSE>::new();
        digest.update(&frame);
        let crc = digest.get().to_le_bytes();

        writer.write(&[id])?;
        writer.write(&frame)?;
        writer.write(&crc)?;

        writer.write(&[0x7E])?;
        Ok(())
    }
}

pub struct GDLEncoder {}
