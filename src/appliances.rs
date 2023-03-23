// Appliance model parser for Remo Cloud API.
// Copyright 2022-2023 Kenta Ida 
// SPDX-License-Identifier: MIT
//
use core::{str::FromStr};

use heapless::{String, Vec};
use fuga_json_seq_parser::{JsonScalarValue, ParserCallbackAction, JsonNode, JsonNumber};
use fuga_json_seq_parser::Parser as JsonParser;
use fuga_json_seq_parser::ParserError as JsonParserError;

use uuid::Uuid;
use crate::{config::*, Device};
use crate::common_types::*;
use crate::node_key::*;
use crate::device::MacAddress;
use crate::parser_options::{ParserOptions, copy_string_option};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Appliance {
    pub id: Uuid,
    pub type_: ApplianceType,
    pub nickname: String<MAX_NICKNAME_LEN>,
    pub image: String<MAX_IMAGE_LEN>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ApplianceType {
    AC,
    TV,
    Light,
    IR,
    SmartMeter,
    ElectricWaterHeater,
    PowerDistMeter,
    EVCD,
    SolarPower,
    StorageBattery,
    QrioLock,
    MorninPlus,
    //Custom(String<16>),
}
impl Default for ApplianceType {
    fn default() -> Self {
        Self::AC
    }
}

impl<'a> TryFrom<&'a str> for ApplianceType {
    type Error = ();
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        match s {
            "AC" => Ok(Self::AC),
            "TV" => Ok(Self::TV),
            "LIGHT" => Ok(Self::Light),
            "IR" => Ok(Self::IR),
            "EL_SMART_METER" => Ok(Self::SmartMeter),
            "EL_ELECTRIC_WATER_HEATER" => Ok(Self::ElectricWaterHeater),
            "EL_POWER_DIST_METER" => Ok(Self::PowerDistMeter),
            "EL_EVCD" => Ok(Self::EVCD),
            "EL_SOLAR_POWER" => Ok(Self::SolarPower),
            "EL_STORAGE_BATTERY" => Ok(Self::StorageBattery),
            "QRIO_LOCK" => Ok(Self::QrioLock),
            "MORNIN_PLUS" => Ok(Self::MorninPlus),
            //s => Ok(Self::Custom(String::from_str(s).map_err(|_| ())?)),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EchonetLiteProperty {
    pub name: String<MAX_ECHONET_LITE_NAME_LEN>,
    pub epc: u32,
    pub val: String<MAX_ECHONET_LITE_VALUE_LEN>,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ApplianceModel {
    pub id: Uuid,
    pub country: String<MAX_COUNTRY_LEN>,
    pub manufacturer: String<MAX_MANUFACTURER_LEN>,
    pub remote_name: String<MAX_REMOTE_NAME_LEN>,
    pub series: String<MAX_SERIES_LEN>,
    pub name: String<MAX_MODEL_NAME_LEN>,
    pub image: String<MAX_IMAGE_LEN>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ApplianceSubNode {
    Device(Device),
    Model(ApplianceModel),
    EchonetLiteProperty(EchonetLiteProperty),
}

type AppliancesParser = JsonParser<REQUIRED_APPLIANCES_PARSER_BUFFER_LEN, 10>;

#[derive(Clone, Copy, Debug)]
enum AppliancesParserState {
    Start,
    AppliancesArray,
    ApplianceMap,
    DeviceMap,
    ModelMap,
    SmartMeterMap,
    EchonetLitePropertiesArray,
    EchonetLitePropertyMap,
    UnknownMap,
    UnknownArray,
}
impl AppliancesParserState {
    fn is_map_state(&self) -> bool {
        match self {
            Self::ApplianceMap => true,
            Self::DeviceMap => true,
            Self::ModelMap => true,
            Self::SmartMeterMap => true,
            Self::EchonetLitePropertyMap => true,
            Self::UnknownMap => true,
            _ => false,
        }
    }
    fn is_array_state(&self) -> bool {
        match self {
            Self::AppliancesArray => true,
            Self::EchonetLitePropertiesArray => true,
            Self::UnknownArray => true,
            _ => false,
        }
    }
}

pub fn read_appliances<R: embedded_io::blocking::Read, F>(
    reader: &mut R,
    total_length: Option<usize>,
    options: &ParserOptions,
    mut callback: F,
) -> Result<(), JsonParserError<R::Error, ModelNodeParseError>>
where
    F: for<'a> FnMut(&'a Appliance, Option<&'a ApplianceSubNode>),
{
    let mut parser = AppliancesParser::new();
    parser.set_bytes_remaining(total_length);
    let mut appliance = Appliance::default();
    let mut subnode = ApplianceSubNode::Device(Device::default());
    let mut state = AppliancesParserState::Start;
    let mut node_key = None;
    let mut state_stack: Vec<AppliancesParserState, 10> = Vec::new();

    while !parser.parse(reader, |node| {
        let new_state = match (state, node) {
            // Start array
            (state, JsonNode::StartArray) => {
                state_stack.push(state).map_err(|_| ModelNodeParseError::NodeTooDeep)?;
                match (state, node_key.take()) {
                    (AppliancesParserState::Start, _) => AppliancesParserState::AppliancesArray,
                    (AppliancesParserState::SmartMeterMap, Some(ModelNodeKey::EchonetLiteProperties)) => AppliancesParserState::EchonetLitePropertiesArray,
                    (_, _)=> AppliancesParserState::UnknownArray,
                }
            },
            // Start map
            (state, JsonNode::StartMap) => {
                state_stack.push(state).map_err(|_| ModelNodeParseError::NodeTooDeep)?;
                match (state, node_key.take()) {
                    (AppliancesParserState::AppliancesArray, _) => AppliancesParserState::ApplianceMap,
                    (AppliancesParserState::ApplianceMap, Some(ModelNodeKey::Device)) => {
                        subnode = ApplianceSubNode::Device(Device::default());
                        AppliancesParserState::DeviceMap
                    },
                    (AppliancesParserState::ApplianceMap, Some(ModelNodeKey::Model)) => {
                        subnode = ApplianceSubNode::Model(ApplianceModel::default());
                        AppliancesParserState::ModelMap
                    },
                    (AppliancesParserState::ApplianceMap, Some(ModelNodeKey::SmartMeter)) => AppliancesParserState::SmartMeterMap,
                    (AppliancesParserState::EchonetLitePropertiesArray, _) => {
                        subnode = ApplianceSubNode::EchonetLiteProperty(EchonetLiteProperty::default());
                        AppliancesParserState::EchonetLitePropertyMap
                    }
                    (_, _)=> AppliancesParserState::UnknownMap,
                }
            },
            // End array
            (state, JsonNode::EndArray) if state.is_array_state() => {
                state_stack.pop().ok_or(ModelNodeParseError::UnexpectedMapArrayEnd)?
            },
            // End map
            (state, JsonNode::EndMap) if state.is_map_state() => {
                let (dont_invoke_callback, is_subnode) = match state {
                    AppliancesParserState::UnknownMap => (true, true),
                    AppliancesParserState::SmartMeterMap => (true, true),
                    AppliancesParserState::ApplianceMap => (false, false),
                    _ => (false, true), // Appliance sub node
                };
                if !dont_invoke_callback {
                    // Invoke callback
                    if is_subnode {
                        callback(&appliance, Some(&subnode));
                    } else {
                        callback(&appliance, None);

                    }
                }
                state_stack.pop().ok_or(ModelNodeParseError::UnexpectedMapArrayEnd)?
            },
            (map_state, JsonNode::Key(key)) => {
                match key {
                    JsonScalarValue::String(key) => {
                        node_key = ModelNodeKey::try_from(key).ok(); // Store key
                    }
                    _ => {} // Unknown key.
                }
                map_state
            }
            // Process map node for device.
            (AppliancesParserState::DeviceMap, JsonNode::Value(value)) => {
                let device = match subnode {
                    ApplianceSubNode::Device(ref mut device) => device,
                    _ => { return Err(ModelNodeParseError::UnexpectedParserState); },
                };
                if let Some(node_key) = node_key.take() {
                    match (node_key, value) {
                        (ModelNodeKey::Name, JsonScalarValue::String(s)) => {
                            device.name = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::Id, JsonScalarValue::String(s)) => {
                            device.id = Uuid::from_str(s)?
                        }
                        (ModelNodeKey::CreatedAt, JsonScalarValue::String(s)) => {
                            device.created_at = Timestamp::from_str(s)?
                        }
                        (ModelNodeKey::UpdatedAt, JsonScalarValue::String(s)) => {
                            device.updated_at = Timestamp::from_str(s)?
                        }
                        (ModelNodeKey::MacAddress, JsonScalarValue::String(s)) => {
                            device.mac_address = MacAddress::from_str(s)?
                        }
                        (ModelNodeKey::BtMacAddress, JsonScalarValue::String(s)) => {
                            device.bt_mac_address = MacAddress::from_str(s)?
                        }
                        (ModelNodeKey::SerialNumber, JsonScalarValue::String(s)) => {
                            device.serial_number = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::FirmwareVersion, JsonScalarValue::String(s)) => {
                            device.firmware_version = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::TemperatureOffset, JsonScalarValue::Number(n)) => {
                            device.temperature_offset = n.into()
                        }
                        (ModelNodeKey::HumidityOffset, JsonScalarValue::Number(n)) => {
                            device.humidity_offset = n.into()
                        }
                        _ => {} // Ignore unknown nodes.
                    }
                }
                AppliancesParserState::DeviceMap
            }
            // Appliance map
            (AppliancesParserState::ApplianceMap, JsonNode::Value(value)) => {
                if let Some(node_key) = node_key.take() {
                    match (node_key, value) {
                        (ModelNodeKey::NickName, JsonScalarValue::String(s)) => {
                            appliance.nickname = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::Id, JsonScalarValue::String(s)) => {
                            appliance.id = Uuid::from_str(s)?
                        }
                        (ModelNodeKey::Type, JsonScalarValue::String(s)) => {
                            appliance.type_ = ApplianceType::try_from(s).or(Err(ModelNodeParseError::UnexpectedEnumValue))?;
                        }
                        (ModelNodeKey::Image, JsonScalarValue::String(s)) => {
                            appliance.image = copy_string_option(s, options)?;
                        }
                        _ => {} // Ignore unknown nodes.
                    }
                }
                AppliancesParserState::ApplianceMap
            }
            // Model map
            (AppliancesParserState::ModelMap, JsonNode::Value(value)) => {
                let model = match subnode {
                    ApplianceSubNode::Model(ref mut model) => model,
                    _ => { return Err(ModelNodeParseError::UnexpectedParserState); },
                };
                if let Some(node_key) = node_key.take() {
                    match (node_key, value) {
                        (ModelNodeKey::Name, JsonScalarValue::String(s)) => {
                            model.name = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::Id, JsonScalarValue::String(s)) => {
                            model.id = Uuid::from_str(s)?
                        }
                        (ModelNodeKey::Country, JsonScalarValue::String(s)) => {
                            model.country = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::Manufacturer, JsonScalarValue::String(s)) => {
                            model.manufacturer = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::RemoteName, JsonScalarValue::String(s)) => {
                            model.remote_name = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::Series, JsonScalarValue::String(s)) => {
                            model.series = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::Image, JsonScalarValue::String(s)) => {
                            model.image = copy_string_option(s, options)?;
                        }
                        _ => {} // Ignore unknown nodes.
                    }
                }
                AppliancesParserState::ModelMap
            }
            // EchonetLite Property map
            (AppliancesParserState::EchonetLitePropertyMap, JsonNode::Value(value)) => {
                let property = match subnode {
                    ApplianceSubNode::EchonetLiteProperty(ref mut property) => property,
                    _ => { return Err(ModelNodeParseError::UnexpectedParserState); },
                };
                if let Some(node_key) = node_key.take() {
                    match (node_key, value) {
                        (ModelNodeKey::Name, JsonScalarValue::String(s)) => {
                            property.name = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::Epc, JsonScalarValue::Number(JsonNumber::I32(n))) => {
                            property.epc = n as u32;
                        }
                        (ModelNodeKey::Val, JsonScalarValue::String(s)) => {
                            property.val = copy_string_option(s, options)?;
                        }
                        (ModelNodeKey::UpdatedAt, JsonScalarValue::String(s)) => {
                            property.updated_at = Timestamp::from_str(s)?;
                        }
                        _ => {} // Ignore unknown nodes.
                    }
                }
                AppliancesParserState::EchonetLitePropertyMap
            }
            (_, JsonNode::EndArray) => {
                return Err(ModelNodeParseError::UnexpectedMapArrayEnd);
            }
            (_, JsonNode::EndMap) => {
                return Err(ModelNodeParseError::UnexpectedMapArrayEnd);
            }
            (AppliancesParserState::UnknownMap, JsonNode::Value(_)) => {    // Unknown map value
                AppliancesParserState::UnknownMap   // Ignore the value.
            }
            (AppliancesParserState::UnknownArray, JsonNode::Value(_)) => {    // Unknown map value
                AppliancesParserState::UnknownArray   // Ignore the value.
            }
            (_, JsonNode::Value(_)) => {    // Unexpected value node
                return Err(ModelNodeParseError::UnexpectedParserState);
            }
        };
        state = new_state;
        Ok(ParserCallbackAction::Nothing)
    })? {}
    Ok(())
}

#[cfg(test)]
mod test {
    use fuga_json_seq_parser::BufferReader;
    use uuid::uuid;

    use super::*;

    fn create_reader<'a>(input: &'a str) -> (usize, BufferReader<'a>) {
        let total_length = input.as_bytes().len();
        (total_length, BufferReader::new(input.as_bytes()))
    }

    #[test]
    fn test_parse_empty_appliances() {
        let (length, mut reader) = create_reader(
            "
        [
        ]
        ",
        );
        read_appliances(&mut reader, Some(length), &ParserOptions::default(), |_appliance, _sub_node| {
            panic!("callback must not be called for empty appliances.");
        })
        .unwrap();
    }
    #[test]
    fn test_parse_appliances() {
        let (length, mut reader) = create_reader(include_str!("../data/appliances.json"));
        let expected_appliances = [
            Appliance {
                id: uuid!("84875896-9f1e-44df-9f49-7989352eeecf"),
                type_: ApplianceType::AC,
                nickname: String::from("てすとエアコン"),
                image: String::from("ico_ac_1"),
            },
            Appliance {
                id: uuid!("081c5163-ee9e-486e-ba4d-e86a16ea4c9b"),
                type_: ApplianceType::SmartMeter,
                nickname: String::from("スマートメーター"),
                image: String::from("ico_smartmeter"),
            },
        ];
        let expected_subnodes = [
            ApplianceSubNode::Device(Device {
                name: String::from("てすとりも"),
                id: uuid!("8afdef94-43f7-4a16-b499-fbb6286f7438"),
                created_at: Timestamp::from_str("2022-10-14T05:51:30Z").unwrap(),
                updated_at: Timestamp::from_str("2022-10-15T02:15:00Z").unwrap(),
                mac_address: MacAddress::from_str("c8:2b:96:00:11:22").unwrap(),
                bt_mac_address: MacAddress::from_str("c8:2b:96:33:44:55").unwrap(),
                serial_number: String::from("1W300000000000"),
                firmware_version: String::from("Remo/1.9.9"),
                temperature_offset: 0.0,
                humidity_offset: 0.0,
            }),
            ApplianceSubNode::Model(ApplianceModel {
                id: uuid!("2a556fb6-f64b-4bd2-a911-610aa68dfc05"),
                country: String::from("JP"),
                manufacturer: String::from("sharp"),
                remote_name: String::from("a986jb"),
                series: String::from(""),
                name: String::from("Sharp AC 033"),
                image: String::from("ico_ac_1"),
            }),
            ApplianceSubNode::Device(Device {
                name: String::from("Remo E lite"),
                id: uuid!("159c34f6-d99a-46ca-a50a-3440ba7f8c8e"),
                created_at: Timestamp::from_str("2022-10-08T07:49:56Z").unwrap(),
                updated_at: Timestamp::from_str("2022-10-08T07:52:43Z").unwrap(),
                mac_address: MacAddress::from_str("34:ab:95:00:11:22").unwrap(),
                bt_mac_address: MacAddress::from_str("34:ab:95:33:44:55").unwrap(),
                serial_number: String::from("4W012345678901"),
                firmware_version: String::from("Remo-E-lite/1.7.2"),
                temperature_offset: 0.0,
                humidity_offset: 0.0,
            }),
            ApplianceSubNode::Model(ApplianceModel {
                id: uuid!("1eb17958-9a47-4000-8b9d-b3dffaf9616c"),
                country: String::from(""),
                manufacturer: String::from(""),
                remote_name: String::from(""),
                series: String::from(""),
                name: String::from("Smart Meter"),
                image: String::from("ico_smartmeter"),
            }),
            ApplianceSubNode::EchonetLiteProperty(EchonetLiteProperty {
                name: String::from("cumulative_electric_energy_effective_digits"),
                epc: 215,
                val: String::from("7"),
                updated_at: Timestamp::from_str("2022-10-22T11:38:14Z").unwrap(),
            }),
            ApplianceSubNode::EchonetLiteProperty(EchonetLiteProperty {
                name: String::from("normal_direction_cumulative_electric_energy"),
                epc: 224,
                val: String::from("1097158"),
                updated_at: Timestamp::from_str("2022-10-22T11:38:14Z").unwrap(),
            }),
            ApplianceSubNode::EchonetLiteProperty(EchonetLiteProperty {
                name: String::from("cumulative_electric_energy_unit"),
                epc: 225,
                val: String::from("2"),
                updated_at: Timestamp::from_str("2022-10-22T11:38:14Z").unwrap(),
            }),
            ApplianceSubNode::EchonetLiteProperty(EchonetLiteProperty {
                name: String::from("measured_instantaneous"),
                epc: 231,
                val: String::from("397"),
                updated_at: Timestamp::from_str("2022-10-22T11:38:14Z").unwrap(),
            }),
        ];
        let mut expected_appliances_iter = expected_appliances.iter();
        let mut expected_subnodes_iter = expected_subnodes.iter();
        read_appliances(
            &mut reader,
            Some(length),
            &ParserOptions::default(),
            |appliance, sub_node| match sub_node {
                None => {
                    let expected_appliance = expected_appliances_iter.next();
                    assert!(
                        expected_appliance.is_some(),
                        "Extra appliance returned from parser - {:?}",
                        appliance
                    );
                    let expected_appliance = expected_appliance.unwrap();
                    assert_eq!(appliance, expected_appliance, "Appliance mismatch.");
                }
                Some(sub_node) => {
                    let expected_subnode = expected_subnodes_iter.next();
                    assert!(
                        expected_subnode.is_some(),
                        "Extra appliance subnode returned from parser - {:?}",
                        sub_node
                    );
                    let expected_subnode = expected_subnode.unwrap();
                    assert_eq!(sub_node, expected_subnode, "Subnode mismatch.");
                }
            },
        )
        .unwrap();
    }
}
