// by ideikiPIX
use serde::{Deserialize};
use std::{fs::File, io::Write};
use reqwest::{Client, header::AUTHORIZATION};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct TunnelListResource {
    tunnels: Vec<Tunnel>,
}

#[derive(Debug, Deserialize)]
struct Tunnel {
    #[serde(rename = "public_url")]
    public_url: String,
}

#[derive(Debug, Deserialize)]
struct ServerStatus {
    players: Players,
}

#[derive(Debug, Deserialize)]
struct Players {
    online: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ngrok API URL and Auth Token
    let ngrok_api_url = "https://api.ngrok.com/tunnels";
    let ngrok_api_key = ""; // Replace with your full ngrok API key

    // Base URL for Minecraft server status request, {} will be replaced with PublicURL
    let mc_status_base_url = "https://api.mcstatus.io/v2/status/java/{}";

    // Infinite loop for periodic checking and writing to a file
    loop {
        // Create HTTP client with Authorization header
        let client = Client::new();
        let ngrok_response = client
            .get(ngrok_api_url)
            .header(AUTHORIZATION, format!("Bearer {}", ngrok_api_key))
            .header("Ngrok-Version", "2") // Ensure the correct ngrok API version is specified
            .send()
            .await?;

        if !ngrok_response.status().is_success() {
            println!("Failed to retrieve ngrok tunnel information: {}", ngrok_response.status());
            println!("Response body: {}", ngrok_response.text().await.unwrap());
            continue;
        }

        // Deserialize JSON response into TunnelListResource struct
        let ngrok_json = ngrok_response.text().await?;
        let ngrok_tunnel: TunnelListResource = serde_json::from_str(&ngrok_json)?;

        // Get the TCP public URL from ngrok API
        let public_url = ngrok_tunnel.tunnels.iter()
            .find(|tunnel| tunnel.public_url.starts_with("tcp://"))
            .map(|tunnel| tunnel.public_url.clone())
            .unwrap_or_else(|| {
                println!("No TCP tunnels found");
                return "".to_string();
            });

        if public_url.is_empty() {
            continue;
        }

        println!("ngrok Public URL: {}", public_url);

        // Replace the tcp:// with an appropriate format for the Minecraft server status API
        let formatted_url = public_url.replace("tcp://", "");

        // Form the full URL for the Minecraft server status request
        let mc_status_url = mc_status_base_url.replace("{}", &formatted_url);

        println!("Minecraft server status URL: {}", mc_status_url);

        // Send GET request to Minecraft server API
        let mc_status_response = client.get(&mc_status_url).send().await?;

        // Check if the request was successful
        if mc_status_response.status().is_success() {
            // Deserialize JSON response into ServerStatus struct
            let server_status: ServerStatus = mc_status_response.json().await?;

            // Extract the number of online players
            let online_players = server_status.players.online;

            // Form a JSON object to write to the file
            let json_data = serde_json::to_string(&online_players)?;

            // Write data to a file
            let mut file = File::create("online_players.json")?;
            file.write_all(json_data.as_bytes())?;
            println!("Online players saved: {}", online_players);
        } else {
            println!("Failed to retrieve Minecraft server status: {}", mc_status_response.status());
            println!("Response body: {}", mc_status_response.text().await.unwrap());
        }

        // Wait for 1 minute before the next iteration
        sleep(Duration::from_secs(60)).await;
    }
}
