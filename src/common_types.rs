// Common types for Remo API models.
// Copyright 2022-2023 Kenta Ida 
// SPDX-License-Identifier: MIT
//

use chrono::{DateTime, Utc};
use heapless::String;

use crate::config::SERIAL_NUMBER_LEN;

pub type Timestamp = DateTime<Utc>;
pub type SerialNumber = String<SERIAL_NUMBER_LEN>;


#[derive(Debug)]
pub enum ModelNodeParseError {
    UuidParseError,
    TimestampParseError,
    MacAddressParseError,
    UnexpectedEnumValue,
    UnknownNewestEventsType,
    NodeTooDeep,
    StringTooLong,
    UnexpectedMapArrayEnd,
    UnexpectedParserState,
    UnexpectedNode(UnexpectedNodeError),
}

pub type UnexpectedNodeError = String<64>;

impl From<uuid::Error> for ModelNodeParseError {
    fn from(_: uuid::Error) -> Self {
        Self::UuidParseError
    }
}
impl From<chrono::ParseError> for ModelNodeParseError {
    fn from(_: chrono::ParseError) -> Self {
        Self::TimestampParseError
    }
}