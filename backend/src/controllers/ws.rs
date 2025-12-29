//! WebSocket controller for terminal streaming

use crate::services::process_manager::{OutputLine, PROCESS_MANAGER};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "output")]
    Output { line: String, is_stderr: bool },
    #[serde(rename = "input")]
    Input { data: String },
    #[serde(rename = "kill")]
    Kill,
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "connected")]
    Connected { task_id: i32, is_running: bool },
}

pub async fn terminal_handler(
    ws: WebSocketUpgrade,
    Path(task_id): Path<i32>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, task_id))
}

async fn handle_socket(socket: WebSocket, task_id: i32) {
    let (mut sender, mut receiver) = socket.split();

    // Check if process is running
    let is_running = PROCESS_MANAGER.is_running(task_id);

    // Send connected message
    let connected_msg = serde_json::to_string(&WsMessage::Connected { task_id, is_running })
        .unwrap_or_default();
    if sender.send(Message::Text(connected_msg.into())).await.is_err() {
        return;
    }

    // Subscribe to process output
    let mut output_rx: broadcast::Receiver<OutputLine> = PROCESS_MANAGER.subscribe_output();

    // Spawn task to forward output to WebSocket
    let send_task = tokio::spawn(async move {
        loop {
            match output_rx.recv().await {
                Ok(output) => {
                    if output.task_id == task_id {
                        let msg = WsMessage::Output {
                            line: output.line,
                            is_stderr: output.is_stderr,
                        };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Skip lagged messages
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                        match msg {
                            WsMessage::Input { data } => {
                                if let Err(e) = PROCESS_MANAGER.send_input(task_id, &data).await {
                                    tracing::error!("Failed to send input: {}", e);
                                }
                            }
                            WsMessage::Kill => {
                                if let Err(e) = PROCESS_MANAGER.kill_process(task_id).await {
                                    tracing::error!("Failed to kill process: {}", e);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}
