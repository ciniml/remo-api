# Unofficial Rust implementation of Nature Remo Cloud API parser.

## 概要

Nature RemoシリーズのCloud APIが返すJSONデータを解析し、各種情報を取り出すためのライブラリ (非公式)

## サポートしているAPI

* `GET /1/devices` https://swagger.nature.global/#/default/get_1_devices
    * ユーザーが操作可能なRemoデバイス一覧と状態を取得する

## 対応予定

* `GET /1/appliances` https://swagger.nature.global/#/default/get_1_appliances
    * 登録されている制御対象の機器一覧と状態を取得する。

## 使い方

`nature_api::read_devices` に `embedded_io::Reader` を実装した型への参照、ストリームの長さ、デバイス情報を処理するコールバックを指定して呼び出すと、Reader実装型から読み出したデータを解析してデバイス情報が確定するたびに、コールバックが呼び出されます。

```rust
use embedded_io::adapters;
use nature_api::read_devices;
use std::{fs::File, io::Read};

fn main() {
    let mut file = File::open("data/devices.json").unwrap();
    let file_length = file.metadata().unwrap().len();
    let mut reader = embedded_io::adapters::FromStd::new(&mut file);
    let mut num_devices = 0;
    read_devices(
        &mut reader,
        Some(file_length as usize),
        |device, sub_node| {
            if sub_node.is_none() {
                num_devices += 1;
            }
            println!("{:?} {:?}", device, sub_node);
        },
    )
    .unwrap();
    println!("num_devices: {}", num_devices);
}
```

コールバックの第一引数は `&nature_api::Device`, 第二引数は `Option(&DeviceSubNode)` となっています。

## ライセンス

本ライブラリはMIT Licenseの下で使用可能です。
