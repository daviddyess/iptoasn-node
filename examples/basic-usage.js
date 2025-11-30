// Basic usage example for iptoasn-server

const { IpToAsn } = require('../iptoasn-node');

async function main() {
    console.log('🚀 IP to ASN Lookup Example\n');

    // Create database instance
    const db = new IpToAsn(
        'https://iptoasn.com/data/ip2asn-combined.tsv.gz',
        './cache'
    );

    console.log('📦 Loading database...');
    await db.load();
    
    const stats = db.stats();
    console.log(`✅ Database loaded: ${stats.recordCount.toLocaleString()} records\n`);

    // Example lookups
    const testIps = [
        '8.8.8.8',        // Google DNS
        '1.1.1.1',        // Cloudflare DNS
        '208.67.222.222', // OpenDNS
        '192.0.2.1',      // Reserved (should not be found)
    ];

    console.log('🔍 Looking up IP addresses:\n');
    
    for (const ip of testIps) {
        const result = db.lookup(ip);
        
        if (result.announced) {
            console.log(`${ip}:`);
            console.log(`  AS${result.asNumber} - ${result.asDescription}`);
            console.log(`  Country: ${result.asCountryCode}`);
            console.log(`  Range: ${result.firstIp} - ${result.lastIp}\n`);
        } else {
            console.log(`${ip}: Not announced\n`);
        }
    }

    // Start auto-updates (check every 60 minutes)
    console.log('⏰ Starting auto-update (60 minute interval)...');
    await db.startAutoUpdate(60);
    
    console.log('\n✨ Database will automatically update in the background!');
    console.log('Press Ctrl+C to exit\n');

    // Keep the process running
    await new Promise(() => {});
}

main().catch(console.error);
