use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

pub use adsb::{ADSBMessageKind, ICAOAddress, Message, MessageKind};

pub struct Aircraft {
    icao: ICAOAddress,
    reg: Option<String>,
    callsign: Option<String>,
    emitter_category: Option<u8>,
    on_ground: Option<bool>,
    squawk: Option<u16>,

    latitude: Option<f64>,
    longitude: Option<f64>,
    altitude: Option<i32>,
    alt_gnss_baro_diff: Option<i32>,
    alt_is_gnss: Option<bool>,

    msg_count: u64,
}

impl Aircraft {
    fn new(icao_address: ICAOAddress) -> Aircraft {
        Aircraft {
            icao: icao_address,
            reg: None,
            callsign: None,
            emitter_category: None,
            on_ground: None,
            squawk: None,
            latitude: None,
            longitude: None,
            altitude: None,
            alt_gnss_baro_diff: None,
            alt_is_gnss: None,
            msg_count: 0,
        }
    }

    fn update(&mut self, adsb: &ADSBMessageKind) {
        use ADSBMessageKind::*;

        match adsb {
            AircraftIdentification { callsign, .. } => self.callsign = Some(callsign.to_string()),
            _ => {}
        }

        self.msg_count += 1
    }
}

pub struct Tracker {
    db: HashMap<ICAOAddress, Aircraft>,
}

impl Tracker {
    pub fn new() -> Tracker {
        Tracker { db: HashMap::new() }
    }

    pub fn process(&mut self, frame: &adsb::Message) {
        use adsb::MessageKind::*;

        match &frame.kind {
            ADSBMessage {
                icao_address, kind, ..
            } => {
                let ac = match self.db.entry(*icao_address) {
                    Vacant(entry) => entry.insert(Aircraft::new(*icao_address)),
                    Occupied(entry) => entry.into_mut(),
                };

                ac.update(&kind);
            }
            _ => {}
        }
    }

    pub fn print(&self) {
        print!("\x1B[2J\x1B[1;1H");

        println!("ICAO\tCALLSIGN\tMSGS");
        for (_, val) in self.db.iter() {
            println!("{}\t{:?}\t{}\t", val.icao, val.callsign, val.msg_count);
        }
    }
}
