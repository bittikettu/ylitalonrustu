use std::str;
use serde::{Serialize, Deserialize};
use bincode;

pub enum Packets {
	EMT_DATA_SIGNAL_MESSAGE,
	EMT_DATA_COLLECTION_TABLE_MESSAGE,
	EMT_DATA_SIGNAL_DEFINITION_MESSAGE,
	EMT_OWN_DATA_SIGNAL_MESSAGE
}

//const EMT_DATA_SIGNAL_MESSAGE:&u8 = 18;
//const EMT_DATA_COLLECTION_TABLE_MESSAGE:&u8 = 19;
//const EMT_DATA_SIGNAL_DEFINITION_MESSAGE:&u8 = 20;
pub const EMT_OWN_DATA_SIGNAL_MESSAGE:u16 = 21;


#[derive(Serialize, Deserialize, Debug)]
pub struct OwnDataSignalPacket {
	pub packet_length:u16, // Paketin kokonaispituus
	pub packet_id:u16, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
	pub sample_packet_length:u16, // pituus tavuina
    pub signal_view_type:u8,
	pub signal_sample_type:u8, // current value, average, minimum or maximum, see SST_
	pub signal_number:u16,
	pub signal_group:u16, // see DSG_
	pub milliseconds:u64, // aikaleima millisekunteina vuodesta 1601
	// datan pituus samplePacketLength - 16
	//data:[u8;MAX_N],
    pub data: Vec<u8>,
	//data:data,

	//signals:[u8;996], // sisaltaa DataSignalSampleja
}

impl OwnDataSignalPacket {
    pub fn packdata(&mut self, value:&str) {
        println!("type: {} value: {}",self.signal_sample_type, value);
        match self.signal_view_type {
            1 => {
                self.data = value.parse::<u32>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            2 => {
                self.data = value.parse::<i8>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            3 => {
                self.data = value.parse::<i16>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            4 => {
                self.data = value.parse::<i32>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            5 => {
                self.data = value.parse::<u8>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            6 => {
                self.data = value.parse::<u16>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            7 => {
                self.data = value.parse::<u32>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            8 => {
                self.data = value.parse::<f64>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            9 => {
                self.data = value.as_bytes().to_vec();
                self.data.push(0);
            },
            0 => {
                println!("Zero")
            },
            _ => {
                println!("Rest of the number")
            }
        }
        let bytes = bincode::serialize(&self).unwrap().len();
        self.packet_length = bytes as u16;
        self.sample_packet_length = self.packet_length - 1;
    }
}
