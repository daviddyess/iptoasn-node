// Express server example using iptoasn-server

const express = require('express');
const { IpToAsn } = require('../iptoasn-node');

const app = express();
const PORT = process.env.PORT || 3000;

let db = null;

async function initializeDatabase() {
    console.log('📦 Initializing database...');
    
    db = new IpToAsn(
        process.env.DATABASE_URL || 'https://iptoasn.com/data/ip2asn-combined.tsv.gz',
        process.env.CACHE_DIR || './cache'
    );

    await db.load();
    
    const stats = db.stats();
    console.log(`✅ Database loaded: ${stats.recordCount.toLocaleString()} records`);

    // Start auto-updates (check every 60 minutes)
    const updateInterval = parseInt(process.env.UPDATE_INTERVAL || '60');
    if (updateInterval > 0) {
        await db.startAutoUpdate(updateInterval);
        console.log(`⏰ Auto-update enabled (${updateInterval} minute interval)`);
    }
}

// Middleware to ensure database is loaded
function requireDb(req, res, next) {
    if (!db) {
        return res.status(503).json({
            error: 'Database not loaded yet',
            status: 'initializing'
        });
    }
    next();
}

// Routes
app.get('/', (req, res) => {
    res.send(`
        <!DOCTYPE html>
        <html>
        <head>
            <title>IP to ASN Lookup Service</title>
            <style>
                body { 
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
                    max-width: 800px; 
                    margin: 50px auto; 
                    padding: 20px; 
                    line-height: 1.6;
                }
                code { 
                    background: #f4f4f4; 
                    padding: 2px 8px; 
                    border-radius: 4px;
                    font-family: 'Monaco', 'Courier New', monospace;
                }
                h1 { color: #2c3e50; }
                h2 { color: #34495e; margin-top: 30px; }
                .endpoint { 
                    background: #ecf0f1; 
                    padding: 15px; 
                    margin: 10px 0;
                    border-radius: 5px;
                }
                a { color: #3498db; text-decoration: none; }
                a:hover { text-decoration: underline; }
            </style>
        </head>
        <body>
            <h1>🌐 IP to ASN Lookup Service</h1>
            <p>High-performance IP to ASN lookups powered by Rust</p>

            <h2>📡 API Endpoints</h2>
            
            <div class="endpoint">
                <strong>GET /v1/as/ip/:ip</strong><br>
                Look up an IP address<br>
                <small>Example: <a href="/v1/as/ip/8.8.8.8">/v1/as/ip/8.8.8.8</a></small>
            </div>

            <div class="endpoint">
                <strong>GET /health</strong><br>
                Health check with database statistics<br>
                <small>Example: <a href="/health">/health</a></small>
            </div>

            <div class="endpoint">
                <strong>POST /admin/update</strong><br>
                Force an immediate database update check
            </div>

            <h2>🚀 Features</h2>
            <ul>
                <li>⚡ Ultra-fast lookups (microseconds)</li>
                <li>🔄 Automatic background updates</li>
                <li>🌍 IPv4 and IPv6 support</li>
                <li>💾 Efficient caching with conditional HTTP requests</li>
                <li>🛡️ Zero-downtime database updates</li>
            </ul>
        </body>
        </html>
    `);
});

app.get('/health', requireDb, (req, res) => {
    const stats = db.stats();
    
    res.json({
        status: 'healthy',
        records: stats.recordCount,
        lastUpdate: stats.lastUpdateTimestamp 
            ? new Date(stats.lastUpdateTimestamp * 1000).toISOString()
            : null
    });
});

app.get('/v1/as/ip/:ip', requireDb, (req, res) => {
    const { ip } = req.params;
    
    try {
        const result = db.lookup(ip);
        
        if (result.announced) {
            res.json({
                ip: result.ip,
                announced: true,
                first_ip: result.firstIp,
                last_ip: result.lastIp,
                as_number: result.asNumber,
                as_country_code: result.asCountryCode,
                as_description: result.asDescription
            });
        } else {
            res.status(404).json({
                ip: result.ip,
                announced: false
            });
        }
    } catch (error) {
        res.status(400).json({
            error: 'Invalid IP address',
            message: error.message
        });
    }
});

app.post('/admin/update', requireDb, async (req, res) => {
    try {
        const updated = await db.forceUpdate();
        const stats = db.stats();
        
        res.json({
            updated,
            records: stats.recordCount,
            lastUpdate: stats.lastUpdateTimestamp 
                ? new Date(stats.lastUpdateTimestamp * 1000).toISOString()
                : null
        });
    } catch (error) {
        res.status(500).json({
            error: 'Update failed',
            message: error.message
        });
    }
});

// Error handling
app.use((err, req, res, next) => {
    console.error('Error:', err);
    res.status(500).json({
        error: 'Internal server error',
        message: err.message
    });
});

// Start server
async function start() {
    try {
        await initializeDatabase();
        
        app.listen(PORT, () => {
            console.log(`\n🚀 Server running on http://localhost:${PORT}`);
            console.log(`\n📖 Try these endpoints:`);
            console.log(`   http://localhost:${PORT}/`);
            console.log(`   http://localhost:${PORT}/v1/as/ip/8.8.8.8`);
            console.log(`   http://localhost:${PORT}/health\n`);
        });
    } catch (error) {
        console.error('❌ Failed to start server:', error);
        process.exit(1);
    }
}

// Graceful shutdown
process.on('SIGTERM', () => {
    console.log('\n⏹️  SIGTERM received, shutting down gracefully...');
    if (db) {
        db.stopAutoUpdate();
    }
    process.exit(0);
});

process.on('SIGINT', () => {
    console.log('\n⏹️  SIGINT received, shutting down gracefully...');
    if (db) {
        db.stopAutoUpdate();
    }
    process.exit(0);
});

start();
