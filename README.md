# TeleDash

Real-time telemetry for embedded systems, written in Rust. A board streams binary sensor packets over TCP (for example through a TCP-UART bridge); TeleDash parses the stream and pushes every packet live to a browser dashboard over WebSockets.

```
embedded board / simulator ──TCP──▶ network thread (mio / epoll)
                                         │  framing parser
                                         ▼
                              tokio broadcast channel
                                         │
                                         ▼
                        Axum server ──WebSocket──▶ browser dashboard
```

## How it works

**Transport (`src/transport`)**
- A dedicated thread runs a non-blocking `mio` (epoll) event loop and reads from the source.
- Sources sit behind a `DataSource` trait (`connect`, `register`, `read_bytes`, `write_bytes`), so a new transport does not touch the parser or the server. Implemented today: `TcpUartSource`.
- `WouldBlock` is handled as "no data yet"; a closed or failed connection is reported as a `TransportError` and the thread reconnects every 2 seconds.

**Protocol (`src/protocol`)**
- Frame layout:

  | Magic | Length | Sensor ID | Payload | Checksum |
  |---|---|---|---|---|
  | `0xAA 0xBB` | 1 byte (N) | 1 byte | N bytes | 1 byte, XOR of all previous bytes |

- `SimpleParser` buffers incoming bytes, skips to the next magic sequence, waits until a full frame has arrived, and drops the magic bytes to resync when a checksum fails.
- It handles a frame split across several reads and several frames arriving in one read.

**Server and dashboard (`src/main.rs`, `static/`)**
- Parsed packets go into a `tokio::sync::broadcast` channel (capacity 1000).
- An Axum server serves the dashboard and a `/ws` endpoint; each WebSocket client gets its own receiver, so one slow client skips lagged packets instead of blocking ingestion or the other clients.
- The dashboard shows connection status, a packet counter and one live card per sensor ID with its latest payload and timestamp. It reconnects automatically if the server restarts.

**Simulator (`simulator/sim_sensor.py`)**
- A TCP server that impersonates a board with four sensors: temperature (1 byte), battery (float32), GPS (2 x float32) and IMU pitch/roll/yaw (3 x float32).
- The IMU packet is deliberately sent in two halves, to exercise fragment reassembly over a real socket.

## Running it

Requires Rust (edition 2024) and Python 3.

```bash
# terminal 1: start the simulated board on 127.0.0.1:15113
python3 simulator/sim_sensor.py

# terminal 2: start TeleDash from the repository root (the dashboard is served from ./static)
cargo run
```

Then open http://localhost:3000.

## Tests

```bash
cargo test
```

Parser unit tests cover a single complete frame, a frame split across two reads, and two frames delivered in one buffer.

## Roadmap

- [x] Non-blocking TCP transport with reconnect
- [x] Binary framing parser with checksum validation and unit tests
- [x] WebSocket server and live browser dashboard
- [x] Python sensor simulator
- [ ] Sensor presets loaded from JSON (name, data type, endianness, unit) to show decoded values instead of raw hex
- [ ] Live plots and a 3D orientation view for IMU data
- [ ] More transports: USB-serial and UDP
