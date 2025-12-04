use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, Query},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

mod db;
use db::init_db;
use sqlx::{Pool, Sqlite, Row};
use uuid;

#[derive(Clone, Debug, PartialEq)]
enum WorkerStatus {
    Idle,
    Busy,
}

struct WorkerClient {
    sender: mpsc::Sender<Message>,
    name: String,
    status: WorkerStatus,
}

#[derive(Clone)]
struct AppState {
    // Store clients as a list of WorkerClient
    clients: Arc<Mutex<Vec<WorkerClient>>>,
    pool: Pool<Sqlite>,
}

#[derive(Serialize)]
struct Stats {
    activeWorkers: usize,
    totalTflops: f64,
    jobsCompleted: usize,
}

#[derive(Serialize)]
struct WorkerInfo {
    id: String,
    hostname: String,
    ip: String,
    gpu: String,
    status: String,
    task: String,
}

#[derive(Deserialize)]
struct JobCompletion {
    job_id: String,
    status: String,
}

#[derive(Serialize)]
struct JobPayload {
    id: String,
    timestamp: u64,
    message: String,
    status: String,
}

#[derive(Serialize)]
struct JobUpdateMessage {
    #[serde(rename = "type")]
    msg_type: String,
    payload: JobPayload,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize Database
    let pool = init_db().await.expect("Failed to initialize database");
    println!("Database initialized successfully");

    let clients = Arc::new(Mutex::new(Vec::new()));
    let app_state = AppState { clients, pool };

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/job", post(submit_job))
        .route("/api/stats", get(get_stats))
        .route("/api/workers", get(get_workers))
        .route("/api/queue", get(get_queue))
        .nest_service("/", ServeDir::new("../coordinator-dashboard/dist"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Coordinator listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn get_stats(State(state): State<AppState>) -> Json<Stats> {
    let worker_count = state.clients.lock().unwrap().len();
    // Mocking TFLOPS and Jobs for demo purposes until we track them
    Json(Stats {
        activeWorkers: worker_count,
        totalTflops: (worker_count as f64) * 45.5, 
        jobsCompleted: 14203 + (worker_count * 12),
    })
}

async fn get_workers(State(state): State<AppState>) -> Json<Vec<WorkerInfo>> {
    let mut workers = Vec::new();
    
    // Generate info for connected workers
    for (i, client) in state.clients.lock().unwrap().iter().enumerate() {
        workers.push(WorkerInfo {
            id: format!("w{}", i),
            hostname: client.name.clone(),
            ip: format!("192.168.1.1{:02}", i),
            gpu: if i % 2 == 0 { "RTX 4090".to_string() } else { "A100".to_string() },
            status: format!("{:?}", client.status).to_lowercase(),
            task: if client.status == WorkerStatus::Busy { "Processing".to_string() } else { "-".to_string() },
        });
    }
    
    Json(workers)
}

async fn get_queue(State(state): State<AppState>) -> Json<HashMap<String, i64>> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT tags FROM jobs WHERE status = 'pending' AND tags IS NOT NULL")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let mut tag_counts = HashMap::new();

    for (tags_json,) in rows {
        if let Ok(tags) = serde_json::from_str::<Vec<String>>(&tags_json) {
            for tag in tags {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }
    }

    Json(tag_counts)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let name = params.get("name").cloned().unwrap_or_else(|| "Unknown".to_string());
    ws.on_upgrade(move |socket| handle_socket(socket, state, name))
}

async fn handle_socket(socket: WebSocket, state: AppState, name: String) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel(100);

    {
        let mut clients = state.clients.lock().unwrap();
        clients.push(WorkerClient {
            sender: tx,
            name: name.clone(),
            status: WorkerStatus::Idle,
        });
    }
    
    println!("New client connected: {}", name);
    
    // Try dispatching jobs immediately as a new worker joined
    try_dispatch_jobs(state.clone()).await;

    // Spawn a task to forward messages from the channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task to read messages from the WebSocket
    let state_clone = state.clone();
    let name_clone = name.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Received from client {}: {}", name_clone, text);
                
                // Simple heuristic: if message contains "completed" or "finished", mark idle.
                // In a real system, we'd parse a proper JSON status message.
                if text.to_lowercase().contains("completed") || text.to_lowercase().contains("finished") {
                     {
                        let mut clients = state_clone.clients.lock().unwrap();
                        if let Some(client) = clients.iter_mut().find(|c| c.name == name_clone) {
                            client.status = WorkerStatus::Idle;
                            println!("Worker {} is now Idle", name_clone);
                        }
                    }
                    try_dispatch_jobs(state_clone.clone()).await;
                }
            }
        }
    });

    // Send welcome message
    let welcome_tx = {
        let guard = state.clients.lock().unwrap();
        guard.last().map(|c| c.sender.clone())
    };

    if let Some(tx) = welcome_tx {
        let _ = tx.send(Message::Text("Welcome to NeuraGrid".to_string())).await;
    }


    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {},
        _ = (&mut recv_task) => {},
    }

    // Remove client on disconnect
    {
        let mut clients = state.clients.lock().unwrap();
        if let Some(pos) = clients.iter().position(|c| c.name == name) {
            clients.remove(pos);
        }
    }
    println!("Client disconnected: {}", name);
}


