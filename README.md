# udp-tokio-stack

[![CI](https://github.com/Fede-Berga/udp-tokio-stack/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Fede-Berga/udp-tokio-stack/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A small Rust demo of asynchronous UDP unicast, broadcast and multicast on Tokio: one server listens on three sockets, and a client sends a bincode-encoded message in each mode.

## Quick start

Prerequisites: a stable Rust toolchain (tested with Rust 1.99.1 on macOS 27.2; CI uses the latest stable on Ubuntu 24.04).

Start the server. Both programs log through `tracing`; without `RUST_LOG` they print errors only.

```sh
git clone https://github.com/Fede-Berga/udp-tokio-stack.git
cd udp-tokio-stack
RUST_LOG=info cargo run --release --bin server
```

In a second terminal, send one message per mode:

```sh
RUST_LOG=info cargo run --release --bin client -- unicast
RUST_LOG=info cargo run --release --bin client -- broadcast
RUST_LOG=info cargo run --release --bin client -- multicast
```

Keep `--release`: in debug builds `client multicast` panics (see [limitations](#design-decisions-and-limitations)). A unicast round trip, client first, then the server:

```text
2026-10-31T15:44:06.581609Z  INFO client: Running Unicast example -> 127.0.0.1:8080
2026-10-31T15:44:06.581788Z  INFO client: Sent Ping to unicast server
2026-10-31T15:44:06.581979Z  INFO client: Unicast reply: Message { id: 1, kind: Echo, data: [80, 111, 110, 103] }
```

```text
2026-10-31T15:44:00.509658Z  INFO server: Starting UDP Tokio Stack Server...
2026-10-31T15:44:00.509741Z  INFO server: Unicast listener on 127.0.0.1:8080
2026-10-31T15:44:00.509765Z  INFO server: Broadcast listener on 0.0.0.0:8081
2026-10-31T15:44:00.509798Z  INFO server: Multicast listener on 0.0.0.0:8082 group 239.255.42.42
2026-10-31T15:44:06.581880Z  INFO server: Received message label=unicast src=127.0.0.1:64066
2026-10-31T15:44:06.581919Z  INFO handle_message{Message { id: 1, kind: Ping, data: [] }}: udp_tokio_stack::service: Handling Ping from 1
```

`[80, 111, 110, 103]` is `Pong` in ASCII. Ctrl-C stops the server.

## How it works

The library crate holds the message type ([`src/proto`](src/proto/mod.rs)), the reply logic ([`src/service`](src/service/mod.rs)) and the socket helpers ([`src/lib.rs`](src/lib.rs)); the server and client programs live in [`src/bin`](src/bin).

### Server

The server runs one Tokio task per socket. Each task decodes incoming datagrams into a `Message` and spawns another task that computes the reply with `handle_message` and sends it to the source address, so a slow reply does not block the receive loop. Datagrams that do not decode are logged and dropped; Ctrl-C aborts the three loops.

| Socket | Bound to (option) | Socket options |
| --- | --- | --- |
| Unicast | `127.0.0.1:8080` (`--bind`, `--port`) | none |
| Broadcast | `0.0.0.0:8081` (`--broadcast-port`) | `SO_BROADCAST`, set by the shared helper; receiving does not need it |
| Multicast | `0.0.0.0:8082` (`--mcast-port`) | joins `239.255.42.42` (`--mcast-group`) on interface `0.0.0.0` |

The reply depends on the message kind, not on the socket the datagram arrived on:

| Request | Reply |
| --- | --- |
| `Ping` | `Echo` with data `Pong` |
| `Echo` | `Echo` with the same data, after one second |
| `Broadcast` | `Broadcast` with data `ack` |
| `Multicast` | `Multicast` with data `ack` |

### Client

- `unicast [--target 127.0.0.1:8080]` binds an ephemeral port, connects the socket to the target, sends `Ping` (id 1, no data) and waits for one reply. Because the socket is connected, a missing local server shows up as `Connection refused`.
- `broadcast [--target 255.255.255.255:8081] [--bind 0.0.0.0:0]` sets `SO_BROADCAST`, which the kernel requires before sending to a broadcast address, sends `Broadcast` (id 2, `hello broadcast`) and logs the first reply that arrives within one second. The limited broadcast address `255.255.255.255` never crosses a router; a subnet broadcast such as `192.168.1.255:8081` can be passed as `--target` instead.
- `multicast [--target 239.255.42.42:8082]` sets the multicast TTL to 1, so the datagram stays on the local network, sends `Multicast` (id 3, `hello mcast`) and exits without reading the server's `ack`. Multicast loopback is left at the OS default (on in Linux and macOS), which is why a server on the same host receives the datagram. The group is in `239.0.0.0/8`, the administratively scoped range (RFC 2365).

### Message format

`Message { id: u64, kind: MessageKind, data: Vec<u8> }` derives serde's `Serialize` and `Deserialize`. Each datagram carries exactly one message, written by `bincode::serialize` (bincode 1.3), which uses little-endian, fixed-size integers:

| Bytes | Field |
| --- | --- |
| 8 | `id` (`u64`) |
| 4 | `kind` as variant index (`u32`): `Ping` 0, `Echo` 1, `Broadcast` 2, `Multicast` 3 |
| 8 | length of `data` (`u64`) |
| n | `data` |

A `Ping` is 20 bytes and the broadcast message 35. There is no framing, version field or checksum beyond UDP's own; the server reads up to 2048 bytes per datagram.

## Running the tests

```sh
cargo test
```

The single integration test ([`tests/integration_test.rs`](tests/integration_test.rs)) binds an inline echo socket on `127.0.0.1:19000`, sends an encoded `Ping` and checks that the reply carries the same id. It exercises the message encoding over a real socket; it does not start the server binary or call `handle_message`, and broadcast and multicast have no automated tests. CI runs `cargo fmt --check`, `cargo clippy`, `cargo test` and `cargo deny check bans licenses sources` on Ubuntu 24.04; security advisories are tracked by Dependabot.

## Design decisions and limitations

This is a learning project about socket setup with Tokio, not a protocol library.

- Delivery is best effort: no retransmission, ordering or authentication. The unicast client has no receive timeout, so if the request or the reply is lost it waits until interrupted.
- Broadcast and multicast are IPv4 only (`join_multicast_v4`, `set_multicast_ttl_v4`), and the unicast client binds `0.0.0.0:0`, so it only reaches IPv4 servers.
- One server per host: the sockets are bound without `SO_REUSEADDR` or `SO_REUSEPORT`, so a second server on the same ports fails with `Address already in use`.
- The kernel chooses the network interface for broadcast and multicast from the routing table: the group is joined on interface `0.0.0.0` and the client does not set `IP_MULTICAST_IF`. Neither program can pick an interface, which matters with VPNs or several network interfaces. On Linux, the join fails with `No such device` when no route covers the group, for example on a host without a default route.
- The broadcast and multicast sockets are bound to `0.0.0.0`, so host firewalls apply to them. On macOS with the application firewall on, the locally built `server` binary receives no broadcast or multicast datagrams until it is allowed to accept incoming connections; datagrams sent to `127.0.0.1` still arrive. On Linux, ufw or firewalld, when enabled, can drop these datagrams unless UDP 8081 and 8082 are allowed.
- `client multicast` panics in debug builds: it hands a blocking `std::net::UdpSocket` to `tokio::net::UdpSocket::from_std`, which Tokio rejects with a debug assertion. Release builds skip the assertion and send normally.
- bincode 1.x is unmaintained upstream ([RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141)). It stays because it defines the wire format, and [`deny.toml`](deny.toml) records the exception.

## License

[MIT](LICENSE)
