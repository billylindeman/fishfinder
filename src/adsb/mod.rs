use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

pub use adsb::{ADSBMessageKind, ICAOAddress, Message, MessageKind};

pub struct Aircraft {
    icao: ICAOAddress,
    call_sign: Option<String>,
    msg_count: u64,
}

impl Aircraft {
    fn new(icao_address: ICAOAddress) -> Aircraft {
        Aircraft {
            icao: icao_address,
            call_sign: None,
            msg_count: 0,
        }
    }

    fn update(&mut self, adsb: &ADSBMessageKind) {
        use ADSBMessageKind::*;

        match adsb {
            AircraftIdentification { callsign, .. } => self.call_sign = Some(callsign.to_string()),
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
            println!("{}\t{:?}\t{}\t", val.icao, val.call_sign, val.msg_count);
        }
    }
}
