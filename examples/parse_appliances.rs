use embedded_io::adapters;
use remo_api::read_appliances;
use std::{fs::File, io::Read};

fn main() {
    let mut file = File::open("data/appliances.json").unwrap();
    let file_length = file.metadata().unwrap().len();
    let mut reader = embedded_io::adapters::FromStd::new(&mut file);
    let mut num_appliances = 0;
    read_appliances(
        &mut reader,
        Some(file_length as usize),
        |device, sub_node| {
            if sub_node.is_none() {
                num_appliances += 1;
            }
            println!("{:?} {:?}", device, sub_node);
        },
    )
    .unwrap();
    println!("num_appliances: {}", num_appliances);
}
