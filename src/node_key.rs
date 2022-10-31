
#[derive(Debug)]
pub enum ModelNodeKey {
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

    Device,
    Model,
    Type,
    Manufacturer,
    Country,
    RemoteName,
    Series,
    Image,
    SmartMeter,
    EchonetLiteProperties,
    Epc,
}

impl<'a> TryFrom<&'a str> for ModelNodeKey {
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
            "device" => Ok(Self::Device),
            "type" => Ok(Self::Type),
            "model" => Ok(Self::Model),
            "manufacturer" => Ok(Self::Manufacturer),
            "country" => Ok(Self::Country),
            "remote_name" => Ok(Self::RemoteName),
            "series" => Ok(Self::Series),
            "image" => Ok(Self::Image),
            "smart_meter" => Ok(Self::SmartMeter),
            "echonetlite_properties" => Ok(Self::EchonetLiteProperties),
            "epc" => Ok(Self::Epc),
            _ => Err(()),
        }
    }
}