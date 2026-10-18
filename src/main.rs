mod error;
mod protocol;
mod transport;
mod decoders;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use mio::{Events, Poll, Token};
use protocol::{simple_parser::SimpleParser, Parser, TelemtryPacket};
use std::{thread, time::Duration};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use transport::{tcpuart::TcpUartSource, DataSource};

const SOURCE_TOKEN: Token = Token(0);
const SERVER_ADDR: &str = "127.0.0.1:15113";
const EVENTS_CAPACITY: usize = 128;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<TelemtryPacket>,
}

#[tokio::main]
async fn main() {
    // Increase channel size to handle high-frequency telemetry
    let (tx, _rx) = broadcast::channel::<TelemtryPacket>(1000);

    let state = AppState { tx: tx.clone() };

    // Background network thread with automatic reconnect logic
    thread::spawn(move || {
        let mut poll = Poll::new().expect("Failed to initialize Poll");
        let mut events = Events::with_capacity(EVENTS_CAPACITY);
        let addr = SERVER_ADDR.parse().unwrap();
        let mut parser = SimpleParser::new();

        loop {
            println!("Attempting to connect to TCP UART at {}...", SERVER_ADDR);
            let mut source = TcpUartSource::new(addr);

            // Try to connect, if it fails immediately, wait and retry
            if let Err(e) = source.connect() {
                eprintln!("Failed to connect: {:?}. Retrying in 2 seconds...", e);
                thread::sleep(Duration::from_secs(2));
                continue;
            }

            // Register with the poll
            if let Err(e) = source.register(poll.registry(), SOURCE_TOKEN) {
                eprintln!("Failed to register socket: {:?}. Retrying in 2 seconds...", e);
                thread::sleep(Duration::from_secs(2));
                continue;
            }

            println!("Connected to TCP UART source!");

            let mut connection_alive = true;
            while connection_alive {
                if let Err(e) = poll.poll(&mut events, Some(Duration::from_millis(500))) {
                    eprintln!("Poll error: {:?}", e);
                    break;
                }

                for event in events.iter() {
                    if event.token() == SOURCE_TOKEN && (event.is_readable() || event.is_error()) {
                        match source.read_bytes() {
                            Ok(Some(bytes)) => {
                                parser.feed_bytes(&bytes);
                                while let Some(packet) = parser.try_parse() {
                                    let _ = tx.send(packet);
                                }
                            }
                            Ok(None) => continue, // WouldBlock
                            Err(e) => {
                                eprintln!("TCP Connection dropped: {:?}", e);
                                connection_alive = false; // Break the inner loop to trigger reconnect
                                break;
                            }
                        }
                    }
                }
            }

            // We broke out of the reading loop due to a connection error
            println!("Connection lost. Reconnecting in 2 seconds...");
            thread::sleep(Duration::from_secs(2));
        }
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running at http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.tx))
}

async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<TelemtryPacket>) {
    let mut rx = tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(packet) => {
                if let Ok(json) = serde_json::to_string(&packet) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                // Ignore lagged error
                continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}
