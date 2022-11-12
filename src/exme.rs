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
use serde::{Deserialize, Serialize};
//use std::path::Path;
use serde_json::Value;

use std::{str};

#[derive(Serialize, Deserialize, Debug)]
pub enum MyError {
    ConversionError,
    ConversionNotDefined,
    PreliminaryDataNotValid,
}

pub enum DataSignalGroups {
    //Info,
    //Common,
    //Spn,
    User = 100
}

pub enum SignalSampleTypes {
    Current,
    /*Average,
    Minimum,
    ChangeCount,
    EnableCount,
    DisableCount,
    ValidCount*/
}
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum ViewTypes {
    V_VOID,               /* void */
    V_BIT,                /* bit */
    V_SIGNED_CHAR,        /* signed char */
    V_SIGNED_SHORT,       /* signed short */
    V_SIGNED_LONG,        /* signed long */
    V_UNSIGNED_CHAR,      /* unsigned char */
    V_UNSIGNED_SHORT,     /* unsigned short */
    V_UNSIGNED_LONG,      /* unsigned long */
    V_FLOAT,              /* float */
    V_STRING,             /* string */
    V_HEX,                /* hex */
    V_BINARY,             /* binary */
    V_BE_SIGNED_SHORT,    /* big-endian signed short */
    V_BE_SIGNED_LONG,     /* big-endian signed long */
    V_BE_UNSIGNED_SHORT,  /* big-endian unsigned short */
    V_BE_UNSIGNED_LONG,   /* big-endian unsigned long */
    V_UNIX_TIME,          /* unix time */
    V_TWO_BIT,            /* J1939 two-bit discrete parameter */
    V_SIGNED_LONG_LONG,   /* __int64 */
    V_UNSIGNED_LONG_LONG, /* unsigned __int64 */
    V_DOUBLE,             /* double */
}

pub const EMT_OWN_DATA_SIGNAL_MESSAGE: u16 = 21;
pub const SNIP_SNIP_VECTOR_HEADER: u16 = 8;
pub const EXTRA_LEN_ADJUSTMENTS: u16 = 4;
pub const MAGIC_HEADER_OFFSET: usize = 20; // really this is the size of Owndatasignalpacket structure until better this is it.
pub const MAGIC_HEADER_SKIP_VECTOR: usize = MAGIC_HEADER_OFFSET + SNIP_SNIP_VECTOR_HEADER as usize;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OwnDataSignalPacket {
    packet_length: u16,        // Paketin kokonaispituus
    packet_id: u16,            // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
    sample_packet_length: u16, // pituus tavuina
    signal_sample_type: u8, // current value, average, minimum or maximum, see SST_
    pub signal_view_type: u8,
    signal_number: u16,
    signal_group: u16, // see DSG_
    milliseconds: i64, // aikaleima millisekunteina vuodesta 1601
    data: Vec<u8>,
}

impl Default for OwnDataSignalPacket {
    fn default() -> OwnDataSignalPacket {
        OwnDataSignalPacket {
            packet_length: 0,                       // Paketin kokonaispituus
            packet_id: EMT_OWN_DATA_SIGNAL_MESSAGE, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
            sample_packet_length: 0,                // pituus tavuina
            signal_sample_type: SignalSampleTypes::Current as u8, // current value, average, minimum or maximum, see SST_
            signal_view_type: 0,
            signal_number: 0,
            signal_group: DataSignalGroups::User as u16, // see DSG_
            milliseconds: 0,   // aikaleima millisekunteina vuodesta 1601
            // datan pituus samplePacketLength - 16
            data: Vec::new(),
        }
    }
}

impl OwnDataSignalPacket {
    pub fn exmebusify(&mut self) -> Result<Vec<u8>, MyError> {
        match bincode::serialize(&self) {
            Ok(bincoded) => {
                // Some unknown territory how to actually serialize the struct with vec flattening.
                // Somehow this just works by snipping some vector serialization header away and calculating correct
                // sizes for packages
                self.packet_length = bincoded.len() as u16 - SNIP_SNIP_VECTOR_HEADER;
                self.sample_packet_length = self.packet_length - EXTRA_LEN_ADJUSTMENTS;
                let derp = bincode::serialize(&self).unwrap();
                return Ok([&derp[0..MAGIC_HEADER_OFFSET], &derp[MAGIC_HEADER_SKIP_VECTOR..derp.len()]].concat());
            }
            Err(e) => {
                println!("error{e:?}");
                return Err(MyError::ConversionNotDefined);
            }
        }
    }
}


