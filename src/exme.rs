/*******************************************************************************
 * Copyright (c) 2022 Juha Viitanen <juha.viitanen@exertus.fi>
 *
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v1.0
 * and Eclipse Distribution License v1.0 which accompany this distribution.
 *
 * The Eclipse Public License is available at
 *    http://www.eclipse.org/legal/epl-v10.html
 * and the Eclipse Distribution License is available at
 *   http://www.eclipse.org/org/documents/edl-v10.php.
 *
 * Contributors:
 *    Juha Viitanen - initial implementation and documentation
 *******************************************************************************/

use bincode;
use paho_mqtt::Message;
use serde::{Deserialize, Serialize};
use std::path::Path;
use serde::ser::{SerializeStruct, Serializer};

use std::{str};
//type V_VOID               0 /* void */
//type V_BIT = u32; /* bit */
//type V_SIGNED_CHAR = i8; /* signed char */
//type V_SIGNED_SHORT = i16; /* signed short */
//type V_SIGNED_LONG = i32; /* signed long */
//type V_UNSIGNED_CHAR = u8; /* unsigned char */
//type V_UNSIGNED_SHORT = u16; /* unsigned short */
//type V_UNSIGNED_LONG = u32; /* unsigned long */
//type V_FLOAT = f32; /* float */
//type V_STRING = str; /* string */
//type V_HEX                10 /* hex */
//type V_BINARY             11 /* binary */
//type V_BE_SIGNED_SHORT    12 /* big-endian signed short */
//type V_BE_SIGNED_LONG     13 /* big-endian signed long */
//type V_BE_UNSIGNED_SHORT  14 /* big-endian unsigned short */
//type V_BE_UNSIGNED_LONG   15 /* big-endian unsigned long */
//type V_UNIX_TIME          16 /* unix time */
//type V_TWO_BIT            17 /* J1939 two-bit discrete parameter */
//type V_SIGNED_LONG_LONG   18 /* __int64 */
//type V_UNSIGNED_LONG_LONG 19 /* unsigned __int64 */
//type V_DOUBLE             20 /* double */
#[derive(Serialize, Deserialize, Debug)]
pub enum MyError {
    ConversionError,
    ConversionNotDefined,
    PreliminaryDataNotValid,
}

//#[derive(Debug)]
//pub enum Packets {
//    EMT_DATA_SIGNAL_MESSAGE,
//    EMT_DATA_COLLECTION_TABLE_MESSAGE,
//    EMT_DATA_SIGNAL_DEFINITION_MESSAGE,
//    EMT_OWN_DATA_SIGNAL_MESSAGE,
//}

//const EMT_DATA_SIGNAL_MESSAGE:&u8 = 18;
//const EMT_DATA_COLLECTION_TABLE_MESSAGE:&u8 = 19;
//const EMT_DATA_SIGNAL_DEFINITION_MESSAGE:&u8 = 20;
pub const EMT_OWN_DATA_SIGNAL_MESSAGE: u16 = 21;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OwnDataSignalPacket {
    packet_length: u16,        // Paketin kokonaispituus
    packet_id: u16,            // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
    sample_packet_length: u16, // pituus tavuina
    signal_sample_type: u8, // current value, average, minimum or maximum, see SST_
    signal_view_type: u8,
    signal_number: u16,
    signal_group: u16, // see DSG_
    milliseconds: i64, // aikaleima millisekunteina vuodesta 1601
    // datan pituus samplePacketLength - 16
    //#[serde(flatten)]
    data: Vec<u8>,
}

impl Default for OwnDataSignalPacket {
    fn default() -> OwnDataSignalPacket {
        OwnDataSignalPacket {
            packet_length: 0,                       // Paketin kokonaispituus
            packet_id: EMT_OWN_DATA_SIGNAL_MESSAGE, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
            sample_packet_length: 0,                // pituus tavuina
            signal_sample_type: 3, // current value, average, minimum or maximum, see SST_
            signal_view_type: 0,
            signal_number: 0,
            signal_group: 100, // see DSG_
            milliseconds: 0,   // aikaleima millisekunteina vuodesta 1601
            // datan pituus samplePacketLength - 16
            data: Vec::new(),
        }
    }
}

