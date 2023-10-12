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
mod exme;
mod appargs;
use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt as mqtt;
use std::{env, process, time::Duration};
use clap::{Parser, ValueEnum};
use std::io::prelude::*;
use std::net::TcpStream;
use log::{error, info,warn,debug, Level, LevelFilter};
use crate::appargs::Args;
use crate::appargs::Mode;
use crate::exme::to_exmebus_better;
use env_logger::Env;


fn main() {
    
    let args = Args::parse();
    if args.debug >= 1 {
        println!("{:?}", args);
    }

    match args.debug {
        2 => env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init(),
        1 => env_logger::Builder::from_env(Env::default().default_filter_or("info")).init(),
        _ => env_logger::init(),
    }

    let version = option_env!("PROJECT_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    let host = format!("{}:{}", args.host, args.mqtt_port);
    let port = args.exmebus_port;
    let _topic = args.topic;
    let mode = args.mode;
    let machine_ide = args.machine_id;

    let mut topic = String::new();
    match mode {
        Mode::JSON => topic = format!("{_topic}{machine_ide}/json"),
        Mode::Redi => topic = format!("{_topic}{machine_ide}"),
    }
    info!("Version {}", version);
    info!("Listening topic {} in mode {:?}!", topic, mode);
    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .mqtt_version(mqtt::MQTT_VERSION_5)
        .server_uri(host)
        .client_id(format!("mac_id_{machine_ide}_{port}"))
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        error!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        // Get message stream before connecting.
        let mut strm = cli.get_stream(50);

        // Define the set of options for the connection
        let lwt = mqtt::Message::new(
            "status",
            format!("{machine_ide} on port {port} lost connection"),
            mqtt::QOS_2,
        );

        let conn_opts = mqtt::ConnectOptionsBuilder::new_v5()
            .clean_start(false)
            .properties(mqtt::properties![mqtt::PropertyCode::SessionExpiryInterval => 3600])
            .will_message(lwt)
            .finalize();

        // Make the connection to the broker
        info!("Connecting to the MQTT server...");
        info!("Machine={topic}");
        info!("Exmebusport={port}");
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
        ).await?;
        // Just loop on incoming messages.
        info!("Waiting for messages...");

        // Note that we're not providing a way to cleanly shut down and
        // disconnect. Therefore, when you kill this app (with a ^C or
        // whatever) the server will get an unexpected drop and then
        // should emit the LWT message.

        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))?;
        match stream.set_write_timeout(Some(Duration::new(1, 0))) {
            Ok(_) => info!("Timeout set"),
            Err(e) => info!("Could not set timeout: {:?}", e),
        }

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                if msg.retained() {
                    debug!("(R) ");
                }
                
                match args.debug {
                    2 => debug!("{}",msg.payload_str()),
                    _ => {},
                }

                match mode {
                    Mode::JSON => {
                        match to_exmebus_better(&msg.payload_str()) {
                            Ok(retvec) => {
                                for mut emsg in retvec {
                                    match emsg.exmebusify() {
                                        Ok(bt) => {
                                            if args.debug >= 1 {
                                                debug!("{:?}", emsg);
                                            }
                                            let mors = stream.write(&bt);
                                            match mors {
                                                Ok(_) => (), // Do not do anything when everything just works fine!
                                                Err(e) => {
                                                    error!("Should be stored to redis {:?}", e);
                                                    stream =
                                                        TcpStream::connect(format!("127.0.0.1:{port}"))?;
                                                    match stream
                                                        .set_write_timeout(Some(Duration::new(1, 0)))
                                                    {
                                                        Ok(_) => info!("Timeout set"),
                                                        Err(e) => {
                                                            warn!("Could not set timeout: {:?}", e)
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => warn!("{:?}", e),
                                    }
                                }
                            }
                            Err(e) => warn!("{:?}", e),
                        }
                    },
                    // Redi-mode has not been implemented
                    Mode::Redi => {},
                }
            } else {
                // A "None" means we were disconnected. Try to reconnect...
                warn!("Lost connection. Attempting reconnect.");
                while let Err(err) = cli.reconnect().await {
                    error!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }
                // reconnect also topics when succesfully reconnected
                //cli.subscribe_with_options(
                //    format!("{_topic}{machine_ide}/json"),
                //    2,
                //    mqtt::SubscribeOptions::with_retain_as_published(),
                //    None,
                //).await?;

            }
        }

        // Explicit return type for the async block
        Ok::<(), mqtt::Error>(())
    }) {
        error!("{}", err);
    }
}
