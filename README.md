# Growatt API Rust

A Rust client for interacting with the Growatt API. This library allows you to:

- Login and authenticate with the Growatt server
- Retrieve plant and device information
- Get energy statistics (daily, monthly, yearly, total)
- Access mix status and battery statistics
- Retrieve fault logs and other device information

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
growatt = "0.1.0"
```

## Quick Start

```rust
use growatt::Growatt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new client
    let mut client = Growatt::new();

    // Login
    if client.login("your_username", "your_password").await? {
        println!("Login successful!");

        // Get plants
        let plants = client.get_plants().await?;
        println!("Plants: {:?}", plants);

        // When you're done
        client.logout().await?;
    }

    Ok(())
}
```

## Environment Variables Configuration

You can initialize the client with environment variables for easier configuration:

```rust
use growatt_api_rust::Growatt;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load from .env file and environment
    dotenv().ok();

    // Create client with env vars (GROWATT_USERNAME, GROWATT_PASSWORD, etc.)
    let mut client = Growatt::from_env();

    // Auto-login using credentials from environment
    if client.check_login().await? {
        // Your code here
    }

    Ok(())
}
```

Expected environment variables:

- `GROWATT_USERNAME`: Your Growatt account username
- `GROWATT_PASSWORD`: Your Growatt account password
- `GROWATT_BASE_URL` (optional): Alternative base URL
- `GROWATT_SESSION_DURATION` (optional): Session duration in minutes

## Client Initialization Options

### Standard Initialization

```rust
// Create with default options
let client = Growatt::new();
```

### Alternative Server URL

```rust
// Use an alternative API server
let client = Growatt::new().with_alternate_url();
```

### Custom Session Duration

```rust
// Set a custom session duration (60 minutes)
let client = Growatt::new().with_session_duration(60);
```

## Authentication

### Login

```rust
// Login with username and password
let success = client.login("username", "password").await?;
```

### Logout

```rust
// Properly terminate the session
let success = client.logout().await?;
```

### Authentication Status

```rust
// Check if currently logged in
if client.is_logged_in() {
    // User is authenticated
}
```

## Core API Methods

### Plant Management

```rust
// Get all plants for the account
let plants = client.get_plants().await?;

// Get detailed information about a specific plant
let plant_details = client.get_plant("plant_id").await?;

// Get weather information for a plant
let weather = client.get_weather("plant_id").await?;
```

### Device Management

```rust
// Get list of MIX device IDs for a plant
let mix_ids = client.get_mix_ids("plant_id").await?;

// Get detailed device list for a plant
let devices = client.get_device_list("plant_id").await?;

// Get devices with pagination
let devices_page = client.get_devices_by_plant_list("plant_id", Some(1)).await?;
```

### Mix Device Data

```rust
// Get total measurements from a specific MIX
let mix_total = client.get_mix_total("plant_id", "mix_sn").await?;

// Get current status of a MIX device
let mix_status = client.get_mix_status("plant_id", "mix_sn").await?;

// Update MIX AC discharge time period
let result = client.post_mix_ac_discharge_time_period_now("plant_id", "mix_sn").await?;
```

### Energy Statistics

```rust
// Get daily energy statistics
let daily_stats = client.get_energy_stats_daily("2025-04-26", "plant_id", "mix_sn").await?;

// Get monthly energy statistics
let monthly_stats = client.get_energy_stats_monthly("2025-04", "plant_id", "mix_sn").await?;

// Get yearly energy statistics
let yearly_stats = client.get_energy_stats_yearly("2025", "plant_id", "mix_sn").await?;

// Get total energy statistics
let total_stats = client.get_energy_stats_total("2025", "plant_id", "mix_sn").await?;
```

### Battery Statistics

```rust
// Get weekly battery statistics
let battery_stats = client.get_weekly_battery_stats("plant_id", "mix_sn").await?;
```

### Fault Logs

```rust
// Get fault logs with detailed parameters
let fault_logs = client.get_fault_logs(
    "plant_id",      // Plant ID
    Some("2025-04-26"),  // Date (optional)
    "device_sn",     // Device serial number
    1,               // Page number
    0,               // Device flag (0 = All)
    0                // Fault type (0 = All)
).await?;

// Using the alias method (identical functionality)
let fault_logs = client.get_plant_fault_logs(
    "plant_id", Some("2025-04-26"), "device_sn", 1, 0, 0
).await?;
```

## Error Handling

The library uses a custom error type `GrowattError` which covers various error scenarios:

```rust
// Example of error handling
match client.login("username", "password").await {
    Ok(success) => {
        if success {
            println!("Login successful");
        } else {
            println!("Login failed");
        }
    },
    Err(err) => match err {
        GrowattError::AuthError(msg) => println!("Authentication error: {}", msg),
        GrowattError::RequestError(err) => println!("Network error: {}", err),
        GrowattError::JsonError(err) => println!("JSON parsing error: {}", err),
        GrowattError::InvalidResponse(msg) => println!("Invalid API response: {}", msg),
        GrowattError::NotLoggedIn => println!("Not logged in"),
    }
}
```

## Data Structures

The library provides structured access to Growatt data:

```rust
// Example of working with plant data
let plants = client.get_plants().await?;
for plant in plants.0 {
    println!("Plant ID: {}", plant.plant_id);
    println!("Plant Name: {}", plant.plant_name);

    if let Some(address) = plant.plant_address {
        println!("Address: {}", address);
    }

    if let Some(power) = plant.plant_watts {
        println!("Power: {} W", power);
    }
}

// Example of working with plant details
let plant_details = client.get_plant("plant_id").await?;
if let Some(total_energy) = plant_details.total_energy {
    println!("Total Energy: {} kWh", total_energy);
}
```

## Advanced Usage

### Session Management

The library automatically handles session expiry and renewal:

```rust
// The client will automatically re-authenticate if needed
client.get_plants().await?;
```

### Token Access

```rust
// Get the authentication token (if needed for external use)
if let Some(token) = client.get_token() {
    println!("Current token: {}", token);
}
```

## License

MIT
