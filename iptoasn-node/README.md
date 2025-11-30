# iptoasn-node

> High-performance IP to ASN (Autonomous System Number) lookup library powered by Rust

A blazingly fast Node.js library for looking up IP addresses and their associated ASN information. Built with Rust using NAPI-RS for optimal performance with automatic database updates.

## 🚀 Features

- ⚡ **Ultra-fast lookups** - Microsecond-level performance with binary search in native code
- 🔄 **Automatic updates** - Background task periodically checks for new data
- 🌍 **IPv4 and IPv6** - Full support for both IP versions
- 💾 **Smart caching** - Efficient HTTP conditional requests (ETag/Last-Modified)
- 🛡️ **Zero-downtime** - Hot-swaps database without interrupting service
- 🦀 **Rust-powered** - Native performance with memory safety
- 📦 **Easy to use** - Simple async/sync API
- 🎯 **TypeScript** - Full type definitions included

## 📦 Installation

```bash
npm install iptoasn-node
# or
yarn add iptoasn-node
# or
pnpm add iptoasn-node
```

## 🎯 Quick Start

```javascript
const { IpToAsn } = require("iptoasn-node");

async function main() {
  // Create database instance
  const db = new IpToAsn(
    "https://iptoasn.com/data/ip2asn-combined.tsv.gz",
    "./cache"
  );

  // Load database (downloads if needed, or loads from cache)
  await db.load();

  // Fast synchronous lookups!
  const result = db.lookup("8.8.8.8");
  console.log(result);
  // {
  //   ip: '8.8.8.8',
  //   announced: true,
  //   asNumber: 15169,
  //   asCountryCode: 'US',
  //   asDescription: 'GOOGLE',
  //   firstIp: '8.8.8.0',
  //   lastIp: '8.8.8.255'
  // }

  // Start automatic updates (every 60 minutes)
  await db.startAutoUpdate(60);
}

main();
```

## 📖 API Documentation

### Class: `IpToAsn`

#### `new IpToAsn(url, cacheDir)`

Create a new database instance.

- **url** `string` - Database URL (HTTP/HTTPS or `file://`)
- **cacheDir** `string` - Directory for caching downloaded databases

```javascript
const db = new IpToAsn(
  "https://iptoasn.com/data/ip2asn-combined.tsv.gz",
  "./cache"
);
```

#### `async load()`

Load the database (initial load or manual refresh). Downloads if needed, or loads from cache.

```javascript
await db.load();
```

#### `lookup(ip)` → `AsnResult`

Look up an IP address. **Synchronous** and very fast (microseconds).

- **ip** `string` - IPv4 or IPv6 address
- **Returns** `AsnResult` - ASN information

```javascript
const result = db.lookup("1.1.1.1");
```

**AsnResult:**

```typescript
{
  ip: string;
  announced: boolean;
  firstIp?: string;
  lastIp?: string;
  asNumber?: number;
  asCountryCode?: string;
  asDescription?: string;
}
```

#### `stats()` → `DatabaseStats`

Get database statistics.

```javascript
const stats = db.stats();
console.log(`${stats.recordCount} records loaded`);
```

**DatabaseStats:**

```typescript
{
  recordCount: number;
  lastUpdateTimestamp?: number; // Unix timestamp in seconds
}
```

#### `async startAutoUpdate(intervalMinutes)`

Start automatic database updates in the background.

- **intervalMinutes** `number` - How often to check for updates

The updater uses HTTP conditional requests (ETag/Last-Modified) to avoid unnecessary downloads.

```javascript
// Check for updates every hour
await db.startAutoUpdate(60);
```

#### `stopAutoUpdate()`

Stop automatic database updates.

```javascript
db.stopAutoUpdate();
```

#### `async forceUpdate()` → `boolean`

Force an immediate database update check.

- **Returns** `boolean` - Whether database was updated

```javascript
const updated = await db.forceUpdate();
```

## 🌐 Express Server Example

```javascript
const express = require("express");
const { IpToAsn } = require("iptoasn-node");

const app = express();
const db = new IpToAsn(
  "https://iptoasn.com/data/ip2asn-combined.tsv.gz",
  "./cache"
);

// Initialize
await db.load();
await db.startAutoUpdate(60);

// API endpoint
app.get("/v1/as/ip/:ip", (req, res) => {
  try {
    const result = db.lookup(req.params.ip);
    if (result.announced) {
      res.json(result);
    } else {
      res.status(404).json({ ip: req.params.ip, announced: false });
    }
  } catch (error) {
    res.status(400).json({ error: "Invalid IP address" });
  }
});

app.listen(3000);
```

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│      Node.js Application            │
│   (Your Express/Fastify server)     │
└──────────────┬──────────────────────┘
               │
        ┌──────▼──────┐
        │  NAPI-RS    │  ← Native bindings
        │  Bindings   │
        └──────┬──────┘
               │
    ┌──────────▼──────────────┐
    │   Rust Core Library     │
    │  - Binary search O(log n)│
    │  - String interning      │
    │  - Background updates    │
    │  - Smart HTTP caching    │
    └─────────────────────────┘
```

## ⚡ Performance

- **Lookup time**: ~1-10 microseconds per lookup
- **Memory usage**: ~200-300MB for 670K+ records
- **Update time**: Hot-swap in milliseconds
- **Zero GC pressure**: Data lives in Rust heap

## 🔄 How Updates Work

1. **Background task** checks for updates at specified interval
2. **Conditional request** sent with ETag/Last-Modified headers
3. If **304 Not Modified** returned, no download needed
4. If **new data** available:
   - Downloads and decompresses in memory
   - Parses TSV format
   - **Hot-swaps** database atomically
   - Old data cleaned up automatically
5. **Zero downtime** - lookups continue during updates

## 🛡️ Error Handling

The library handles errors gracefully:

- Network failures → Falls back to cached data
- Parse errors → Keeps existing database
- Invalid IPs → Returns error via exception

```javascript
try {
  const result = db.lookup("not-an-ip");
} catch (error) {
  console.error("Invalid IP:", error.message);
}
```

## 📊 Data Source

This library uses the IP to ASN database from [iptoasn.com](https://iptoasn.com/), which provides:

- IPv4 and IPv6 address ranges
- ASN numbers and descriptions
- Country codes
- Regular updates

## 🔧 Environment Variables

```bash
DATABASE_URL=https://iptoasn.com/data/ip2asn-combined.tsv.gz
CACHE_DIR=./cache
UPDATE_INTERVAL=60  # minutes
RUST_LOG=info       # Rust logging level
```

## 🏗️ Building from Source

```bash
# Install dependencies
npm install

# Build native module
npm run build

# Build for production
npm run build:release
```

## 📝 TypeScript

Full TypeScript definitions are included:

```typescript
import { IpToAsn, AsnResult, DatabaseStats } from "iptoasn-node";

const db = new IpToAsn(url, cacheDir);
const result: AsnResult = db.lookup("8.8.8.8");
```

## 🤝 Contributing

Contributions are welcome! Please see the main repository for contribution guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Credits

- Built with [NAPI-RS](https://napi.rs/)
- Data from [iptoasn.com](https://iptoasn.com/)
- Created by David Dyess II

## 🔗 Links

- [GitHub Repository](https://github.com/daviddyess/iptoasn-node)
- [npm Package](https://www.npmjs.com/package/iptoasn-node)
- [Documentation](https://github.com/daviddyess/iptoasn-node#readme)
