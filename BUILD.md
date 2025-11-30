# Build and Setup Guide

Complete guide to building and running the iptoasn-server Node.js addon.

## Prerequisites

### 1. Install Rust

```bash
# Install rustup (Rust installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart your shell or run:
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install Node.js

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Or download from: https://nodejs.org/

# Verify installation
node --version
npm --version
```

### 3. Install Build Tools

#### Linux (Debian/Ubuntu)
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Windows
```bash
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
# Select "Desktop development with C++"
```

## Building the Project

### Step 1: Clone/Navigate to the Workspace

```bash
cd iptoasn-workspace
```

### Step 2: Build the Rust Core Library

```bash
# Build in debug mode (faster compilation)
cargo build

# Build in release mode (optimized)
cargo build --release

# Run tests
cargo test
```

### Step 3: Build the Node.js Addon

```bash
cd iptoasn-node

# Install Node.js dependencies (includes @napi-rs/cli)
npm install

# Build the native addon in debug mode
npm run build:debug

# Build in release mode (recommended for production)
npm run build
```

This will create a `.node` file in the `iptoasn-node` directory named something like:
- `iptoasn-node.linux-x64-gnu.node` (Linux)
- `iptoasn-node.darwin-arm64.node` (macOS M1/M2)
- `iptoasn-node.darwin-x64.node` (macOS Intel)
- `iptoasn-node.win32-x64-msvc.node` (Windows)

### Step 4: Run Examples

```bash
cd ../examples

# Install Express (for server example)
npm install

# Run basic usage example
node basic-usage.js

# Run Express server example
node express-server.js
```

## Troubleshooting

### "Cannot find module './iptoasn-node.*.node'"

This means the native addon wasn't built. Run:
```bash
cd iptoasn-node
npm run build
```

### "error: linking with 'cc' failed"

Missing C compiler. Install build tools for your platform (see Prerequisites).

### "ENOENT: no such file or directory, open '...metadata.json'"

This is normal on first run. The cache directory will be created automatically.

### Network/Download Issues

If you have issues downloading the database:

1. **Use a local file**:
   ```javascript
   const db = new IpToAsn('file:///path/to/ip2asn-combined.tsv.gz', './cache');
   ```

2. **Check firewall/proxy settings**

3. **Try manual download**:
   ```bash
   mkdir -p cache
   wget https://iptoasn.com/data/ip2asn-combined.tsv.gz -O cache/ip2asn-combined.tsv.gz
   ```

## Development Workflow

### Making Changes to Rust Code

1. Edit files in `iptoasn-core/src/` or `iptoasn-node/src/`
2. Rebuild:
   ```bash
   cd iptoasn-node
   npm run build
   ```
3. Test your changes

### Hot Reload Development

For faster iteration during development:

```bash
# Terminal 1: Watch Rust changes
cd iptoasn-node
npm run build:debug

# Terminal 2: Run your Node.js app with nodemon
npm install -g nodemon
nodemon ../examples/basic-usage.js
```

## Platform-Specific Notes

### Linux

The addon will work on most Linux distributions. For Alpine Linux (musl):
```bash
rustup target add x86_64-unknown-linux-musl
npm run build -- --target x86_64-unknown-linux-musl
```

### macOS

On Apple Silicon (M1/M2), the addon builds for `aarch64-apple-darwin` by default.
For universal binaries:
```bash
npm run build -- --target universal-apple-darwin
```

### Windows

- Ensure Visual Studio Build Tools are installed
- Use PowerShell or Command Prompt (not Git Bash)
- Long path support may need to be enabled

## Production Deployment

### Docker

Example Dockerfile:

```dockerfile
FROM node:18-alpine

# Install Rust and build dependencies
RUN apk add --no-cache \
    curl \
    build-base \
    openssl-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy workspace
COPY . .

# Build addon
WORKDIR /app/iptoasn-node
RUN npm install
RUN npm run build

# Copy example
WORKDIR /app
COPY examples ./examples
WORKDIR /app/examples
RUN npm install

EXPOSE 3000
CMD ["node", "express-server.js"]
```

### PM2 Process Manager

```bash
# Install PM2
npm install -g pm2

# Start application
cd examples
pm2 start express-server.js --name iptoasn-server

# Monitor
pm2 monit

# Logs
pm2 logs iptoasn-server
```

## Environment Variables

```bash
# Database configuration
export DATABASE_URL="https://iptoasn.com/data/ip2asn-combined.tsv.gz"
export CACHE_DIR="./cache"
export UPDATE_INTERVAL="60"  # minutes

# Logging
export RUST_LOG="info"  # error, warn, info, debug, trace

# Server (for Express example)
export PORT="3000"
```

## Performance Tuning

### Memory

The database uses approximately 200-300MB of memory. Ensure your system has at least 512MB available.

### CPU

Lookups are single-threaded but extremely fast (~1-10μs). A single core can handle 100,000+ lookups per second.

For high-load scenarios, run multiple Node.js processes:

```bash
# With PM2
pm2 start express-server.js -i max
```

### Disk Space

- Database file: ~30-40MB compressed, ~100MB uncompressed
- Cache directory: 40-50MB total

## Next Steps

1. ✅ Build successful? Run the examples!
2. 📖 Read the [API documentation](../iptoasn-node/README.md)
3. 🚀 Integrate into your application
4. 📊 Monitor performance and memory usage
5. 🔄 Configure auto-updates for your use case

## Getting Help

- Check the [README](../README.md)
- Review [examples](../examples/)
- Open an issue on GitHub
- Check NAPI-RS documentation: https://napi.rs/

## Continuous Integration

Example GitHub Actions workflow:

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - uses: actions/setup-node@v3
        with:
          node-version: 18
      
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build Rust
        run: cargo build --release
      
      - name: Build Node addon
        working-directory: ./iptoasn-node
        run: |
          npm install
          npm run build
      
      - name: Run examples
        working-directory: ./examples
        run: |
          npm install
          node basic-usage.js
```

Happy building! 🚀
