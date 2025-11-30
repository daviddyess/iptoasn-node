# iptoasn-node

> High-performance IP to ASN lookup service - Rust library with Node.js bindings

A monorepo containing a high-performance IP to ASN (Autonomous System Number) lookup service built in Rust, with native Node.js bindings via NAPI-RS.

## 📦 Packages

### [`iptoasn-core`](./iptoasn-core)

Core Rust library providing the database functionality:

- Database fetching with HTTP conditional requests
- TSV parsing with string interning
- Binary search lookup (O(log n))
- Background updater with hot-swapping

### [`iptoasn-node`](./iptoasn-node)

Node.js native addon built with NAPI-RS:

- Native bindings to the Rust core
- Async/sync API for Node.js
- TypeScript definitions included
- Cross-platform support

## 🚀 Quick Start

### Node.js Usage

```bash
cd iptoasn-node
npm install
npm run build
```

```javascript
const { IpToAsn } = require("iptoasn-node");

const db = new IpToAsn(
  "https://iptoasn.com/data/ip2asn-combined.tsv.gz",
  "./cache"
);

await db.load();
const result = db.lookup("8.8.8.8");
console.log(result);
// {
//   ip: '8.8.8.8',
//   announced: true,
//   firstIp: '8.8.8.0',
//   lastIp: '8.8.8.255',
//   asNumber: 15169,
//   asCountryCode: 'US',
//   asDescription: 'GOOGLE'
// }
```

### Development

```bash
# Build the entire workspace
cargo build --release

# Run tests
cargo test

# Build Node.js addon
cd iptoasn-node
npm run build

# Run examples
node ../examples/basic-usage.js
node ../examples/express-server.js
```

## 🏗️ Architecture

```
┌──────────────────────────────────────────┐
│          Your Application                │
│  (Node.js, Rust binary, or both)         │
└────────────┬─────────────────────────────┘
             │
     ┌───────┴────────┐
     │                │
┌────▼────┐    ┌─────▼──────┐
│ Node.js │    │   Direct   │
│ Bindings│    │ Rust Usage │
│ (NAPI)  │    │            │
└────┬────┘    └─────┬──────┘
     │                │
     └───────┬────────┘
             │
      ┌──────▼──────┐
      │ iptoasn-core│
      │             │
      │ - Fetcher   │
      │ - Parser    │
      │ - Store     │
      │ - Updater   │
      └─────────────┘
```

## ⚡ Features

- **Blazing Fast**: Microsecond-level lookups with binary search
- **Memory Efficient**: String interning keeps memory usage low (~200-300MB)
- **Auto-Updates**: Background task with smart HTTP caching
- **Zero-Downtime**: Hot-swaps database atomically
- **Cross-Platform**: Runs on Linux, macOS, Windows
- **IPv4 & IPv6**: Full support for both
- **Production-Ready**: Comprehensive error handling and logging

## 📊 Performance

| Operation       | Time         |
| --------------- | ------------ |
| Lookup          | 1-10 μs      |
| Database Load   | ~2-5 seconds |
| Hot-Swap Update | <10 ms       |
| Memory Usage    | ~200-300 MB  |

## 🔄 How It Works

### Database Updates

1. **Periodic checks** at configured interval (e.g., every 60 minutes)
2. **Conditional HTTP** requests with ETag/Last-Modified headers
3. **304 Not Modified** if data unchanged (no download)
4. **Download & parse** if new data available
5. **Atomic hot-swap** - old data replaced seamlessly
6. **Zero downtime** - lookups continue throughout

### Lookup Process

```
User Query ("8.8.8.8")
        ↓
    Parse IP
        ↓
Binary Search (O(log n))
  on sorted records
        ↓
    Return ASN Info
```

## 🛠️ Development

### Prerequisites

- Rust 1.70+ ([Install](https://rustup.rs/))
- Node.js 16+ ([Install](https://nodejs.org/))
- cargo, npm/yarn/pnpm

### Building

```bash
# Build Rust workspace
cargo build --release

# Build Node.js addon
cd iptoasn-node
npm install
npm run build
```

### Testing

```bash
# Rust tests
cargo test

## 📁 Project Structure

```

iptoasn-workspace/
├── Cargo.toml # Workspace definition
├── iptoasn-core/ # Core Rust library
│ ├── src/
│ │ ├── lib.rs # Public API
│ │ ├── fetcher.rs # HTTP fetching
│ │ ├── parser.rs # TSV parsing
│ │ ├── store.rs # Lookup data structure
│ │ ├── updater.rs # Background updates
│ │ └── error.rs # Error types
│ └── Cargo.toml
├── iptoasn-node/ # Node.js bindings (NAPI-RS)
│ ├── src/
│ │ └── lib.rs # NAPI bindings
│ ├── index.js # JS loader
│ ├── index.d.ts # TypeScript definitions
│ ├── package.json
│ └── Cargo.toml
└── examples/ # Usage examples
├── basic-usage.js
└── express-server.js

```

## 🌐 Examples

See the [`examples/`](./examples) directory for:

- **basic-usage.js** - Simple lookup example
- **express-server.js** - Full HTTP API server

## 📖 Documentation

- [Core Library (Rust)](./iptoasn-core/README.md)
- [Node.js Package](./iptoasn-node/README.md)

## 🤝 Contributing

Contributions are welcome! Please:

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 📮 Contact

- **Author**: David Dyess II
- **Repository**: https://github.com/daviddyess/iptoasn-node
- **Issues**: https://github.com/daviddyess/iptoasn-node/issues

---

Built with ❤️ using Rust 🦀 and Node.js 💚
```
