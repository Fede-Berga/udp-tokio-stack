# UDP Tokio Stack

A professional-grade, asynchronous UDP network stack implementation in Rust, powered by **Tokio**. This project demonstrates complete unicast, broadcast, and multicast communication patterns with structured logging and robust error handling.

## Features

- **Asynchronous I/O**: Built on top of the `tokio` runtime for high-performance non-blocking networking.
- **Robust Protocol**: Uses `serde` and `bincode` for type-safe, efficient binary serialization of messages.
- **Multiple Patterns**:
  - **Unicast**: Direct client-server request-response.
  - **Broadcast**: One-to-all communication on the local subnet.
  - **Multicast**: Efficient one-to-many group communication.
- **Professional Tooling**:
  - **CLI**: Powered by `clap` for robust command-line argument parsing.
  - **Logging**: Structured logging with `tracing` and `tracing-subscriber`.
  - **Error Handling**: Modern error handling with `thiserror` and `anyhow`.

## Getting Started

### Prerequisites

- Rust (latest stable)
- Cargo

### Building

```bash
cargo build --release
```

## Usage

This project contains two main binaries: `server` and `client`.

### Server

The server listens on multiple ports simultaneously for unicast, broadcast, and multicast traffic.

**Start the server:**
```bash
# Default (Unicast: 8080, Broadcast: 8081, Multicast: 8082)
cargo run --bin server

# Custom configuration
cargo run --bin server -- --port 9000 --broadcast-port 9001 --mcast-port 9002
```

**Options:**
```
  -p, --port <PORT>             Port to bind unicast listener to [default: 8080]
      --bind <BIND>             Bind address for unicast listener [default: 127.0.0.1]
      --broadcast-port <PORT>   Broadcast port to listen on [default: 8081]
      --mcast-group <IP>        Multicast group to join [default: 239.255.42.42]
      --mcast-port <PORT>       Multicast port [default: 8082]
  -h, --help                    Print help
```

### Client

 The client receives commands via subcommands to send different types of messages.

**1. Unicast (Request-Response)**
Connects to the server, sends a 'Ping', and awaits a 'Pong' echo response.
```bash
cargo run --bin client -- unicast --target 127.0.0.1:8080
```

**2. Broadcast**
Sends a message to the broadcast address. The server (and any other listeners) will acknowledge.
```bash
cargo run --bin client -- broadcast --target 255.255.255.255:8081
```

**3. Multicast**
Sends a message to a multicast group.
```bash
cargo run --bin client -- multicast --target 239.255.42.42:8082
```

## Logging

You can control the log level using the `RUST_LOG` environment variable.

```bash
# Show info logs (recommended)
RUST_LOG=info cargo run --bin server

# Show debug logs for detailed library output
RUST_LOG=debug cargo run --bin client -- unicast
```

## Testing

Run the integration suite to verify connectivity and logic:

```bash
cargo test
```
