use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

pub const MAX_EXMEBUS_PKG_SIZE: usize = 1000;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SignalDataType {
    V_VOID,
    V_BIT,
    V_SIGNED_CHAR,
    V_SIGNED_SHORT,
    V_SIGNED_LONG,
    V_UNSIGNED_CHAR,
    V_UNSIGNED_SHORT,
    V_UNSIGNED_LONG,
    V_FLOAT,
    V_STRING,
    V_HEX,
    V_BINARY,
    V_BE_SIGNED_SHORT,
    V_BE_SIGNED_LONG,
    V_BE_UNSIGNED_SHORT,
    V_BE_UNSIGNED_LONG,
    V_UNIX_TIME,
    V_TWO_BIT,
    V_SIGNED_LONG_LONG,
    V_UNSIGNED_LONG_LONG,
    V_DOUBLE,
    V_IP_ADDRESS_NETWORK,
    V_IP_ADDRESS_HOST,
    V_STRUCT = 101,
}

#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ExmebusMessageType {
    EMT_CAN_MESSAGE,
    EMT_CONTROL_PACKET,
    EMT_LOG_MESSAGE,
    EMT_ALARM_MESSAGE,
    EMT_SYSTEM_COMMAND,
    EMT_RESOURCE_MESSAGE,
    EMT_AUTHENTICATION,
    EMT_HEARTBEAT,
    EMT_IO_MESSAGE,
    EMT_SYNC_MESSAGE,
    EMT_REMOTE_CONTROL,
    EMT_GUI_STATUS,
    EMT_LOGICAL_IO_SIGNAL_PROPERTY,
    EMT_IOMUX_IDENTIFICATION,
    EMT_FILTER_MESSAGE,
    EMT_MODULE_INFO_MESSAGE,
    EMT_FILE_TRANSFER_MESSAGE,
    EMT_DATA_SIGNAL_MESSAGE,
    EMT_DATA_COLLECTION_TABLE_MESSAGE,
    EMT_DATA_SIGNAL_DEFINITION_MESSAGE,
    EMT_OWN_DATA_SIGNAL_MESSAGE,
    EMT_TIMESTAMP,
    EMT_SIGNED_MESSAGE,
    EMT_TOTAL,
}

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// Defines the group where the sent data should go into
pub enum DataSignalGroups {
    /// Signal metadata sent by RediServer. See IS_*
    DSG_INFO,
    /// Common for all Guitu projects, defined in model database
    DSG_COMMON,
    /// J1939 SPNs
    DSG_SPN,
    /// Global metadata sent from Exertus modules
    DSG_SYSTEM_INFO,
    /// Company-specific signals, defined on machine in application.info files
    DSG_APPLICATION_VERSION,
    /// Product-specific signals, defined in Guitu project database
    DSG_USER = 100,
}

/// Defines the signal sample type
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SignalSampleType {
    /// latest value of the signal
    SST_CURRENTVALUE,
    /// average value of the signal samples in the current time window
    SST_AVERAGE,
    /// smallest value in the current time window
    SST_MINIMUM,
    /// biggest value in the current time window
    SST_MAXIMUM,
    /// how many times the value has changed during the current time window
    SST_CHANGE_COUNT,
    /// how many times the value has changed from zero to nonzero during the current time window
    SST_ENABLE_COUNT,
    /// how many times the value has changed from nonzero to zero during the current time window
    SST_DISABLE_COUNT,
    /// how many valid samples have been received in the current time window
    SST_VALID_SAMPLE_COUNT,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OwnDataSignalPacket {
    pub packet_length: u16,        // Total packet len
    pub packet_id: u16,            // Packet type, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
    pub sample_packet_length: u16, // Packet payload len
    pub signal_sample_type: u8,    // current value, average, minimum or maximum, see SST_
    pub signal_view_type: u8,
    pub signal_number: u16,
    pub signal_group: u16, // see DSG_
    pub milliseconds: i64, // Timestamp in milliseconds since 1601
    pub data: Vec<u8>,
}

impl Default for OwnDataSignalPacket {
    fn default() -> OwnDataSignalPacket {
        OwnDataSignalPacket {
            packet_length: 0, // Paketin kokonaispituus
            packet_id: ExmebusMessageType::EMT_OWN_DATA_SIGNAL_MESSAGE as u16, // Paketin tyyppi, EMT_OWN_DATA_SIGNAL_MESSAGE, 21
            sample_packet_length: 0,                                           // pituus tavuina
            signal_sample_type: SignalSampleType::SST_CURRENTVALUE as u8, // current value, average, minimum or maximum, see SST_
            signal_view_type: 0,
            signal_number: 0,
            signal_group: DataSignalGroups::DSG_USER as u16, // see DSG_
            milliseconds: 0,                                 // Timestamp in milliseconds since 1601
            // data len samplePacketLength - 16
            data: Vec::new(),
        }
    }
}

#[derive( Debug, Clone, Copy)]
struct ClientPropertiesPacket {
    /* ExmebusServer fills these fields */
    ip: u32,        /* IP address */
    ipv6: [u8; 16], /* IP6 address (for future, not yet supported) */
    port: u32,
    bytes_received: u64,   /* amount of bytes received from the client */
    bytes_sent: u64,       /* amount of bytes sent to the client */
    uptime: i64,           /* in milliseconds */
    alive: u8,             /* is client alive */
    is_default_client: u8, /* RediServer marks the default client here */
    reservedbyte2: u8,
    send_period: u32, /* how often server sends data to client, default: 0 (instantly) */
    reserved2: i32,

    /* ExmebusClient should fill these fields */
    client_state: u32, /* client-specific information */
    role: u16,         /* see CR_* */
    number: u16,       /* for virtual modules this is the module number */
    process_id: u32,   /* process ID in Windows */
    //name: [i8; 1000],  /* null-terminated UTF8 string */
}
union CommandData {
    data: [u8; MAX_EXMEBUS_PKG_SIZE],
    derp: ClientPropertiesPacket,
}

#[allow(non_camel_case_types)]
pub struct ResourcePacketHeader {
    packet_length: u16,           /* packet total size in bytes */
    packet_id: u16,               /* packet type: 6 (EMT_RESOURCE_MESSAGE) */
    source_server_uuid: [i8; 16], /* UUID of the source server (if 0, server fills) */
    source_client_id: i32,        /* source client ID (if 0, server fills) */
    source_resource_id: i32, /* client-specific number for the resource. 0 means client itself. */
    source_resource_type: i32, /* see RESOURCE_TYPE_* */
    reserved: i32,
    destination_server_uuid: [i8; 16], /* UUID of the destination server. 0: only to this server, FFFF...FFFF: to all servers. */
    destination_client_id: i32,        /* destination client. 0: to all clients */
    destination_resource_id: i32,      /* client-specific number for the resource */
    destination_resource_type: i32,    /* see RESOURCE_TYPE_* */
    reserved2: i32,
    command: i32,
    command_data: CommandData,
    /* see RESOURCE_PACKET_* */
    /* data depends on the command, see RESOURCE_PACKET_* */
    /*union {
         data[MAX_EXMEBUS_PKG_SIZE], /* data for other kind of resource packets */
        struct BusPropertiesPacket busProperties,
        struct ClientPropertiesPacket clientProperties,
        struct InitBusPacket initBus,
        struct BindResourcePacket bindResource,
        struct ServerPropertiesPacket serverProperties,
        struct DisplayPropertiesPacket display,
        struct BridgePropertiesPacket bridge,
        struct EventPacket event,
        struct EventDefinitionPacket eventDefinition,
    },*/
}
