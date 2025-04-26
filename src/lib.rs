use chrono::prelude::*;
use md5::{Digest, Md5};
use reqwest::{Client, cookie::Jar};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use std::env;
use dotenv::dotenv;

// Include test modules
#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum GrowattError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Not logged in")]
    NotLoggedIn,
}

pub type Result<T> = std::result::Result<T, GrowattError>;

// Define structs for plant data
#[derive(Debug, Serialize, Deserialize)]
pub struct Plant {
    #[serde(rename = "id")]
    pub plant_id: String,
    #[serde(rename = "name", alias = "plantName")]
    pub plant_name: String,
    #[serde(rename = "plantAddress", default)]
    pub plant_address: Option<String>,
    #[serde(rename = "plantPower", default)]
    pub plant_watts: Option<f64>,
    #[serde(rename = "isShare", default)]
    pub is_share: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlantList(pub Vec<Plant>);

#[derive(Debug, Serialize, Deserialize)]
pub struct PlantData {
    #[serde(rename = "plantName")]
    pub plant_name: Option<String>,
    #[serde(rename = "plantId")]
    pub plant_id: Option<String>,
    #[serde(rename = "capacity")]
    pub capacity: Option<f64>,
    #[serde(rename = "todayEnergy")]
    pub today_energy: Option<f64>,
    #[serde(rename = "totalEnergy")]
    pub total_energy: Option<f64>,
    #[serde(rename = "currentPower")]
    pub current_power: Option<f64>,
    // Add more fields as needed based on the actual API response
}

pub struct Growatt {
    base_url: String,
    client: Client,
    username: Option<String>,
    password: Option<String>,
    is_logged_in: bool,
    session_expiry: Option<DateTime<Utc>>,
    session_duration: chrono::Duration,
    token: Option<String>,  // Add token field
}

impl Growatt {
    pub fn new() -> Self {
        // Create a client with cookie storage
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(Arc::clone(&jar))
            .build()
            .unwrap();

        Self {
            base_url: "https://server.growatt.com".to_string(),
            client,
            username: None,
            password: None,
            is_logged_in: false,
            session_expiry: None,
            // Default session duration of 30 minutes
            session_duration: chrono::Duration::minutes(30),
            token: None,  // Initialize token as None
        }
    }
    
    /// Creates a new Growatt client with configuration from environment variables.
    /// 
    /// This method attempts to load the following environment variables:
    /// - GROWATT_USERNAME: The username for Growatt API
    /// - GROWATT_PASSWORD: The password for Growatt API
    /// - GROWATT_BASE_URL: (Optional) Alternative base URL (defaults to standard URL if not set)
    /// - GROWATT_SESSION_DURATION: (Optional) Session duration in minutes (defaults to 30 minutes if not set)
    /// 
    /// You can set these variables in a `.env` file in the project directory.
    pub fn from_env() -> Self {
        // Load .env file if it exists
        dotenv().ok();
        
        let mut client = Self::new();
        
        // Set username and password if available in environment
        if let Ok(username) = env::var("GROWATT_USERNAME") {
            client.username = Some(username);
        }
        
        if let Ok(password) = env::var("GROWATT_PASSWORD") {
            client.password = Some(password);
        }
        
        // Set base URL if specified
        if let Ok(base_url) = env::var("GROWATT_BASE_URL") {
            client.base_url = base_url;
        }
        
        // Set session duration if specified
        if let Ok(duration_str) = env::var("GROWATT_SESSION_DURATION") {
            if let Ok(duration) = duration_str.parse::<i64>() {
                client.session_duration = chrono::Duration::minutes(duration);
            }
        }
        
        client
    }

    pub fn with_alternate_url(mut self) -> Self {
        self.base_url = "https://openapi.growatt.com".to_string();
        self
    }

    pub fn with_session_duration(mut self, minutes: i64) -> Self {
        self.session_duration = chrono::Duration::minutes(minutes);
        self
    }

