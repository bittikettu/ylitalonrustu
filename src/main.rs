use std::{
    env,
    process,
    thread,
    str,
    time::{Duration,SystemTime},
    mem, intrinsics::size_of_val,
};
use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};
use to_vec::ToVec;
use std::path::Path;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::convert::TryFrom;
extern crate paho_mqtt as mqtt;

enum IpAddrKind {
        V4 = 0,
        V6 = 1,
    }

struct BufferElement {
    timestamp: SystemTime,
    id: String,
    data: Vec<u8>,
}

enum Packets {
	EMT_DATA_SIGNAL_MESSAGE,
	EMT_DATA_COLLECTION_TABLE_MESSAGE,
	EMT_DATA_SIGNAL_DEFINITION_MESSAGE,
	EMT_OWN_DATA_SIGNAL_MESSAGE
}

//const EMT_DATA_SIGNAL_MESSAGE:&u8 = 18;
//const EMT_DATA_COLLECTION_TABLE_MESSAGE:&u8 = 19;
//const EMT_DATA_SIGNAL_DEFINITION_MESSAGE:&u8 = 20;
//const EMT_OWN_DATA_SIGNAL_MESSAGE:&u8 = 21;


struct RediSignal {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

//struct DataSignalSample {
//	sample_packet_length:u16, // pituus tavuina
//	signal_sample_type:u16, // current value, average, minimum or maximum, see SST_
//	signal_number:u16,
//	signal_group:u16, // see DSG_
//	milliseconds:u64, // aikaleima millisekunteina vuodesta 1601
//	// datan pituus samplePacketLength - 16
//	data:[u8;900], // tata ei tarvitse olla kokonaan, jos samplePacketLength < 900
//}

	union data {
	    type_str: u32,
	    f2: f32,
	}

const MAX_N: usize = 900;

#[derive(Serialize, Deserialize, Debug)]
struct OwnDataSignalPacket {
	packet_length:u16, // Paketin kokonaispituus
	packet_id:u16, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
	sample_packet_length:u16, // pituus tavuina
	signal_sample_type:u16, // current value, average, minimum or maximum, see SST_
	signal_number:u16,
	signal_group:u16, // see DSG_
	milliseconds:u64, // aikaleima millisekunteina vuodesta 1601
	// datan pituus samplePacketLength - 16
	//data:[u8;MAX_N],
    data: Vec<u8>,
	//data:data,

