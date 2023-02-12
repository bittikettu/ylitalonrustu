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
mod appargs;
mod exme;
use crate::appargs::Args;
use crate::appargs::Mode;
use crate::exme::to_exmebus_better;
use clap::{Parser, ValueEnum};
use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt as mqtt;
use std::io::prelude::*;
use std::net::TcpStream;
use std::{env, process, time::Duration};

fn main() {
    let args = Args::parse();
    if args.debug >= 1 {
        println!("{:?}", args);
    }
    let version = option_env!("PROJECT_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));

    let mut topic = String::new();
    match args.mode {
        Mode::JSON => topic = format!("{}{}/json", args.topic, args.machine_id),
        Mode::Redi => topic = format!("{}{}", args.topic, args.machine_id),
    }
    println!("Version {}", version);
    println!("Listening topic {} in mode {:?}!", topic, args.mode);

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .mqtt_version(mqtt::MQTT_VERSION_5)
        .server_uri(format!("{}:{}", args.host, args.mqtt_port))
        .client_id(format!("mac_id_{}_{}", args.machine_id, args.exmebus_port))
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
            format!(
                "{} on port {} lost connection",
                args.machine_id, args.exmebus_port
            ),
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
        println!("Exmebusport={}", args.exmebus_port);
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

        let mut exmebus_stream:TcpStream;

        match TcpStream::connect(format!("127.0.0.1:{}", args.exmebus_port)) {
            Ok(ret_strm) => exmebus_stream = ret_strm,
            Err(e) => println!("Could not set timeout: {:?}", e),
        }



        //let mut stream = TcpStream::connect(format!("127.0.0.1:{}", args.exmebus_port))?;
        match exmebus_stream.set_write_timeout(Some(Duration::new(1, 0))) {
            Ok(_) => println!("Timeout set"),
            Err(e) => println!("Could not set timeout: {:?}", e),
        }

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                if msg.retained() {
                    print!("(R) ");
                }

                match args.debug {
                    2 => println!("{}", msg.payload_str()),
                    _ => {}
                }

                match args.mode {
                    Mode::JSON => {
                        match to_exmebus_better(&msg.payload_str()) {
                            Ok(retvec) => {
                                for mut emsg in retvec {
                                    match emsg.exmebusify() {
                                        Ok(bt) => {
                                            if args.debug >= 1 {
                                                println!("{:?}", emsg);
                                            }
                                            match exmebus_stream.write(&bt) {
                                                Ok(_) => (), // Do not do anything when everything just works fine!
                                                Err(e) => {
                                                    println!("Should be stored to redis {:?}", e);
                                                    exmebus_stream = TcpStream::connect(format!("127.0.0.1:{}",args.exmebus_port))?;
                                                    match exmebus_stream.set_write_timeout(Some(
                                                        Duration::new(1, 0),
                                                    )) {
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
                    }
                    // Redi-mode has not been implemented
                    Mode::Redi => {}
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
