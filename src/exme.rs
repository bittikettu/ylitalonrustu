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
use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt as mqtt;
// use paho_mqtt::Message;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{env, process, time::Duration};
use std::{str, thread};
use paho_mqtt::Message;
//type V_VOID               0 /* void */
type V_BIT                 = u32; /* bit */
type V_SIGNED_CHAR        = i8; /* signed char */
type V_SIGNED_SHORT       = i16; /* signed short */
type V_SIGNED_LONG        = i32; /* signed long */
type V_UNSIGNED_CHAR      = u8; /* unsigned char */
type V_UNSIGNED_SHORT     = u16; /* unsigned short */
type V_UNSIGNED_LONG      = u32; /* unsigned long */
type V_FLOAT              = f32; /* float */
type V_STRING             = str; /* string */
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
    ConversionNotDefined
}


#[derive(Debug)]
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

impl Default for OwnDataSignalPacket {
    fn default () -> OwnDataSignalPacket {
        OwnDataSignalPacket {
            packet_length: 0,                             // Paketin kokonaispituus
            packet_id: EMT_OWN_DATA_SIGNAL_MESSAGE, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
            sample_packet_length: 0,                      // pituus tavuina
            signal_sample_type: 3, // current value, average, minimum or maximum, see SST_
            signal_view_type: 0,
            signal_number: 0,
            signal_group: 100, // see DSG_
            milliseconds: 0, // aikaleima millisekunteina vuodesta 1601
            // datan pituus samplePacketLength - 16
            data: Vec::new(),
        }
    }
}

impl OwnDataSignalPacket {
    pub fn to_exmebus(&mut self, msg:&Message) -> Result<(Vec<u8>), MyError> {
        let path = Path::new(msg.topic());
        //let machine = path.file_name().unwrap().to_str().unwrap();
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
                self.signal_view_type = v["type"].as_u64().unwrap() as u8;
                self.signal_number = v["id"].as_u64().unwrap() as u16;
                self.milliseconds = v["ts"].as_str().unwrap().parse::<u64>().unwrap();
                //    let parssi = v["value"].as_str();
                match v["value"].as_str() {
                    Some(x) => {
                        let parsed = self.packdata(x);
                        match parsed {
                            Ok(pars) => {
                                self.data = pars;
                                //let serialized = serde_json::to_string(&sample2).unwrap();
                                //println!("serialized = {}", serialized);
                                match bincode::serialize(&self) {
                                    Ok(bincoded) => {
                                        let bytes = bincoded;
                                        println!("{:?} {}", bytes, bytes.len());
                                        return Ok(bytes);
                                    }
                                    Err(e) => {
                                        println!("error{e:?}");
                                        return Err(MyError::ConversionNotDefined);
                                    },
                                }
                            }
                            Err(e) =>  {
                                println!("error");
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
    pub fn packdata(&mut self, value:&str) -> Result<(Vec<u8>), MyError> {
        let mut retvec: Vec<u8> = Vec::new();
        println!("type: {} value: {}",self.signal_sample_type, value);
        match self.signal_view_type {
            1 => {
                match value.parse::<u32>() {
                    Ok(v) => retvec = v.to_be_bytes().to_vec(),
                    Err(e) => println!("error {e:?}"),
                }
            },
            2 => {
                match value.parse::<i8>() {
                    Ok(v) => retvec = v.to_be_bytes().to_vec(),
                    Err(e) => println!("error {e:?}"),
                }
            },
            3 => {
                match value.parse::<i16>() {
                    Ok(v) => retvec = v.to_be_bytes().to_vec(),
                    Err(e) => println!("error {e:?}"),
                }
            },
            4 => {
                match value.parse::<i32>() {
                    Ok(v) => retvec = v.to_be_bytes().to_vec(),
                    Err(e) => println!("error {e:?}"),
                }
            },
            5 => {
                match value.parse::<u8>() {
                    Ok(v) => retvec = v.to_be_bytes().to_vec(),
                    Err(e) => println!("error {e:?}"),
                }
            },
            6 => {
                match value.parse::<u16>() {
                    Ok(v) => retvec = v.to_be_bytes().to_vec(),
                    Err(e) => println!("error {e:?}"),
                }
            },
            7 => {
                match value.parse::<u32>() {
                    Ok(v) => retvec = v.to_be_bytes().to_vec(),
                    Err(e) => println!("error {e:?}"),
                }
            },
            8 => {
                match value.parse::<f32>() {
                    Ok(v) => retvec = v.to_be_bytes().to_vec(),
                    Err(e) => println!("error {e:?}"),
                }
            },
            9 => {
                retvec = value.as_bytes().to_vec();
                
            },
            0 => {
                return Err(MyError::ConversionNotDefined);
            },
            _ => {
                return Err(MyError::ConversionNotDefined);
            }
        }
        retvec.push(0);
        return Ok(retvec);
    }
}