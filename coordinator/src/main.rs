use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde::Serialize;

mod db;
use db::init_db;
use sqlx::{Pool, Sqlite};
use uuid;

#[derive(Clone)]
struct AppState {
    clients: Arc<Mutex<Vec<mpsc::Sender<Message>>>>,
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
    let count = state.clients.lock().unwrap().len();
    let mut workers = Vec::new();
    
    // Generate mock info for connected workers
    for i in 0..count {
        workers.push(WorkerInfo {
            id: format!("w{}", i),
            hostname: format!("Worker-{:02}", i + 1),
            ip: format!("192.168.1.1{:02}", i),
            gpu: if i % 2 == 0 { "RTX 4090".to_string() } else { "A100".to_string() },
            status: "busy".to_string(),
            task: "Prime Search".to_string(),
        });
    }
    
    Json(workers)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel(100);

    {
        let mut clients = state.clients.lock().unwrap();
        clients.push(tx);
    }
    
    println!("New client connected");

    // Spawn a task to forward messages from the channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task to read messages from the WebSocket
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // Handle incoming messages (e.g., job results)
            println!("Received from client: {:?}", msg);
        }
    });

    // Send welcome message
    let welcome_tx = {
        let guard = state.clients.lock().unwrap();
        guard.last().cloned()
    };

    if let Some(tx) = welcome_tx {
        let _ = tx.send(Message::Text("Welcome to NeuraGrid".to_string())).await;
    }


    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {},
        _ = (&mut recv_task) => {},
    }

    println!("Client disconnected");
}

async fn submit_job(
    State(state): State<AppState>,
    body: String,
) -> String {
    println!("Received job submission: {}", body);
    
    // Parse job to get ID (or generate one if missing)
    let job_id = uuid::Uuid::new_v4().to_string();
    
    // Persist to DB
    let _ = sqlx::query("INSERT INTO jobs (id, job_type, args, status) VALUES (?, ?, ?, ?)")
        .bind(&job_id)
        .bind("unknown") // We'd parse the JSON properly in a real app
        .bind(&body)
        .bind("pending")
        .execute(&state.pool)
        .await
        .map_err(|e| println!("DB Error: {}", e));

    let msg = Message::Text(body);
    
    // 1. Lock and clone senders to avoid holding lock across await
    let clients: Vec<mpsc::Sender<Message>> = {
        let guard = state.clients.lock().unwrap();
        guard.clone()
    };
    
    let mut sent_count = 0;
    
    // 2. Iterate and send (async)
    for client in clients {
        if client.send(msg.clone()).await.is_ok() {
            sent_count += 1;
        } else {
            println!("Failed to send to a client (disconnected)");
        }
    }

    format!("Job {} submitted to {} workers", job_id, sent_count)
}
