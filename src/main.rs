use growatt::Growatt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Growatt client
    let mut client = Growatt::new();
    
    // Replace these with your actual credentials
    let username = "enwufttest";
    let password = "enwuft1234";
    
    // Login to the Growatt API
    match client.login(username, password).await {
        Ok(true) => {
            println!("Login successful!");
            
            // Get the list of plants
            match client.get_plants().await {
                Ok(plants) => {
                    println!("Found {} plants:", plants.0.len());
                    
                    // Display information about each plant
                    for plant in plants.0 {
                        println!("Plant ID: {}", plant.plant_id);
                        println!("Plant Name: {}", plant.plant_name);
                        
                        if let Some(address) = plant.plant_address {
                            println!("Address: {}", address);
                        }
                        
                        if let Some(power) = plant.plant_watts {
                            println!("Power (W): {}", power);
                        }
                        
                        println!("-------------------");
                        
                        // Optional: Get more detailed data for each plant
                        match client.get_plant(&plant.plant_id).await {
                            Ok(plant_data) => {
                                println!("Additional plant data:");
                                if let Some(capacity) = plant_data.capacity {
                                    println!("Capacity: {}", capacity);
                                }
                                if let Some(today_energy) = plant_data.today_energy {
                                    println!("Today's Energy: {}", today_energy);
                                }
                                if let Some(total_energy) = plant_data.total_energy {
                                    println!("Total Energy: {}", total_energy);
                                }
                                println!("-------------------");
                            },
                            Err(e) => println!("Error getting detailed plant data: {}", e),
                        }
                    }
                },
                Err(e) => println!("Error getting plants: {}", e),
            }
            
            // Logout when done
            if let Err(e) = client.logout().await {
                println!("Error during logout: {}", e);
            } else {
                println!("Successfully logged out");
            }
        },
        Ok(false) => println!("Login failed! Check your credentials."),
        Err(e) => println!("Error during login: {}", e),
    }
    
    Ok(())
}
