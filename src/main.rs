use growatt_api_rust::Growatt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Growatt API client
    let mut client = Growatt::new();
    
    // Example usage - replace with your actual credentials
    let username = "enwufttest";
    let password = "enwuft1234";
    
    // Login to the Growatt service
    match client.login(username, password).await {
        Ok(true) => {
            println!("Login successful!");
            
            // Example: Get plants
            match client.get_plants().await {
                Ok(plants) => {
                    println!("Plants: {}", plants);
                    
                    // Further API calls can be made here
                },
                Err(e) => println!("Error getting plants: {}", e),
            }
            
            // Logout when done
            if let Err(e) = client.logout().await {
                println!("Error during logout: {}", e);
            }
        },
        Ok(false) => println!("Login failed!"),
        Err(e) => println!("Error during login: {}", e),
    }
    
    Ok(())
}