    fn hash_password(&self, password: &str) -> String {
        let mut hasher = Md5::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<bool> {
        // If already logged in with a valid session, return early
        if self.is_logged_in && self.is_session_valid() {
            return Ok(true);
        }

        self.username = Some(username.to_string());
        self.password = Some(password.to_string());

        let password_hash = self.hash_password(password);

        let form = [
            ("account", username),
            ("password", ""),
            ("validateCode", ""),
            ("isReadPact", "1"),
            ("passwordCrc", &password_hash),
        ];

        let response = self.client
            .post(format!("{}/login", self.base_url))
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .form(&form)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        response.error_for_status_ref()?;

        let json_response: serde_json::Value = response.json().await?;

        println!("Login response: {}", json_response);

        if let Some(result) = json_response.get("result").and_then(|v| v.as_i64()) {
            if result == 1 {
                self.is_logged_in = true;
                // Set session expiry time
                self.session_expiry = Some(Utc::now() + self.session_duration);
                
                // Extract and store token if available in the response
                if let Some(token) = json_response.get("token").and_then(|v| v.as_str()) {
                    self.token = Some(token.to_string());
                }
                
                Ok(true)
            } else {
                let error_msg = json_response
                    .get("msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                
                println!("Login failed with error: {}", error_msg);
                self.is_logged_in = false;
                self.session_expiry = None;
                Err(GrowattError::AuthError(error_msg))
            }
        } else {
            self.is_logged_in = false;
            self.session_expiry = None;
            Err(GrowattError::InvalidResponse(
                "Invalid response structure".to_string(),
            ))
        }
    }

    // Check if the current session is valid
    fn is_session_valid(&self) -> bool {
        if let Some(expiry) = self.session_expiry {
            Utc::now() < expiry
        } else {
            false
        }
    }

    // Ensure a valid session exists, auto-login if needed
    async fn ensure_session(&mut self) -> Result<()> {
        if !self.is_logged_in || !self.is_session_valid() {
            if let (Some(username), Some(password)) = (self.username.clone(), self.password.clone()) {
                self.login(&username, &password).await?;
            } else {
                return Err(GrowattError::NotLoggedIn);
            }
        }
        Ok(())
    }

    pub async fn logout(&mut self) -> Result<bool> {
        if !self.is_logged_in {
            println!("No active session to log out from.");
            return Ok(false);
        }

        // Create request with all headers in a more concise way
        let response = self.client
            .get(format!("{}/logout", self.base_url))
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Upgrade-Insecure-Requests", "1")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .header("Sec-Fetch-Site", "same-origin")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-User", "?1")
            .header("Sec-Fetch-Dest", "document")
            .header("Referer", format!("{}/index", self.base_url))
            .send()
            .await?;

        // Growatt returns 302 redirect on successful logout
        let status = response.status().as_u16();
        let success = status == 302;
        
        // Update the session state based on the result
        if success {
            self.is_logged_in = false;
            self.session_expiry = None;
            println!("Successfully logged out.");
        } else {
            println!("Logout returned unexpected status code: {}", status);
        }
        
        Ok(success)
    }

    // Helper method to check if user is logged in, with auto-reconnect
    async fn check_login(&mut self) -> Result<()> {
        self.ensure_session().await
    }

    pub async fn get_plants(&mut self) -> Result<PlantList> {
        self.check_login().await?;

        let response = self.client
            .post(format!("{}/index/getPlantListTitle", self.base_url))
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.as_array().map_or(true, |arr| arr.is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            let plants: Vec<Plant> = serde_json::from_value(json_response)?;
            Ok(PlantList(plants))
        }
    }

    pub async fn get_plant(&mut self, plant_id: &str) -> Result<PlantData> {
        self.check_login().await?;

        let response = self.client
            .post(format!("{}/panel/getPlantData?plantId={}", self.base_url, plant_id))
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if let Some(obj) = json_response.get("obj") {
            if obj.is_null() || (obj.is_object() && obj.as_object().unwrap().is_empty()) {
                Err(GrowattError::InvalidResponse(
                    "Empty response. Please ensure you are logged in.".to_string(),
                ))
            } else {
                let plant_data: PlantData = serde_json::from_value(obj.clone())?;
                Ok(plant_data)
            }
        } else {
            Err(GrowattError::InvalidResponse(
                "Invalid response structure".to_string(),
            ))
        }
    }

    pub async fn get_mix_ids(&mut self, plant_id: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let response = self.client
            .post(format!("{}/panel/getDevicesByPlant?plantId={}", self.base_url, plant_id))
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if let Some(obj) = json_response.get("obj").and_then(|o| o.get("mix")) {
            if obj.is_null() || (obj.is_array() && obj.as_array().unwrap().is_empty()) {
                Err(GrowattError::InvalidResponse(
                    "Empty response. Please ensure you are logged in.".to_string(),
                ))
            } else {
                Ok(obj.clone())
            }
        } else {
            Err(GrowattError::InvalidResponse(
                "Invalid response structure".to_string(),
            ))
        }
    }

    pub async fn get_mix_total(&mut self, plant_id: &str, mix_sn: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [("mixSn", mix_sn)];

        let response = self.client
            .post(format!("{}/panel/mix/getMIXTotalData?plantId={}", self.base_url, plant_id))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if let Some(obj) = json_response.get("obj") {
            if obj.is_null() || (obj.is_object() && obj.as_object().unwrap().is_empty()) {
                Err(GrowattError::InvalidResponse(
                    "Empty response. Please ensure you are logged in.".to_string(),
                ))
            } else {
                Ok(obj.clone())
            }
        } else {
            Err(GrowattError::InvalidResponse(
                "Invalid response structure".to_string(),
            ))
        }
    }

    pub async fn get_mix_status(&mut self, plant_id: &str, mix_sn: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [("mixSn", mix_sn)];

        let response = self.client
            .post(format!("{}/panel/mix/getMIXStatusData?plantId={}", self.base_url, plant_id))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if let Some(obj) = json_response.get("obj") {
            if obj.is_null() || (obj.is_object() && obj.as_object().unwrap().is_empty()) {
                Err(GrowattError::InvalidResponse(
                    "Empty response. Please ensure you are logged in.".to_string(),
                ))
            } else {
                Ok(obj.clone())
            }
        } else {
            Err(GrowattError::InvalidResponse(
                "Invalid response structure".to_string(),
            ))
        }
    }

    pub async fn get_energy_stats_daily(&mut self, date: &str, plant_id: &str, mix_sn: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [
            ("date", date),
            ("plantId", plant_id),
            ("mixSn", mix_sn),
        ];

        let response = self.client
            .post(format!("{}/panel/mix/getMIXEnergyDayChart", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn get_energy_stats_monthly(&mut self, date: &str, plant_id: &str, mix_sn: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [
            ("date", date),
            ("plantId", plant_id),
            ("mixSn", mix_sn),
        ];

        let response = self.client
            .post(format!("{}/panel/mix/getMIXEnergyMonthChart", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn get_energy_stats_yearly(&mut self, year: &str, plant_id: &str, mix_sn: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [
            ("year", year),
            ("plantId", plant_id),
            ("mixSn", mix_sn),
        ];

        let response = self.client
            .post(format!("{}/panel/mix/getMIXEnergyYearChart", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn get_energy_stats_total(&mut self, year: &str, plant_id: &str, mix_sn: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [
            ("year", year),
            ("plantId", plant_id),
            ("mixSn", mix_sn),
        ];

        let response = self.client
            .post(format!("{}/panel/mix/getMIXEnergyTotalChart", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn get_weekly_battery_stats(&mut self, plant_id: &str, mix_sn: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [
            ("plantId", plant_id),
            ("mixSn", mix_sn),
        ];

        let response = self.client
            .post(format!("{}/panel/mix/getMIXBatChart", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn post_mix_ac_discharge_time_period_now(&mut self, _plant_id: &str, mix_sn: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let now = Local::now();
        let param1 = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let form = [
            ("action", "mixSet"),
            ("serialNum", mix_sn),
            ("type", "pf_sys_year"),
            ("param1", &param1),
        ];

        let response = self.client
            .post(format!("{}/tcpSet.do", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn get_device_list(&mut self, plant_id: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [
            ("plantId", plant_id),
            ("currPage", "1"),
        ];

        let response = self.client
            .post(format!("{}/device/getMAXList", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn get_weather(&mut self, plant_id: &str) -> Result<serde_json::Value> {
        self.check_login().await?;

        let form = [
            ("plantId", plant_id),
            ("currPage", "1"),
        ];

        let response = self.client
            .post(format!("{}/device/getEnvList", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn get_devices_by_plant_list(&mut self, plant_id: &str, curr_page: Option<i32>) -> Result<serde_json::Value> {
        self.check_login().await?;

        let curr_page = curr_page.unwrap_or(1).to_string();

        let form = [
            ("plantId", plant_id),
            ("currPage", &curr_page),
        ];

        let response = self.client
            .post(format!("{}/panel/getDevicesByPlantList", self.base_url))
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse(
                "Empty response. Please ensure you are logged in.".to_string(),
            ))
        } else {
            Ok(json_response)
        }
    }

    pub async fn get_fault_logs(
        &mut self, 
        plant_id: &str, 
        date: Option<&str>, 
        device_sn: &str, 
        page_num: i32, 
        device_flag: i32, 
        fault_type: i32
    ) -> Result<serde_json::Value> {
        self.check_login().await?;

        // Use current date if none provided
        let date = match date {
            Some(d) => d.to_string(),
            None => Local::now().format("%Y-%m-%d").to_string(),
        };

        // Validate inputs
        if plant_id.is_empty() {
            return Err(GrowattError::InvalidResponse("Plant ID must be provided".to_string()));
        }

        let form = [
            ("deviceSn", device_sn),
            ("date", &date),
            ("plantId", plant_id),
            ("toPageNum", &page_num.to_string()),
            ("type", &fault_type.to_string()),
            ("deviceFlag", &device_flag.to_string()),
        ];

        let response = self.client
            .post(format!("{}/log/getNewPlantFaultLog", self.base_url))
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .form(&form)
            .send()
            .await?;

        response.error_for_status_ref()?;
        
        let json_response: serde_json::Value = response.json().await?;
        
        if json_response.is_null() || (json_response.is_object() && json_response.as_object().unwrap().is_empty()) {
            Err(GrowattError::InvalidResponse("Empty response received from server".to_string()))
        } else {
            Ok(json_response)
        }
    }

    // Alias for backward compatibility
    pub async fn get_plant_fault_logs(
        &mut self, 
        plant_id: &str, 
        date: Option<&str>, 
        device_sn: &str, 
        page_num: i32, 
        device_flag: i32, 
        fault_type: i32
    ) -> Result<serde_json::Value> {
        self.get_fault_logs(plant_id, date, device_sn, page_num, device_flag, fault_type).await
    }

    // Add a public method to check login status
    pub fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }
}

impl Default for Growatt {
    fn default() -> Self {
        Self::new()
    }
}
