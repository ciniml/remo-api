// Model parameter configuration of Remo Cloud API models.
// Copyright 2022-2023 Kenta Ida 
// SPDX-License-Identifier: MIT
//
pub const MAX_FIRMWARE_VERSION_LEN: usize = 48;
pub const MAX_NICKNAME_LEN: usize = 48;
pub const MAX_MODEL_NAME_LEN: usize = 64;
pub const MAX_DEVICE_NAME_LEN: usize = 48;
pub const MAX_MANUFACTURER_LEN: usize = 32;
pub const MAX_REMOTE_NAME_LEN: usize = 32;
pub const MAX_SERIES_LEN: usize = 32;
pub const MAX_IMAGE_LEN: usize = 32;
pub const MAX_COUNTRY_LEN: usize = 8;
pub const MAX_ECHONET_LITE_NAME_LEN: usize = 64;
pub const MAX_ECHONET_LITE_VALUE_LEN: usize = 16;
pub const ID_LEN: usize = 36;
pub const TIMESTAMP_LEN: usize = 20;
pub const SERIAL_NUMBER_LEN: usize = 14;

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

pub const REQUIRED_APPLIANCES_PARSER_BUFFER_LEN: usize = REQUIRED_DEVICES_PARSER_BUFFER_LEN;