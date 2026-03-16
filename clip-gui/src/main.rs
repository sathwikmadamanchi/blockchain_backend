use axum::{
    routing::{get, post},
    Json, Router, extract::Path,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use clip_client::{ClipClient, proto::Transaction};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct SubmitRequest {
    metadata: String,
    count: u64,
}

#[derive(Serialize, Deserialize)]
struct VerifyRequest {
    block_index: u64,
    tx_hash_hex: String,
}

#[derive(Serialize)]
struct APIResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

struct AppState {
    client: Mutex<ClipClient>,
}

#[tokio::main]
async fn main() {
    // Using 127.0.0.1 is more reliable on Windows than [::1] in some configurations
    let addr = "http://localhost:50051".to_string();
    let client = loop {
        match ClipClient::connect(addr.clone()).await {
            Ok(c) => break c,
            Err(e) => {
                eprintln!("Waiting for CLIP Ledger at {}... (Error: {})", addr, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    };

    let shared_state = Arc::new(AppState {
        client: Mutex::new(client),
    });

    let app = Router::new()
        .route("/api/submit", post(submit_batch))
        .route("/api/verify", post(verify_proof))
        .route("/api/block/:index", get(get_block))
        .nest_service("/", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("CLIP GUI Dashboard running at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn submit_batch(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(payload): Json<SubmitRequest>,
) -> Json<APIResponse<serde_json::Value>> {
    let mut transactions = Vec::new();
    let mut tx_hashes = Vec::new();

    for i in 0..payload.count {
        let meta = format!("{}_{}", payload.metadata, i);
        let mut hasher = Sha256::new();
        hasher.update(meta.as_bytes());
        let hash = hasher.finalize().to_vec();
        
        tx_hashes.push(hex::encode(&hash));
        
        transactions.push(Transaction {
            hash,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: meta.into_bytes(),
        });
    }

    let mut client = state.client.lock().await;
    match client.submit_batch(transactions).await {
        Ok(res) => Json(APIResponse {
            success: true,
            data: Some(serde_json::json!({
                "block_hash": hex::encode(res.block_hash),
                "block_index": res.block_index,
                "transaction_hashes": tx_hashes,
            })),
            error: None,
        }),
        Err(e) => Json(APIResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn verify_proof(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(payload): Json<VerifyRequest>,
) -> Json<APIResponse<serde_json::Value>> {
    let tx_hash = match hex::decode(&payload.tx_hash_hex.trim()) {
        Ok(h) => h,
        Err(_) => return Json(APIResponse {
            success: false,
            data: None,
            error: Some("Invalid hex hash format".to_string()),
        }),
    };

    let mut client = state.client.lock().await;
    match client.verify_proof(payload.block_index, tx_hash).await {
        Ok(res) => Json(APIResponse {
            success: true,
            data: Some(serde_json::json!({
                "is_valid": res.is_valid,
                "timestamp": res.timestamp,
            })),
            error: None,
        }),
        Err(e) => Json(APIResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn get_block(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(index): Path<u64>,
) -> Json<APIResponse<serde_json::Value>> {
    let mut client = state.client.lock().await;
    match client.get_block(index).await {
        Ok(res) => Json(APIResponse {
            success: true,
            data: Some(serde_json::json!({
                "block_hash": hex::encode(res.block_hash),
                "timestamp": res.timestamp,
                "transaction_hashes": res.transaction_hashes.iter().map(|h| hex::encode(h)).collect::<Vec<_>>(),
                "prev_block_hash": hex::encode(res.prev_block_hash),
            })),
            error: None,
        }),
        Err(e) => Json(APIResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}