// This is what #[derive(Serialize)] would generate.
//impl Serialize for OwnDataSignalPacket {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer,
//    {
//        let mut s = serializer.serialize_struct("OwnDataSignalPacket", 9)?;
//        s.serialize_field("packet_length", &self.packet_length)?;
//        s.serialize_field("packet_id", &self.packet_id)?;
//        s.serialize_field("sample_packet_length", &self.sample_packet_length)?;
//        s.serialize_field("signal_sample_type", &self.sample_packet_length)?;
//        s.serialize_field("signal_view_type", &self.sample_packet_length)?;
//        s.serialize_field("signal_number", &self.sample_packet_length)?;
//        s.serialize_field("signal_group", &self.sample_packet_length)?;
//        s.serialize_field("milliseconds", &self.sample_packet_length)?;
//        let flattened = &self.data;
//        s.serialize_field("data", &self.data)?;
//        s.end()
//    }
//}

impl OwnDataSignalPacket {
    pub fn to_exmebus(&mut self, msg: &Message) -> Result<Vec<u8>, MyError> {
        let path = Path::new(msg.topic());
        match path.file_name() {
            Some(polku) => match polku.to_str() {
                Some(macstr) => {
                    let machine = macstr;
                    println!("{machine:?}");
                }
                None => println!("failed to convert string"),
            },
            None => println!("failed to convert string"),
        }

        let obj = serde_json::from_str::<serde_json::Value>(&msg.payload_str());
        match obj {
            Ok(v) => {
                match v["type"].as_u64() {
                    Some(value) => self.signal_view_type = value as u8,
                    None => return Err(MyError::PreliminaryDataNotValid),
                }

                match v["id"].as_u64() {
                    Some(value) => self.signal_number = value as u16,
                    None => return Err(MyError::PreliminaryDataNotValid),
                }

                match v["ts"].as_str() {
                    Some(value) => match value.parse::<i64>() {
                        Ok(value) => self.milliseconds = value,
                        Err(e) => { 
                            println!("error{e:?}");
                            return Err(MyError::PreliminaryDataNotValid);
                        }
                    },
                    None => return Err(MyError::PreliminaryDataNotValid),
                }

                match v["value"].as_str() {
                    Some(x) => {
                        let parsed = self.packdata(x);
                        let flattened = self.packdata(x).into_iter().flatten().collect::<Vec<u8>>();
                        println!("littana {:?}", flattened);
                        match parsed {
                            Ok(pars) => {
                                self.data = pars;

                                match bincode::serialize(&self) {
                                    Ok(bincoded) => {
                                        // Some unknown territory how to actually serialize the struct with vec flattening.
                                        self.packet_length = bincoded.len() as u16 - 8;
                                        self.sample_packet_length = self.packet_length - 4;
                                        let derp = bincode::serialize(&self).unwrap();
                                        return Ok([&derp[0..20], &derp[28..derp.len()]].concat());
                                    }
                                    Err(e) => {
                                        println!("error{e:?}");
                                        return Err(MyError::ConversionNotDefined);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("error{e:?}");
                                return Err(MyError::ConversionNotDefined);
                            }
                        }
                    }
                    None => {
                        println!("failed to convert string");
                        return Err(MyError::ConversionNotDefined);
                    }
                }
            }
            Err(e) => {
                println!("error{e:?}");
                return Err(MyError::ConversionNotDefined);
            }
        }
    }
}

impl OwnDataSignalPacket {
    pub fn packdata(&mut self, value: &str) -> Result<Vec<u8>, MyError> {
        let mut retvec: Vec<u8> = Vec::new();
        println!("type: {} value: {}", self.signal_sample_type, value);
        match self.signal_view_type {
            1 => match value.parse::<u32>() {
                Ok(v) => retvec = v.to_be_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            2 => match value.parse::<i8>() {
                Ok(v) => retvec = v.to_be_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            3 => match value.parse::<i16>() {
                Ok(v) => retvec = v.to_be_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            4 => match value.parse::<i32>() {
                Ok(v) => retvec = v.to_be_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            5 => match value.parse::<u8>() {
                Ok(v) => retvec = v.to_be_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            6 => match value.parse::<u16>() {
                Ok(v) => retvec = v.to_be_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            7 => match value.parse::<u32>() {
                Ok(v) => retvec = v.to_be_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            8 => match value.parse::<f32>() {
                Ok(v) => retvec = v.to_be_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            9 => {
                retvec = value.as_bytes().to_vec();
            }
            0 => {
                return Err(MyError::ConversionNotDefined);
            }
            _ => {
                return Err(MyError::ConversionNotDefined);
            }
        }
        retvec.push(0);
        return Ok(retvec);
    }
}
