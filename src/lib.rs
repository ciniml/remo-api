// main source file of remo-api crate
// Copyright 2022 Kenta Ida 
// SPDX-License-Identifier: MIT
//

#![no_std]
pub mod json;

use core::{fmt::Write, str::FromStr};

use chrono::{DateTime, Utc};
use heapless::String;
use json::{JsonScalarValue, ParserCallbackAction};
use nom::{
    character::complete::one_of,
    combinator::{map, recognize},
    error::{ContextError, ParseError},
    sequence::{pair, separated_pair},
    IResult,
};
use uuid::Uuid;
pub type Timestamp = DateTime<Utc>;

pub const MAX_FIRMWARE_VERSION_LEN: usize = 32;
pub const MAX_NICKNAME_LEN: usize = 32;
pub const MAX_MODEL_NAME_LEN: usize = 32;
pub const MAX_DEVICE_NAME_LEN: usize = 48;
pub const MAX_MANUFACTURER_LEN: usize = 16;
pub const MAX_REMOTE_NAME_LEN: usize = 16;
pub const MAX_SERIES_LEN: usize = 16;
pub const MAX_IMAGE_LEN: usize = 16;
pub const MAX_COUNTRY_LEN: usize = 8;
pub const MAX_ECHONET_LITE_NAME_LEN: usize = 64;
pub const MAX_ECHONET_LITE_VALUE_LEN: usize = 16;
pub const ID_LEN: usize = 36;
pub const TIMESTAMP_LEN: usize = 20;

const fn max_usize_array(a: &[usize]) -> usize {
    let mut max = 0;
    let mut index = 0;
    while index < a.len() {
        if max < a[index] {
            max = a[index];
        }
        index += 1;
    }
    max
}

pub const REQUIRED_DEVICES_PARSER_BUFFER_LEN: usize = max_usize_array(&[
    MAX_FIRMWARE_VERSION_LEN,
    MAX_NICKNAME_LEN,
    MAX_MODEL_NAME_LEN,
    MAX_DEVICE_NAME_LEN,
    SERIAL_NUMBER_LEN,
    ID_LEN,
    TIMESTAMP_LEN,
]) + 2;

#[derive(Clone, Debug, Default)]
pub struct SensorValue {
    pub val: f32,
    pub created_at: Timestamp,
}
#[derive(Debug, Default)]
pub struct User {
    pub id: Uuid,
    pub nickname: String<MAX_NICKNAME_LEN>,
    pub superuser: bool,
}

const SERIAL_NUMBER_LEN: usize = 14;
pub type SerialNumber = String<SERIAL_NUMBER_LEN>;

