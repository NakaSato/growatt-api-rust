use chrono::{Duration, Utc};
use std::env;
use crate::{Growatt, Plant, PlantList, PlantData};

#[test]
fn test_new_client() {
    let client = Growatt::new();
    assert_eq!(client.base_url, "https://server.growatt.com");
    assert_eq!(client.is_logged_in, false);
    assert!(client.username.is_none());
    assert!(client.password.is_none());
    assert!(client.session_expiry.is_none());
    assert!(client.token.is_none());
    // Check session duration is 30 minutes
    assert_eq!(client.session_duration, Duration::minutes(30));
}

#[test]
fn test_with_alternate_url() {
    let client = Growatt::new().with_alternate_url();
    assert_eq!(client.base_url, "https://openapi.growatt.com");
}

#[test]
fn test_with_session_duration() {
    let client = Growatt::new().with_session_duration(60);
    assert_eq!(client.session_duration, Duration::minutes(60));
}

#[test]
fn test_hash_password() {
    let client = Growatt::new();
    let password = "testpassword";
    let hash = client.hash_password(password);
    // MD5 of "testpassword" is "e16b2ab8d12314bf4efbd6203906ea6c"
    assert_eq!(hash, "e16b2ab8d12314bf4efbd6203906ea6c");
}

#[test]
fn test_is_session_valid() {
    let mut client = Growatt::new();
    
    // Session should be invalid by default
    assert!(!client.is_session_valid());
    
    // Set session to expire in the future
    client.session_expiry = Some(Utc::now() + Duration::hours(1));
    assert!(client.is_session_valid());
    
    // Set session to expire in the past
    client.session_expiry = Some(Utc::now() - Duration::hours(1));
    assert!(!client.is_session_valid());
}

#[test]
fn test_from_env() {
    // Backup existing env vars if any
    let original_username = env::var("GROWATT_USERNAME").ok();
    let original_password = env::var("GROWATT_PASSWORD").ok();
    let original_base_url = env::var("GROWATT_BASE_URL").ok();
    let original_duration = env::var("GROWATT_SESSION_DURATION").ok();
    
    // Set test environment variables
    env::set_var("GROWATT_USERNAME", "test_username");
    env::set_var("GROWATT_PASSWORD", "test_password");
    env::set_var("GROWATT_BASE_URL", "https://openapi.growatt.com");
    env::set_var("GROWATT_SESSION_DURATION", "45");
    
    // Create client from environment
    let client = Growatt::from_env();
    
    // Check values were correctly loaded
    assert_eq!(client.username, Some("test_username".to_string()));
    assert_eq!(client.password, Some("test_password".to_string()));
    assert_eq!(client.base_url, "https://openapi.growatt.com");
    assert_eq!(client.session_duration, Duration::minutes(45));
    
    // Restore original environment variables or remove test ones
    match original_username {
        Some(value) => env::set_var("GROWATT_USERNAME", value),
        None => env::remove_var("GROWATT_USERNAME"),
    }
    match original_password {
        Some(value) => env::set_var("GROWATT_PASSWORD", value),
        None => env::remove_var("GROWATT_PASSWORD"),
    }
    match original_base_url {
        Some(value) => env::set_var("GROWATT_BASE_URL", value),
        None => env::remove_var("GROWATT_BASE_URL"),
    }
    match original_duration {
        Some(value) => env::set_var("GROWATT_SESSION_DURATION", value),
        None => env::remove_var("GROWATT_SESSION_DURATION"),
    }
}

#[test]
fn test_plant_structs() {
    // Test plant struct serialization/deserialization
    let json_data = r#"{
        "id": "12345",
        "name": "Test Plant",
        "plantAddress": "123 Test St",
        "plantPower": 5000.0,
        "isShare": false
    }"#;
    
    let plant: Plant = serde_json::from_str(json_data).unwrap();
    
    assert_eq!(plant.plant_id, "12345");
    assert_eq!(plant.plant_name, "Test Plant");
    assert_eq!(plant.plant_address, Some("123 Test St".to_string()));
    assert_eq!(plant.plant_watts, Some(5000.0));
    assert_eq!(plant.is_share, Some(false));
}

#[test]
fn test_plant_list() {
    // Test PlantList wrapper
    let plant1 = Plant {
        plant_id: "1".to_string(),
        plant_name: "Plant 1".to_string(),
        plant_address: Some("Address 1".to_string()),
        plant_watts: Some(1000.0),
        is_share: Some(false),
    };
    
    let plant2 = Plant {
        plant_id: "2".to_string(),
        plant_name: "Plant 2".to_string(),
        plant_address: Some("Address 2".to_string()),
        plant_watts: Some(2000.0),
        is_share: Some(true),
    };
    
    let plant_list = PlantList(vec![plant1, plant2]);
    
    assert_eq!(plant_list.0.len(), 2);
    assert_eq!(plant_list.0[0].plant_id, "1");
    assert_eq!(plant_list.0[1].plant_id, "2");
}

#[test]
fn test_plant_data_struct() {
    // Test PlantData struct serialization/deserialization
    let json_data = r#"{
        "plantName": "Test Plant",
        "plantId": "12345",
        "capacity": 5000.0,
        "todayEnergy": 23.5,
        "totalEnergy": 1234.5,
        "currentPower": 4500.0
    }"#;
    
    let plant_data: PlantData = serde_json::from_str(json_data).unwrap();
    
    assert_eq!(plant_data.plant_name, Some("Test Plant".to_string()));
    assert_eq!(plant_data.plant_id, Some("12345".to_string()));
    assert_eq!(plant_data.capacity, Some(5000.0));
    assert_eq!(plant_data.today_energy, Some(23.5));
    assert_eq!(plant_data.total_energy, Some(1234.5));
    assert_eq!(plant_data.current_power, Some(4500.0));
}