	//signals:[u8;996], // sisaltaa DataSignalSampleja
}

impl OwnDataSignalPacket {
    fn packdata(&mut self, value:&str) {
        //println!("{:?}",value);
        match self.signal_sample_type {
            1=>{
                println!("type: {} value: {}",self.signal_sample_type, value.parse::<u64>().unwrap());
            },
            2=>{
                println!("type: {} value: {}",self.signal_sample_type, value);
            },
            3=>{
                println!("type: {} value: {}",self.signal_sample_type, value);
            },
            4=>{
                println!("type: {} value: {}",self.signal_sample_type, value);
            },
            5=>{
                println!("type: {} value: {}",self.signal_sample_type, value);
            },
            6=>{
                println!("type: {} value: {}",self.signal_sample_type, value);
            },
            7=>{
                println!("type: {} value: {}",self.signal_sample_type, value);
            },
            8=>{
                //println!("type: {} value: {}",self.signal_sample_type, value);
                println!("type: {} value: {}",self.signal_sample_type, value.parse::<f64>().unwrap());
                self.data = value.parse::<f64>().unwrap().to_be_bytes().to_vec();
                self.data.push(0);
            },
            9=>{
                println!("type: {} value: {}",self.signal_sample_type, value);
                
                //let bytes = value.as_bytes().to_vec().push(0);//value.to_be_bytes();
                //derp.as_bytes().to_vec();
                self.data = value.as_bytes().to_vec();
                self.data.push(0);
            },
            0=>{
                println!("Zero")
            },
            _=>{
                println!("Rest of the number")
            }
        }
    }
}

//fn f(s: &[u8]) {}

//pub fn main() {
//    let x = "a";
//    f(x.as_bytes())
//}

//fn build_sample(signal_number: u16, username: String) -> OwnDataSignalPacket {
//    OwnDataSignalPacket {
//		packet_id : 1,
//    }
//}

const DFLT_BROKER:&str = "tcp://10.3.1.132:1883";
const DFLT_CLIENT:&str = "rust_subscribe";
const DFLT_TOPICS:&[&str] = &["rust/mqtt", "incoming/machine/+"];
// The qos list that match topics above.
const DFLT_QOS:&[i32] = &[0, 1];



// Reconnect to the broker when connection is lost.
fn try_reconnect(cli: &mqtt::Client) -> bool
{
    println!("Connection lost. Waiting to retry connection");
    for _ in 0..12 {
        thread::sleep(Duration::from_millis(5000));
        if cli.reconnect().is_ok() {
            println!("Successfully reconnected");
            return true;
        }
    }
    println!("Unable to reconnect after several attempts.");
    false
}

// Subscribes to multiple topics.
fn subscribe_topics(cli: &mqtt::Client) {
    if let Err(e) = cli.subscribe_many(DFLT_TOPICS, DFLT_QOS) {
        println!("Error subscribes topics: {:?}", e);
        process::exit(1);
    }
}


fn main() {
    let host = env::args().nth(1).unwrap_or_else(||
        DFLT_BROKER.to_string()
    );

    let mut sample = OwnDataSignalPacket {
	 	packet_length:66, // Paketin kokonaispituus
		packet_id:21, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
		sample_packet_length:32, // pituus tavuina
		signal_sample_type:0, // current value, average, minimum or maximum, see SST_
		signal_number:1000,
		signal_group:100, // see DSG_
		milliseconds:123123, // aikaleima millisekunteina vuodesta 1601
		// datan pituus samplePacketLength - 16
		
        data:Vec::new(),
    };
    
    sample.packdata("1");
    //let value: u64 = 0x1FFFF;
    let derp = "Testirimpsutekstihommeli";
    //let bytes = derp.as_bytes().to_vec().push(0);//value.to_be_bytes();
    //let hep = derp.as_bytes();
    //sample.data.to_vec(bytes);
    sample.data = derp.as_bytes().to_vec();//bytes.to_vec();
    sample.data.push(0);

    let s = match str::from_utf8(&sample.data) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    //sample.data[..bytes.len()].copy_from_slice(&value.to_be_bytes()[..bytes.len()]);
    println!("serialized = {}", s);
    let serialized = serde_json::to_string(&sample).unwrap();

    // Prints serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);
    //let derp = mem::size_of::<OwnDataSignalPacket>();
    
    //sample.data.as_slice(value.to_be_bytes());// = value.to_be_bytes().map(f)
    //for i in 0..bytes.len() {
	//	sample.data[i] = (bytes[i]) as u8;
	//}
    
    //println!("{:?}" , bytes);

    // Define the set of options for the create.
    // Use an ID for a persistent session.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(DFLT_CLIENT.to_string())
        .finalize();

    // Create a client.
    let cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    // Initialize the consumer before connecting.
    let rx = cli.start_consuming();

    // Define the set of options for the connection.
    let lwt = mqtt::MessageBuilder::new()
        .topic("test")
        .payload("Consumer lost connection")
        .finalize();
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    // Connect and wait for it to complete or fail.
    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    // Subscribe topics.
    subscribe_topics(&cli);

    println!("Processing requests...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            //println!("{}", msg.topic());
            let path = Path::new(msg.topic());
            //println!("{}", path.display());
            let machine = path.file_name().unwrap().to_str().unwrap();
            //println!("{}", machine);
            //println!("{}", msg.payload_str());
            //let v: Value = serde_json::from_str(machine);
            //let mut object: Value = serde_json::from_str(machine).unwrap();
            //println!("{:?}", serde_json::from_str::<serde_json::Value>(&msg.payload_str()));
            let obj = serde_json::from_str::<serde_json::Value>(&msg.payload_str());

            match obj {
                Ok(v) => {
                    //println!("{v:?}")
                    //println!("{:?}",v["type"].as_u64().unwrap() as u16);
                    //println!("{:?}",v["id"].as_u64().unwrap() as u16);
                    //println!("{}",v["ts"].as_str().unwrap().parse::<u64>().unwrap());
                    //println!("{:?}",v["value"].as_u64().unwrap() as u8);
                    
                    let mut sample2 = OwnDataSignalPacket {
                        packet_length:0, // Paketin kokonaispituus
                        packet_id:21, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
                        sample_packet_length:0, // pituus tavuina
                        signal_sample_type:v["type"].as_u64().unwrap() as u16, // current value, average, minimum or maximum, see SST_
                        signal_number:v["id"].as_u64().unwrap() as u16,
                        signal_group:100, // see DSG_
                        milliseconds:v["ts"].as_str().unwrap().parse::<u64>().unwrap(), // aikaleima millisekunteina vuodesta 1601
                        // datan pituus samplePacketLength - 16
                        data:Vec::new(),
                   };
                   
                   sample2.packdata(v["value"].as_str().unwrap());
                   let serialized = serde_json::to_string(&sample2).unwrap();
                   println!("serialized = {}", serialized);
                },
                Err(e) => println!("error{e:?}"),
            }

        }
        else if !cli.is_connected() {
            if try_reconnect(&cli) {
                println!("Resubscribe topics...");
                subscribe_topics(&cli);
            } else {
                break;
            }
        }
    }

    // If still connected, then disconnect now.
    if cli.is_connected() {
        println!("Disconnecting");
        cli.unsubscribe_many(DFLT_TOPICS).unwrap();
        cli.disconnect(None).unwrap();
    }
    println!("Exiting");
}
