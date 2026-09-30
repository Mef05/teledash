# TeleDash

A modular desktop utility for 3D visualization, realtime telemetry, and sensor simulation for embedded systems. Built entirely in Rust, focusing on low-latency, non-blocking I/O, and a heavily decoupled architecture.

## Project Goal
To provide a robust, highly-performant software architecture where embedded hardware can stream raw data (over TCP-UART bridges, USB-Serial, or UDP) to a host machine. The host application parses this binary stream, extracts telemetry data, and visualizes it in real-time using fast plots and 3D models (e.g., for IMU / Quaternion data).

## Architecture & Milestones

The project is being built progressively in isolated architectural layers:

### Completed
* **Milestone 1: Transport Layer** 
  * Implemented non-blocking network I/O using the `mio` epoll event loop.
  * `DataSource` trait abstraction for decoupled transport (currently implemented: `TcpUartSource`).
  * Robust error handling and non-blocking socket management (`WouldBlock` handling).
* **Milestone 2: Parser & Protocol Layer** 
  * Binary framing implementation using a Magic Bytes state machine (`0xAA 0xBB`).
  * Seamless handling of packet fragmentation and network delays.
  * Checksum validation (XOR).
  * Fully unit tested for single, fragmented, and chained/multiple packets.

### Upcoming
* **Milestone 3: Presentation Layer (GUI)**
  * Integrate `egui` (Immediate Mode GUI) for real-time data visualization.
  * Multi-threaded architecture using `mpsc` channels to completely decouple the `mio` network loop from the GUI rendering loop.
* **Milestone 4: 3D Visualization**
  * Real-time 3D model rendering for spatial orientation (Euler angles / Quaternions).
* **Milestone 5: Sensor Simulation Engine**
  * A local dummy source to generate synthetic sensor data, allowing GUI and Parser testing without physical embedded hardware.
* **Milestone 6: Plugin & Custom Sensor Registry**
  * Extensible architecture for dynamically registering new binary formats and custom sensor payload parsers.

## Getting Started
*(Instructions will be added once the GUI layer is merged)*
