use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Topic of the MQTT-channel to listen ie. incoming/machine/HD453/json
    #[arg(long)]
    pub topic: String,

    /// Exmebus-port where to forward data
    #[arg(long)]
    pub exmebus_port: u64,

    /// Local mqtt port, 1883 or something
    #[arg(long)]
    pub mqtt_port: u64,

    /// MQTT-server address ie. tcp://localhost
    #[arg(long)]
    pub host: String,

    #[arg(long)]
    pub machine_id: String,

    /// Mode of the parser json/redi
    #[arg(long, value_enum)]
    pub mode: Mode,

    //// Debug mode
    /// Maximum debug level 2 ie. -dd
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    /// Convert incoming data from JSON to redi
    JSON,
    /// Convert incoming data from redi signals to redi (not implemented)
    Redi,
}