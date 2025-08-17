use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::AppState;

#[axum::debug_handler]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(submission_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, submission_id, state))
}

async fn handle_socket(socket: WebSocket, submission_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Create a broadcast channel for this submission
    let (_tx, mut rx) = broadcast::channel::<String>(100);

    // Store the sender in a global map (in a real implementation, you'd want proper management)
    // For now, we'll just send initial status

    // Send initial submission status
    if let Ok(Some(submission)) = state.db.get_submission(submission_id).await {
        let status_msg = serde_json::json!({
            "event": "status_update",
            "status": submission.status
        });
        
        if sender
            .send(Message::Text(status_msg.to_string()))
            .await
            .is_err()
        {
            return;
        }
    }

    // Handle incoming messages (though we mainly send updates)
    tokio::select! {
        _ = async {
            while let Some(msg) = receiver.next().await {
                if let Ok(msg) = msg {
                    match msg {
                        Message::Text(_) => {
                            // Handle text messages if needed
                        }
                        Message::Close(_) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        } => {}
        
        _ = async {
            while let Ok(message) = rx.recv().await {
                if sender.send(Message::Text(message)).await.is_err() {
                    break;
                }
            }
        } => {}
    }
}