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
growatt-api-rust = "0.1.0"
```

## Usage

```rust
use growatt_api_rust::Growatt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new client
    let mut client = Growatt::new();

    // Login
    if client.login("your_username", "your_password").await? {
        println!("Login successful!");

        // Get plants
        let plants = client.get_plants().await?;
        println!("Plants: {}", plants);

        // When you're done
        client.logout().await?;
    }

    Ok(())
}
```

## API Reference

The library provides the following methods:

- `login(username, password)` - Authenticate with Growatt servers
- `logout()` - End your session
- `get_plants()` - List all plants associated with the account
- `get_plant(plant_id)` - Get detailed information about a specific plant
- `get_mix_ids(plant_id)` - Get the MIX device IDs for a plant
- `get_mix_total(plant_id, mix_sn)` - Get total measurements from a specific MIX
- `get_mix_status(plant_id, mix_sn)` - Get current status of a MIX device
- `get_energy_stats_daily(date, plant_id, mix_sn)` - Get daily energy statistics
- `get_energy_stats_monthly(date, plant_id, mix_sn)` - Get monthly energy statistics
- `get_energy_stats_yearly(year, plant_id, mix_sn)` - Get yearly energy statistics
- `get_energy_stats_total(year, plant_id, mix_sn)` - Get total energy statistics
- `get_weekly_battery_stats(plant_id, mix_sn)` - Get weekly battery statistics
- `get_device_list(plant_id)` - Get list of devices for a plant
- `get_weather(plant_id)` - Get weather information for a plant
- `get_fault_logs(plant_id, date, device_sn, page_num, device_flag, fault_type)` - Get fault logs

## License

MIT
