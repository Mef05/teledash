mod json;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DataType{
    U8,
    F32,
    F64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Endian {
    BIG,
    LITTLE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEntry {
    pub name: String,
    pub data_type: DataType,
    pub endian: Endian,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorPerset {
    pub id: u8,
    pub name: String,
    pub fields: Vec<FieldEntry>,
}


#[derive(Debug, Clone, Serialize)]
pub struct DecodedTelemetryPacket {
    pub id: u8,
    pub name: Option<String>,
    pub data_type: Option<String>,
    pub unit: Option<String>,
}
