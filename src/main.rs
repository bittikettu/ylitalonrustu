use std::{
    env,
    process,
    thread,
    str,
    time::Duration,
};
use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};
use std::path::Path;
extern crate paho_mqtt as mqtt;
use bincode;
mod exme;

const DFLT_BROKER:&str = "tcp://localhost:1883";
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
            let path = Path::new(msg.topic());
            //let machine = path.file_name().unwrap().to_str().unwrap();
            match path.file_name(){
                Some(polku) => {
                    
                    match polku.to_str() {
                        Some(macstr) => {
                            let machine = macstr;
                            println!("{machine:?}");
                        },
                        None => println!("failed to convert string"),
                    }
                }
                None => println!("failed to convert string"),
            }

            let obj = serde_json::from_str::<serde_json::Value>(&msg.payload_str());
            match obj {
                Ok(v) => {
                    let mut sample2 = exme::OwnDataSignalPacket {
                        packet_length:0, // Paketin kokonaispituus
                        packet_id:exme::EMT_OWN_DATA_SIGNAL_MESSAGE, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
                        sample_packet_length:0, // pituus tavuina
                        signal_sample_type:3, // current value, average, minimum or maximum, see SST_
                        signal_view_type: v["type"].as_u64().unwrap() as u8,
                        signal_number:v["id"].as_u64().unwrap() as u16,
                        signal_group:100, // see DSG_
                        milliseconds:v["ts"].as_str().unwrap().parse::<u64>().unwrap(), // aikaleima millisekunteina vuodesta 1601
                        // datan pituus samplePacketLength - 16
                        data:Vec::new(),
                   };
                //    let parssi = v["value"].as_str();
                   match v["value"].as_str() {
                    Some(x) => {
                        let parsed = sample2.packdata(x);
                        match parsed {
                         Ok(pars) => {
                             sample2.data = pars;
                             //let serialized = serde_json::to_string(&sample2).unwrap();
                             //println!("serialized = {}", serialized);
                             match bincode::serialize(&sample2) {
                                Ok(bincoded) => {
                                    let bytes = bincoded;
                                    println!("{:?} {}", bytes,bytes.len());
                                },
                                Err(e) => println!("error{e:?}"),
                             }

                         },
                         Err(e) => println!("error"),
                        }
                    },
                    None => println!("failed to convert string"),
                   }
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
