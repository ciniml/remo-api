// Common types for Remo API models.
// Copyright 2022 Kenta Ida 
// SPDX-License-Identifier: MIT
//

use chrono::{DateTime, Utc};
use heapless::String;

use crate::config::SERIAL_NUMBER_LEN;

pub type Timestamp = DateTime<Utc>;
pub type SerialNumber = String<SERIAL_NUMBER_LEN>;