async fn broadcast_job_update(state: AppState, payload: JobPayload) {
    let msg = JobUpdateMessage {
        msg_type: "job_update".to_string(),
        payload,
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    
    let senders: Vec<mpsc::Sender<Message>> = {
        let clients = state.clients.lock().unwrap();
        clients.iter()
            .filter(|c| !c.name.starts_with("Worker"))
            .map(|c| c.sender.clone())
            .collect()
    };

    for sender in senders {
        let _ = sender.send(Message::Text(json.clone())).await;
    }
}

async fn submit_job(
    State(state): State<AppState>,
    body: String,
) -> String {
    println!("Received job submission: {}", body);
    
    let job_id = uuid::Uuid::new_v4().to_string();
    
    let job_json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return "Invalid JSON".to_string(),
    };

    let tags = job_json.get("tags");
    let tags_str = tags.and_then(|t| serde_json::to_string(t).ok());

    // Persist to DB as PENDING
    let _ = sqlx::query("INSERT INTO jobs (id, job_type, args, status, tags) VALUES (?, ?, ?, ?, ?)")
        .bind(&job_id)
        .bind("unknown") 
        .bind(&body)
        .bind("pending")
        .bind(tags_str)
        .execute(&state.pool)
        .await
        .map_err(|e| println!("DB Error: {}", e));

    // Try to dispatch
    tokio::spawn(try_dispatch_jobs(state.clone()));

    // Broadcast to dashboard
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let payload = JobPayload {
        id: job_id.clone(),
        timestamp,
        message: format!("New Job Submitted: {}", job_id),
        status: "pending".to_string(),
    };
    tokio::spawn(broadcast_job_update(state, payload));

    format!("Job {} queued", job_id)
}

async fn try_dispatch_jobs(state: AppState) {
    // 1. Fetch pending jobs (FIFO)
    let pending_jobs: Vec<(String, String, Option<String>)> = sqlx::query_as("SELECT id, args, tags FROM jobs WHERE status = 'pending' ORDER BY created_at ASC")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    if pending_jobs.is_empty() {
        return;
    }

    println!("Attempting to dispatch {} pending jobs...", pending_jobs.len());

    for (job_id, body, _tags_json) in pending_jobs {
        // Parse body to check for target
        let job_json: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
        let target_worker = job_json.get("target").and_then(|v| v.as_str()).map(|s| s.trim_start_matches('@').to_string());

        let mut assigned_worker_idx = None;
        let mut assigned_worker_name = String::new();

        {
            let mut clients = state.clients.lock().unwrap();
            
            // Find a suitable worker
            for (i, client) in clients.iter().enumerate() {
                // Skip non-worker clients (e.g. Dashboard)
                if !client.name.starts_with("Worker") {
                    continue;
                }

                if client.status == WorkerStatus::Busy {
                    continue;
                }

                if let Some(ref target) = target_worker {
                    if &client.name != target {
                        continue;
                    }
                }
                
                // Found a match
                assigned_worker_idx = Some(i);
                break;
            }
            
            if let Some(idx) = assigned_worker_idx {
                // Mark busy
                clients[idx].status = WorkerStatus::Busy;
                assigned_worker_name = clients[idx].name.clone();
                println!("Assigning job {} to worker {}", job_id, clients[idx].name);
            }
        }

        if let Some(idx) = assigned_worker_idx {
            // Send job
            let sender = {
                let clients = state.clients.lock().unwrap();
                clients[idx].sender.clone()
            };
            
            if sender.send(Message::Text(body)).await.is_ok() {
                // Update DB
                let _ = sqlx::query("UPDATE jobs SET status = 'processing' WHERE id = ?")
                    .bind(&job_id)
                    .execute(&state.pool)
                    .await;

                // Broadcast update
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                let payload = JobPayload {
                    id: job_id.clone(),
                    timestamp,
                    message: format!("Job picked up by {}", assigned_worker_name),
                    status: "processing".to_string(),
                };
                tokio::spawn(broadcast_job_update(state.clone(), payload));
            } else {
                println!("Failed to send job to worker, reverting status");
                 // Revert worker status
                 let mut clients = state.clients.lock().unwrap();
                 clients[idx].status = WorkerStatus::Idle;
            }
        }
    }
}
