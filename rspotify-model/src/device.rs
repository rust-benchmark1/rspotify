use crate::DeviceType;
use serde::{Deserialize, Serialize};

/// Device object
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Device {
    pub id: Option<String>,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub _type: DeviceType,
    pub volume_percent: Option<u32>,
}

/// Intermediate device payload object
#[derive(Deserialize)]
pub struct DevicePayload {
    pub devices: Vec<Device>,
}

#[test]
fn test_devices() {
    let json_str = r#"
        {
            "devices" : [ {
                "id" : "5fbb3ba6aa454b5534c4ba43a8c7e8e45a63ad0e",
                "is_active" : false,
                "is_private_session": true,
                "is_restricted" : false,
                "name" : "My fridge",
                "type" : "Computer",
                "volume_percent" : 100
            } ]
        }
"#;
    let payload: DevicePayload = serde_json::from_str(json_str).unwrap();
    assert_eq!(payload.devices[0]._type, DeviceType::Computer)
}
