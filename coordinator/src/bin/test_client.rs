use tokio_tungstenite::connect_async;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:3000/ws";
    println!("Connecting to {}", url);
    
    match connect_async(url).await {
        Ok((ws_stream, _)) => {
            println!("Connected to WebSocket");
            let (_, mut read) = ws_stream.split();
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(m) => println!("Received: {}", m),
                    Err(e) => println!("Error: {}", e),
                }
            }
        }
        Err(e) => println!("Connection failed: {}", e),
    }
}