#[derive(Clone, Debug, Default)]
pub struct NewestEvents {
    pub temperature: Option<SensorValue>,
    pub humidity: Option<SensorValue>,
    pub illumination: Option<SensorValue>,
    pub motion: Option<SensorValue>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Device {
    pub id: Uuid,
    pub name: String<MAX_DEVICE_NAME_LEN>,
    pub temperature_offset: f32,
    pub humidity_offset: f32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub firmware_version: String<MAX_FIRMWARE_VERSION_LEN>,
    pub mac_address: MacAddress,
    pub bt_mac_address: MacAddress,
    pub serial_number: SerialNumber,
}

#[derive(Debug, Default)]
pub struct Model {
    pub id: Uuid,
    pub country: String<MAX_COUNTRY_LEN>,
    pub manifacturer: String<MAX_MANUFACTURER_LEN>,
    pub remote_name: String<MAX_REMOTE_NAME_LEN>,
    pub series: String<MAX_SERIES_LEN>,
    pub name: String<MAX_MODEL_NAME_LEN>,
    pub image: String<MAX_IMAGE_LEN>,
}

#[derive(Debug)]
pub enum ApplianceType {
    AC,
    TV,
    Light,
    IR,
    SmartMeter,
}
#[derive(Debug, Default)]
pub struct EchonetLiteProperty {
    pub name: String<MAX_ECHONET_LITE_NAME_LEN>,
    pub epc: u32,
    pub val: String<MAX_ECHONET_LITE_VALUE_LEN>,
    pub updated_at: Timestamp,
}

#[derive(Debug)]
pub enum DeviceSubNode {
    User(User),
    NewestEvents(NewestEvents),
}

type DevicesParser = json::Parser<REQUIRED_DEVICES_PARSER_BUFFER_LEN, 6>;

#[derive(Clone, Copy, Debug)]
enum DevicesParserState {
    Start,
    DevicesArray,
    DeviceMap,
    UsersArray,
    UserMap,
    NewestEventsMap,
    NewestEventMap(NewestEventType),
    UnknownMapArray,
}

#[derive(Clone, Copy, Debug)]
enum NewestEventType {
    Temperature,
    Humidity,
    Illumination,
    Motion,
}

enum DeviceNodeKey {
    Name,
    Id,
    CreatedAt,
    UpdatedAt,
    MacAddress,
    BtMacAddress,
    SerialNumber,
    FirmwareVersion,
    TemperatureOffset,
    HumidityOffset,
    Users,
    NickName,
    SuperUser,
    NewestEvents,
    Te,
    Hu,
    Il,
    Mo,
    Val,
}

impl<'a> TryFrom<&'a str> for DeviceNodeKey {
    type Error = ();
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        match s {
            "name" => Ok(Self::Name),
            "id" => Ok(Self::Id),
            "created_at" => Ok(Self::CreatedAt),
            "updated_at" => Ok(Self::UpdatedAt),
            "mac_address" => Ok(Self::MacAddress),
            "bt_mac_address" => Ok(Self::BtMacAddress),
            "serial_number" => Ok(Self::SerialNumber),
            "firmware_version" => Ok(Self::FirmwareVersion),
            "temperature_offset" => Ok(Self::TemperatureOffset),
            "humidity_offset" => Ok(Self::HumidityOffset),
            "users" => Ok(Self::Users),
            "nickname" => Ok(Self::NickName),
            "superuser" => Ok(Self::SuperUser),
            "newest_events" => Ok(Self::NewestEvents),
            "te" => Ok(Self::Te),
            "hu" => Ok(Self::Hu),
            "il" => Ok(Self::Il),
            "mo" => Ok(Self::Mo),
            "val" => Ok(Self::Val),
            _ => Err(()),
        }
    }
}

pub type UnexpectedNodeError = String<64>;

#[derive(Debug)]
pub enum DeviceNodeParseError {
    UuidParseError,
    TimestampParseError,
    MacAddressParseError,
    UnknownNewestEventsType,
    UnexpectedNode(UnexpectedNodeError),
}
impl From<uuid::Error> for DeviceNodeParseError {
    fn from(_: uuid::Error) -> Self {
        Self::UuidParseError
    }
}
impl From<chrono::ParseError> for DeviceNodeParseError {
    fn from(_: chrono::ParseError) -> Self {
        Self::TimestampParseError
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);

impl Default for MacAddress {
    fn default() -> Self {
        Self([0u8; 6])
    }
}

fn parse_byte_string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, u8, E> {
    let hex_char_list = "0123456789ABCDEFabcdef";
    map(
        recognize(pair(one_of(hex_char_list), one_of(hex_char_list))),
        |byte_str| u8::from_str_radix(byte_str, 16).unwrap(),
    )(i)
}

fn parse_mac_address<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, [u8; 6], E> {
    let delimiter_list = ":-";

    map(
        separated_pair(
            parse_byte_string,
            one_of(delimiter_list),
            separated_pair(
                parse_byte_string,
                one_of(delimiter_list),
                separated_pair(
                    parse_byte_string,
                    one_of(delimiter_list),
                    separated_pair(
                        parse_byte_string,
                        one_of(delimiter_list),
                        separated_pair(
                            parse_byte_string,
                            one_of(delimiter_list),
                            parse_byte_string,
                        ),
                    ),
                ),
            ),
        ),
        |(l, r)| [l, r.0, r.1 .0, r.1 .1 .0, r.1 .1 .1 .0, r.1 .1 .1 .1],
    )(i)
}

impl FromStr for MacAddress {
    type Err = DeviceNodeParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, mac_address) = parse_mac_address(s)
            .map_err(|_: nom::Err<()>| DeviceNodeParseError::MacAddressParseError)?;
        Ok(Self(mac_address))
    }
}

