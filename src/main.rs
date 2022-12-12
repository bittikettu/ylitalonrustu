// paho-mqtt/examples/async_subscribe.rs
// This is a Paho MQTT Rust client, sample application.
//
//! This application is an MQTT subscriber using the asynchronous client
//! interface of the Paho Rust client library.
//! It also monitors for disconnects and performs manual re-connections.
//!
//! The sample demonstrates:
//!   - An async/await subscriber
//!   - Connecting to an MQTT server/broker.
//!   - Subscribing to multiple topics
//!   - Using MQTT v5 subscribe options
//!   - Receiving messages from an async stream.
//!   - Handling disconnects and attempting manual reconnects.
//!   - Using a "persistent" (non-clean) session so the broker keeps
//!     subscriptions and messages through reconnects.
//!   - Last will and testament
//!

/*******************************************************************************
 * Copyright (c) 2017-2022 Frank Pagliughi <fpagliughi@mindspring.com>
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
 *    Frank Pagliughi - initial implementation and documentation
 *******************************************************************************/

//use futures::future::OrElse;
use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt as mqtt;
use std::{env, process, time::Duration};
mod exme;
use crate::exme::to_exmebus_better;
use clap::{Parser, ValueEnum};
use std::io::prelude::*;
use std::net::TcpStream;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Topic of the MQTT-channel to listen
    #[arg(long)]
    topic: String,

    /// Exmebus-port where to forward data
    #[arg(long)]
    exmebus_port: u64,

    /// Local mqtt port, 1883 or something
    #[arg(long)]
    mqtt_port: u64,

    /// MQTT-server address ie. tcp://localhost
    #[arg(long)]
    host: String,

    /// Mode of the parser json/redi
    #[arg(long, value_enum)]
    mode: Mode,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Convert incoming data from JSON to redi
    json,
    /// Convert incoming data from redi signals to redi (not implemented)
    redi,
}

fn main() {
    // Initialize the logger from the environment
    // env_logger::init();
    //let id = Uuid::new_v4();
    //let mut counter: u128 = 0;
    let args = Args::parse();

    println!("{:?}", args);
    // println!("Hello {}!", args);
    let version = option_env!("PROJECT_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    println!("Version {}", version);

    let host = "tcp://localhost:1893".to_string(); //env::args()
                                                   //.nth(1)
                                                   //.unwrap_or_else(|| "tcp://localhost:1893".to_string());

    let port = env::args().nth(1).unwrap_or_else(|| "2048".to_string());
    let _topic = env::args().nth(2).unwrap_or_else(|| "trash".to_string());
    let mode = env::args().nth(3).unwrap_or_else(|| "json".to_string());

    let mut topic = String::new();
    if mode == "json" {
        topic = format!("incoming/machine/{_topic}/json");
    } else {
        topic = format!("incoming/machine/{_topic}");
    }

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .mqtt_version(mqtt::MQTT_VERSION_5)
        .server_uri(host)
        .client_id(format!("mac_id_{_topic}_{port}"))
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        // Get message stream before connecting.
        let mut strm = cli.get_stream(50);

        // Define the set of options for the connection
        let lwt = mqtt::Message::new(
            "status",
            format!("{_topic} on port {port} lost connection"),
            mqtt::QOS_2,
        );

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .mqtt_version(mqtt::MQTT_VERSION_5)
            .clean_start(false)
            .properties(mqtt::properties![mqtt::PropertyCode::SessionExpiryInterval => 3600])
            .will_message(lwt)
            .finalize();

        // Make the connection to the broker
        println!("Connecting to the MQTT server...");
        println!("Machine={topic}");
        println!("Exmebusport={port}");
        cli.connect(conn_opts).await?;

        //println!("Subscribing to topics: {:?}", TOPICS);
        //let sub_opts = vec![mqtt::SubscribeOptions::with_retain_as_published(); TOPICS.len()];
        //cli.subscribe_many_with_options(TOPICS, QOS, &sub_opts, None)
        //    .await?;

        //let sub_opts = vec![mqtt::SubscribeOptions::with_retain_as_published()];
        cli.subscribe_with_options(
            topic,
            2,
            mqtt::SubscribeOptions::with_retain_as_published(),
            None,
        )
        .await?;
        // Just loop on incoming messages.
        println!("Waiting for messages...");

        // Note that we're not providing a way to cleanly shut down and
        // disconnect. Therefore, when you kill this app (with a ^C or
        // whatever) the server will get an unexpected drop and then
        // should emit the LWT message.

        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))?;
        match stream.set_write_timeout(Some(Duration::new(1, 0))) {
            Ok(_) => println!("Timeout set"),
            Err(e) => println!("Could not set timeout: {:?}", e),
        }

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                if msg.retained() {
                    print!("(R) ");
                }
                match to_exmebus_better(&msg.payload_str()) {
                    Ok(retvec) => {
                        for mut emsg in retvec {
                            match emsg.exmebusify() {
                                Ok(bt) => {
                                    let mors = stream.write(&bt);
                                    match mors {
                                        Ok(_) => (), // Do not do anything when everything just works fine!
                                        Err(e) => {
                                            println!("Should be stored to redis {:?}", e);
                                            stream =
                                                TcpStream::connect(format!("127.0.0.1:{port}"))?;
                                            match stream
                                                .set_write_timeout(Some(Duration::new(1, 0)))
                                            {
                                                Ok(_) => println!("Timeout set"),
                                                Err(e) => {
                                                    println!("Could not set timeout: {:?}", e)
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => println!("{:?}", e),
                            }
                        }
                    }
                    Err(e) => println!("{:?}", e),
                }
            } else {
                // A "None" means we were disconnected. Try to reconnect...
                println!("Lost connection. Attempting reconnect.");
                while let Err(err) = cli.reconnect().await {
                    println!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        // Explicit return type for the async block
        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}
