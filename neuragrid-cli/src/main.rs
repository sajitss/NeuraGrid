use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use url::Url;

#[derive(Parser)]
#[command(name = "neuragrid-cli")]
#[command(about = "CLI for NeuraGrid", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit a job to the grid
    Submit {
        /// Job type (e.g., string_search, prime_search)
        #[arg(long)]
        r#type: String,

        /// Arguments for the job
        #[arg(long, num_args = 1..)]
        args: Vec<String>,

        /// Wait for job completion
        #[arg(long, default_value_t = false)]
        wait: bool,

        /// Target specific worker
        #[arg(long)]
        target: Option<String>,

        /// Tags for the job
        #[arg(long)]
        tags: Vec<String>,
    },
    /// Listen for jobs with a specific tag
    Listen {
        /// Tag to listen for (e.g., #urgent)
        #[arg(long)]
        tag: String,
    },
}

#[derive(Deserialize, Debug)]
struct JobUpdatePayload {
    id: String,
    status: String,
    message: String,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct JobUpdateMessage {
    #[serde(rename = "type")]
    msg_type: String,
    payload: JobUpdatePayload,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Listen { tag } => {
            println!("Listening for jobs with tag: {}", tag);

            // 1. Fetch initial queue state to see if there are pending jobs
            let client = reqwest::Client::new();
            let queue_res = client.get("http://localhost:3000/api/queue").send().await;
            
            let mut pending_count = 0;

            if let Ok(res) = queue_res {
                if let Ok(queue_data) = res.json::<std::collections::HashMap<String, i64>>().await {
                    pending_count = *queue_data.get(&tag).unwrap_or(&0);
                    println!("Found {} pending jobs for tag '{}'", pending_count, tag);
                }
            }

            if pending_count == 0 {
                println!("No pending jobs found for tag '{}'. Exiting.", tag);
                return;
            }

            // 2. Connect to WebSocket
            let url = Url::parse("ws://localhost:3000/ws?name=CLI-Listener").unwrap();
            let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect to WebSocket");
            println!("Connected to event stream. Waiting for updates...");

            while let Some(msg) = ws_stream.next().await {
                if let Ok(Message::Text(text)) = msg {
                    if let Ok(update) = serde_json::from_str::<JobUpdateMessage>(&text) {
                        if update.msg_type == "job_update" {
                             if let Some(tags) = &update.payload.tags {
                                if tags.contains(&tag) {
                                    println!("[{}] {}", update.payload.status.to_uppercase(), update.payload.message);

                                    if update.payload.status == "completed" || update.payload.status == "failed" {
                                        pending_count -= 1;
                                        println!("Remaining jobs for '{}': {}", tag, pending_count);
                                    } else if update.payload.status == "pending" {
                                        // New job submitted with this tag
                                        pending_count += 1;
                                        println!("New job added. Pending: {}", pending_count);
                                    }

                                    if pending_count <= 0 {
                                        println!("All jobs for tag '{}' finished. Exiting.", tag);
                                        process::exit(0);
                                    }
                                }
                             }
                        }
                    }
                }
            }
        }
        Commands::Submit { r#type, args, wait, target, tags } => {
            let client = reqwest::Client::new();
            
            let mut job_body = json!({
                "job_type": r#type,
                "args": args,
                "tags": tags
            });

            if let Some(t) = target {
                job_body["target"] = json!(t);
            }

            println!("Submitting job...");
            
            // If waiting, connect to WS FIRST to avoid race condition
            let mut ws_stream = None;
            if wait {
                let url = Url::parse("ws://localhost:3000/ws?name=CLI-Client").unwrap();
                let (ws, _) = connect_async(url).await.expect("Failed to connect to WebSocket");
                ws_stream = Some(ws);
                println!("Connected to event stream.");
            }

            let res = client.post("http://localhost:3000/job")
                .json(&job_body)
                .send()
                .await;

            match res {
                Ok(response) => {
                    let text = response.text().await.unwrap_or_default();
                    println!("Response: {}", text);
                    
                    if wait {
                        // Extract Job ID from response "Job <ID> queued"
                        let job_id = text.split_whitespace().nth(1).unwrap_or("");
                        if job_id.is_empty() {
                            eprintln!("Failed to parse Job ID from response.");
                            return;
                        }
                        println!("Waiting for Job {} to complete...", job_id);

                        if let Some(mut ws) = ws_stream {
                            while let Some(msg) = ws.next().await {
                                if let Ok(Message::Text(text)) = msg {
                                    if let Ok(update) = serde_json::from_str::<JobUpdateMessage>(&text) {
                                        if update.msg_type == "job_update" && update.payload.id == job_id {
                                            println!("[{}] {}", update.payload.status.to_uppercase(), update.payload.message);
                                            
                                            if update.payload.status == "completed" {
                                                println!("Job finished successfully.");
                                                process::exit(0);
                                            } else if update.payload.status == "failed" {
                                                eprintln!("Job failed.");
                                                process::exit(1);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to submit job: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