impl OwnDataSignalPacket {
    fn val_to_datsignalpacket(&mut self, msg: &Value) -> Result<(), MyError> {
        //match &msg as &str {
        //    "type" => println!("derp"),
        //    _ => println!("derps2"),
        //}

        match msg["type"].as_u64() {
            Some(value) => self.signal_view_type = value as u8,
            None => (), //return Err(MyError::PreliminaryDataNotValid),
        }
        match msg["dt"].as_u64() {
            Some(value) => self.signal_view_type = value as u8,
            None => (),
        }
        match msg["id"].as_u64() {
            Some(value) => self.signal_number = value as u16,
            None => (),
        }

        match msg["ts"].as_str() {
            Some(value) => match value.parse::<i64>() {
                Ok(value) => self.milliseconds = value,
                Err(e) => {
                    println!("error{e:?}");
                    return Err(MyError::PreliminaryDataNotValid);
                }
            },
            None => return Err(MyError::PreliminaryDataNotValid),
        }

        match msg["val"].as_str() {
            Some(x) => {
                let parsed = self.packdata(x);
                match parsed {
                    Ok(pars) => {
                        self.data = pars;
                        //self.exmebusify().unwrap();
                        /*match self.exmebusify() {
                            Ok(val) => return Ok(val),
                            Err(e) => {
                                return Err(e);
                            }
                        }*/
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
        Ok(())
    }
}

pub fn to_exmebus_better(msg: &str) -> Result<Vec<OwnDataSignalPacket>, MyError> {
    let obj = serde_json::from_str::<serde_json::Value>(&msg);
    //println!("{:?}",obj);
    match obj {
        Ok(parsed) => {
            let mut vec: Vec<OwnDataSignalPacket> = Vec::new();
            //let mut sample2: OwnDataSignalPacket = OwnDataSignalPacket::default();
            if parsed.is_array() {
                for name in parsed.as_array() {
                    for muu in name.iter() {
                        let mut sample2: OwnDataSignalPacket = OwnDataSignalPacket::default();
                        sample2.val_to_datsignalpacket(muu).unwrap();
                        vec.push(sample2);
                    }
                }
            }
            if parsed.is_object() {
                let mut conversion: Vec<Value> = Vec::new();
                conversion.push(parsed);
                for name in conversion {
                    let mut sample2: OwnDataSignalPacket = OwnDataSignalPacket::default();
                    sample2.val_to_datsignalpacket(&name).unwrap();
                    vec.push(sample2);
                }

                //let mut sample2: OwnDataSignalPacket = OwnDataSignalPacket::default();
                //sample2.val_to_datsignalpacket(parsed.to).unwrap();
                //vec.push(sample2);
            }
            //println!("{:#?}", vec);
            return Ok(vec);
        }
        Err(_e) => Err(MyError::PreliminaryDataNotValid),
    }
}

/*
impl OwnDataSignalPacket {
    pub fn to_exmebus(&mut self, msg: &Message) -> Result<Vec<u8>, MyError> {
        // If there is a need for parsing the "path" at some point
        //let path = Path::new(msg.topic());
        //let comps = path.components();
        //println!("{:?}",comps);
        /*match path.file_name() {
            Some(polku) => match polku.to_str() {
                Some(macstr) => {
                    let machine = macstr;
                    //println!("{machine:?}");
                }
                None => println!("failed to convert string"),
            },
            None => println!("failed to convert string"),
        }*/

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
                        match parsed {
                            Ok(pars) => {
                                self.data = pars;
                                match self.exmebusify() {
                                    Ok(val) => return Ok(val),
                                    Err(e) => { 
                                        return Err(e);
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
}*/

impl OwnDataSignalPacket {
    pub fn packdata(&mut self, value: &str) -> Result<Vec<u8>, MyError> {
        let mut retvec: Vec<u8> = Vec::new();

        match self.signal_view_type {
            1 => match value.parse::<u8>() {
                Ok(v) => retvec = v.to_ne_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            2 => match value.parse::<i8>() {
                Ok(v) => retvec = v.to_ne_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            3 => match value.parse::<i16>() {
                Ok(v) => retvec = v.to_ne_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            4 => match value.parse::<i32>() {
                Ok(v) => retvec = v.to_ne_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            5 => match value.parse::<u8>() {
                Ok(v) => retvec = v.to_ne_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            6 => match value.parse::<u16>() {
                Ok(v) => retvec = v.to_ne_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            7 => match value.parse::<u32>() {
                Ok(v) => retvec = v.to_ne_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            8 => match value.parse::<f32>() {
                Ok(v) => retvec = v.to_ne_bytes().to_vec(),
                Err(e) => println!("error {e:?}"),
            },
            9 => {
                retvec = value.as_bytes().to_vec();
                //retvec.push(0);
            }
            0 => {
                return Err(MyError::ConversionNotDefined);
            }
            _ => {
                return Err(MyError::ConversionNotDefined);
            }
        }        
        return Ok(retvec);
    }
}