pub fn read_devices<R: embedded_io::blocking::Read, F>(
    reader: &mut R,
    total_length: Option<usize>,
    mut callback: F,
) -> Result<(), json::ParserError<R::Error, DeviceNodeParseError>>
where
    F: for<'a> FnMut(&'a Device, Option<&'a DeviceSubNode>),
{
    let mut parser = DevicesParser::new();
    parser.set_bytes_remaining(total_length);
    let mut device = Device::default();
    let mut subnode = DeviceSubNode::User(User::default());
    let mut state = DevicesParserState::Start;
    let mut node_key = None;
    let mut unknown_map_depth = 0;
    let mut unknown_array_depth = 0;

    while !parser.parse(reader, |node| {
        let new_state = match (state, node) {
            (DevicesParserState::Start, json::JsonNode::StartArray) => {
                DevicesParserState::DevicesArray
            }
            (DevicesParserState::DevicesArray, json::JsonNode::EndArray) => {
                DevicesParserState::Start
            }
            (DevicesParserState::DevicesArray, json::JsonNode::StartMap) => {
                DevicesParserState::DeviceMap
            }
            (DevicesParserState::DeviceMap, json::JsonNode::EndMap) => {
                DevicesParserState::DevicesArray
            }
            (map_state, json::JsonNode::Key(key)) => {
                match key {
                    JsonScalarValue::String(key) => {
                        node_key = DeviceNodeKey::try_from(key).ok(); // Store key
                    }
                    _ => {}
                }
                map_state
            }
            // Process map node for device.
            (DevicesParserState::DeviceMap, json::JsonNode::Value(value)) => {
                if let Some(node_key) = node_key.take() {
                    match (node_key, value) {
                        (DeviceNodeKey::Name, JsonScalarValue::String(s)) => {
                            device.name = String::from(s)
                        }
                        (DeviceNodeKey::Id, JsonScalarValue::String(s)) => {
                            device.id = Uuid::from_str(s)?
                        }
                        (DeviceNodeKey::CreatedAt, JsonScalarValue::String(s)) => {
                            device.created_at = Timestamp::from_str(s)?
                        }
                        (DeviceNodeKey::UpdatedAt, JsonScalarValue::String(s)) => {
                            device.updated_at = Timestamp::from_str(s)?
                        }
                        (DeviceNodeKey::MacAddress, JsonScalarValue::String(s)) => {
                            device.mac_address = MacAddress::from_str(s)?
                        }
                        (DeviceNodeKey::BtMacAddress, JsonScalarValue::String(s)) => {
                            device.bt_mac_address = MacAddress::from_str(s)?
                        }
                        (DeviceNodeKey::SerialNumber, JsonScalarValue::String(s)) => {
                            device.serial_number = String::from(s)
                        }
                        (DeviceNodeKey::FirmwareVersion, JsonScalarValue::String(s)) => {
                            device.firmware_version = String::from(s)
                        }
                        (DeviceNodeKey::TemperatureOffset, JsonScalarValue::Number(n)) => {
                            device.temperature_offset = n.into()
                        }
                        (DeviceNodeKey::HumidityOffset, JsonScalarValue::Number(n)) => {
                            device.humidity_offset = n.into()
                        }
                        _ => {} // Ignore unknown nodes.
                    }
                }
                DevicesParserState::DeviceMap
            }
            (DevicesParserState::DeviceMap, json::JsonNode::StartArray) => {
                match node_key.take() {
                    Some(DeviceNodeKey::Users) => {
                        // Call callback for current device
                        callback(&device, None);
                        DevicesParserState::UsersArray
                    }
                    _ => {
                        unknown_array_depth += 1;
                        DevicesParserState::UnknownMapArray
                    }
                }
            }
            (DevicesParserState::DeviceMap, json::JsonNode::StartMap) => match node_key.take() {
                Some(DeviceNodeKey::NewestEvents) => {
                    subnode = DeviceSubNode::NewestEvents(NewestEvents::default());
                    DevicesParserState::NewestEventsMap
                }
                _ => {
                    unknown_map_depth += 1;
                    DevicesParserState::UnknownMapArray
                }
            },

            // Process users array
            (DevicesParserState::UsersArray, json::JsonNode::EndArray) => {
                DevicesParserState::DeviceMap
            } // Return to device map state
            (DevicesParserState::UsersArray, json::JsonNode::StartMap) => {
                subnode = DeviceSubNode::User(User::default());
                DevicesParserState::UserMap
            }
            // Process user map
            (DevicesParserState::UserMap, json::JsonNode::Value(value)) => {
                if let DeviceSubNode::User(ref mut user) = &mut subnode {
                    if let Some(node_key) = node_key.take() {
                        match (node_key, value) {
                            (DeviceNodeKey::Id, JsonScalarValue::String(s)) => {
                                user.id = Uuid::from_str(s)?
                            }
                            (DeviceNodeKey::NickName, JsonScalarValue::String(s)) => {
                                user.nickname = String::from(s)
                            }
                            (DeviceNodeKey::SuperUser, JsonScalarValue::Boolean(v)) => {
                                user.superuser = v
                            }
                            _ => {} // Ignore unknown nodes.
                        }
                    }
                }
                DevicesParserState::UserMap
            }
            (DevicesParserState::UserMap, json::JsonNode::EndMap) => {
                callback(&device, Some(&subnode));
                DevicesParserState::UsersArray // Return to users array.
            }
            // Process newest_events map
            (DevicesParserState::NewestEventsMap, json::JsonNode::EndMap) => {
                callback(&device, Some(&subnode));
                DevicesParserState::DeviceMap // Return to device map state
            }
            (DevicesParserState::NewestEventsMap, json::JsonNode::StartMap) => {
                let newest_events = if let DeviceSubNode::NewestEvents(ref mut newest_events) =
                    &mut subnode
                {
                    newest_events
                } else {
                    panic!(
                        "sub_node must contains newest_events at (NewestEventsMap, StartMap) state"
                    );
                };

                match node_key.take() {
                    Some(DeviceNodeKey::Te) => {
                        newest_events.temperature = Some(SensorValue::default());
                        DevicesParserState::NewestEventMap(NewestEventType::Temperature)
                    }
                    Some(DeviceNodeKey::Hu) => {
                        newest_events.humidity = Some(SensorValue::default());
                        DevicesParserState::NewestEventMap(NewestEventType::Humidity)
                    }
                    Some(DeviceNodeKey::Il) => {
                        newest_events.illumination = Some(SensorValue::default());
                        DevicesParserState::NewestEventMap(NewestEventType::Illumination)
                    }
                    Some(DeviceNodeKey::Mo) => {
                        newest_events.motion = Some(SensorValue::default());
                        DevicesParserState::NewestEventMap(NewestEventType::Motion)
                    }
                    _ => return Err(DeviceNodeParseError::UnknownNewestEventsType),
                }
            }
            // Process maps in a newest_events map
            (
                DevicesParserState::NewestEventMap(newest_event_type),
                json::JsonNode::Value(value),
            ) => {
                if let DeviceSubNode::NewestEvents(ref mut newest_events) = &mut subnode {
                    let sensor_value = match newest_event_type {
                        NewestEventType::Temperature => newest_events.temperature.as_mut().unwrap(),
                        NewestEventType::Humidity => newest_events.humidity.as_mut().unwrap(),
                        NewestEventType::Illumination => {
                            newest_events.illumination.as_mut().unwrap()
                        }
                        NewestEventType::Motion => newest_events.motion.as_mut().unwrap(),
                    };
                    match (node_key.take(), value) {
                        (Some(DeviceNodeKey::Val), JsonScalarValue::Number(n)) => {
                            sensor_value.val = n.into()
                        }
                        (Some(DeviceNodeKey::CreatedAt), JsonScalarValue::String(s)) => {
                            sensor_value.created_at = Timestamp::from_str(s)?
                        }
                        _ => {}
                    }
                }
                DevicesParserState::NewestEventMap(newest_event_type)
            }
            (DevicesParserState::NewestEventMap(_), json::JsonNode::EndMap) => {
                DevicesParserState::NewestEventsMap
            }

            // Process unknown nodes in device nodes.
            (DevicesParserState::UnknownMapArray, json::JsonNode::StartArray) => {
                unknown_array_depth += 1;
                DevicesParserState::UnknownMapArray
            }
            (DevicesParserState::UnknownMapArray, json::JsonNode::StartMap) => {
                unknown_map_depth += 1;
                DevicesParserState::UnknownMapArray
            }
            (DevicesParserState::UnknownMapArray, json::JsonNode::EndArray) => {
                unknown_array_depth -= 1;
                if unknown_array_depth == 0 && unknown_map_depth == 0 {
                    DevicesParserState::DeviceMap
                } else {
                    DevicesParserState::UnknownMapArray
                }
            }
            (DevicesParserState::UnknownMapArray, json::JsonNode::EndMap) => {
                unknown_map_depth -= 1;
                if unknown_array_depth == 0 && unknown_map_depth == 0 {
                    DevicesParserState::DeviceMap
                } else {
                    DevicesParserState::UnknownMapArray
                }
            }
            (state, json_node) => {
                let mut error = UnexpectedNodeError::new();
                write!(&mut error, "{:?}", (state, json_node)).ok();
                return Err(DeviceNodeParseError::UnexpectedNode(error));
            }
        };
        state = new_state;
        Ok(ParserCallbackAction::Nothing)
    })? {}
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::json::BufferReader;
    use uuid::uuid;

    use super::*;

    fn create_reader<'a>(input: &'a str) -> (usize, BufferReader<'a>) {
        let total_length = input.as_bytes().len();
        (total_length, BufferReader::new(input.as_bytes()))
    }

    #[test]
    fn test_parse_mac_address_colon() {
        let parsed = MacAddress::from_str("f0:08:d1:00:11:22").unwrap();
        assert_eq!(parsed, MacAddress([0xf0, 0x08, 0xd1, 0x00, 0x11, 0x22]));
    }
    #[test]
    fn test_parse_mac_address_hyphen() {
        let parsed = MacAddress::from_str("f0-08-d1-00-11-22").unwrap();
        assert_eq!(parsed, MacAddress([0xf0, 0x08, 0xd1, 0x00, 0x11, 0x22]));
    }

    #[test]
    fn test_parse_empty_devices() {
        let (length, mut reader) = create_reader(
            "
        [
        ]
        ",
        );
        read_devices(&mut reader, Some(length), |_device, _sub_node| {
            panic!("callback must not be called for empty devices.");
        })
        .unwrap();
    }
    #[test]
    fn test_parse_devices() {
        let (length, mut reader) = create_reader(include_str!("../data/devices.json"));
        let expected_devices = [
            Device {
                name: String::from("test remo device hoge"),
                id: uuid!("f262cb0c-a853-47bb-9559-44d0f2c4d6e2"),
                created_at: Timestamp::from_str("2022-10-18T06:42:59Z").unwrap(),
                updated_at: Timestamp::from_str("2022-10-19T05:22:28Z").unwrap(),
                mac_address: MacAddress::from_str("e8:db:84:00:11:22").unwrap(),
                bt_mac_address: MacAddress::from_str("e8:db:84:22:33:44").unwrap(),
                serial_number: String::from("2B012345678901"),
                firmware_version: String::from("Remo-mini/1.10.0"),
                temperature_offset: -0.5,
                humidity_offset: 1.5,
            },
            Device {
                name: String::from("Remo"),
                id: uuid!("12948215-568a-49ca-be45-c556e8140c56"),
                created_at: Timestamp::from_str("2022-10-07T05:57:52Z").unwrap(),
                updated_at: Timestamp::from_str("2022-10-07T05:57:52Z").unwrap(),
                mac_address: MacAddress::from_str("24:6f:28:00:11:22").unwrap(),
                bt_mac_address: MacAddress::from_str("24:6f:28:22:33:44").unwrap(),
                serial_number: String::from("1W012345678901"),
                firmware_version: String::from("Remo/1.10.0"),
                temperature_offset: 1.0,
                humidity_offset: 0.0,
            },
            Device {
                name: String::from("Remo E lite"),
                id: uuid!("b08bdb7b-a2ad-4c3c-88f6-68645ae98077"),
                created_at: Timestamp::from_str("2022-08-22T05:51:50Z").unwrap(),
                updated_at: Timestamp::from_str("2022-10-03T04:16:16Z").unwrap(),
                mac_address: MacAddress::from_str("f0:08:d1:00:11:22").unwrap(),
                bt_mac_address: MacAddress::from_str("f0:08:d1:22:33:44").unwrap(),
                serial_number: String::from("4W012345678901"),
                firmware_version: String::from("Remo-E-lite/1.7.4"),
                temperature_offset: 0.0,
                humidity_offset: 0.0,
            },
        ];
        let mut expected_devices_iter = expected_devices.iter();
        read_devices(
            &mut reader,
            Some(length),
            |device, sub_node| match sub_node {
                None => {
                    let expected_device = expected_devices_iter.next();
                    assert!(
                        expected_device.is_some(),
                        "Extra device returned from parser - {:?}",
                        device
                    );
                    let expected_device = expected_device.unwrap();
                    assert_eq!(device, expected_device, "Device mismatch.");
                }
                _ => {}
            },
        )
        .unwrap();
    }
}
