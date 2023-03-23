// main source file of remo-api crate
// Copyright 2022-2023 Kenta Ida 
// SPDX-License-Identifier: MIT
//

//! # Unofficial Rust implementation of Remo Cloud API parser.
//! 
//! ## 概要
//! 
//! Nature RemoシリーズのCloud APIが返すJSONデータを解析し、各種情報を取り出すためのライブラリ (非公式)
//!

#![no_std]
pub mod config;
mod device;
mod appliances;
mod common_types;
mod node_key;
mod parser_options;

pub use device::*;
pub use appliances::*;
pub use common_types::*;
pub use parser_options::ParserOptions;