#!/usr/bin/env node

/**
 * Load Testing Script for Flinderax Backend
 * 
 * This script tests the backend's capacity to handle concurrent requests
 * and measures RPS (requests per second) for different endpoints.
 * 
 * Prerequisites:
 *   npm install autocannon
 * 
 * Usage:
 *   node load-test.js [local|production]
 */

const autocannon = require('autocannon');
const http = require('http');
const https = require('https');
const { URL } = require('url');

// Configuration
const ENV = process.argv[2] || 'local';
const BASE_URL = ENV === 'production'
    ? 'https://flinderax-backend.fly.dev'
    : 'http://localhost:3000';

// Test credentials (make sure this user exists in your database)
const TEST_USER = {
    user_name: 'loadtest',
    user_password: 'LoadTest123!'
};

// Multi-user configuration
const MULTI_USER_CONFIG = {
    enabled: process.argv[3] === '--multi-user',
    userCount: parseInt(process.argv[4]) || 10,
    userPrefix: 'loadtest_user_',
    userPassword: 'LoadTest123!'
};


// ANSI colors for output
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m',
};

function printHeader(text) {
    console.log(`\n${colors.bright}${colors.cyan}${'='.repeat(60)}${colors.reset}`);
    console.log(`${colors.bright}${colors.cyan}${text}${colors.reset}`);
    console.log(`${colors.bright}${colors.cyan}${'='.repeat(60)}${colors.reset}\n`);
}

function printResults(result) {
    console.log(`${colors.green}Results:${colors.reset}`);
    console.log(`  Requests:          ${result.requests.total} (${result.requests.average}/sec)`);
    console.log(`  Throughput:        ${(result.throughput.average / 1024 / 1024).toFixed(2)} MB/sec`);
    console.log(`  Latency (avg):     ${result.latency.mean.toFixed(2)} ms`);
    console.log(`  Latency (p99):     ${result.latency.p99.toFixed(2)} ms`);
    console.log(`  Errors:            ${result.errors}`);
    console.log(`  Timeouts:          ${result.timeouts}`);
    console.log(`  2xx responses:     ${result['2xx']}`);
    console.log(`  Non-2xx responses: ${result.non2xx || 0}`);
}

async function getAuthToken() {
    printHeader('Step 1: Getting Authentication Token');

    const url = new URL(`${BASE_URL}/common/login`);
    const client = url.protocol === 'https:' ? https : http;

    const postData = JSON.stringify(TEST_USER);

    const options = {
        hostname: url.hostname,
        port: url.port || (url.protocol === 'https:' ? 443 : 80),
        path: url.pathname,
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(postData)
        }
    };

    return new Promise((resolve, reject) => {
        const req = client.request(options, (res) => {
            let data = '';
            res.on('data', (chunk) => { data += chunk; });
            res.on('end', () => {
                if (res.statusCode === 200) {
                    try {
                        const body = JSON.parse(data);
                        if (body.access_token) {
                            console.log(`${colors.green}✓ Login successful. Token obtained.${colors.reset}`);
                            resolve(body.access_token);
                        } else {
                            reject(new Error('Login response missing access_token'));
                        }
                    } catch (e) {
                        reject(new Error('Failed to parse login response'));
                    }
                } else {
                    reject(new Error(`Login failed with status ${res.statusCode}: ${data}`));
                }
            });
        });

        req.on('error', (e) => {
            reject(new Error(`Login request error: ${e.message}`));
        });

        req.write(postData);
        req.end();
    });
}

async function createTestUsers(count) {
    printHeader(`Creating ${count} Test Users`);

    const users = [];
    const timestamp = Date.now();
    const url = new URL(`${BASE_URL}/common/register`);
    const client = url.protocol === 'https:' ? https : http;

    for (let i = 0; i < count; i++) {
        const username = `${MULTI_USER_CONFIG.userPrefix}${timestamp}_${i}`;
        const userData = {
            user_name: username,
            user_password: MULTI_USER_CONFIG.userPassword
        };

        const postData = JSON.stringify(userData);
        const options = {
            hostname: url.hostname,
            port: url.port || (url.protocol === 'https:' ? 443 : 80),
            path: url.pathname,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(postData)
            }
        };

        try {
            const token = await new Promise((resolve, reject) => {
                const req = client.request(options, (res) => {
                    let data = '';
                    res.on('data', (chunk) => { data += chunk; });
                    res.on('end', () => {
                        if (res.statusCode === 200) {
                            try {
                                const body = JSON.parse(data);
                                if (body.access_token) {
                                    resolve(body.access_token);
                                } else {
                                    reject(new Error('Registration response missing access_token'));
                                }
                            } catch (e) {
                                reject(new Error('Failed to parse registration response'));
                            }
                        } else {
                            reject(new Error(`Registration failed with status ${res.statusCode}: ${data}`));
                        }
                    });
                });

                req.on('error', (e) => {
                    reject(new Error(`Registration request error: ${e.message}`));
                });

                req.write(postData);
                req.end();
            });

            users.push({ username, password: MULTI_USER_CONFIG.userPassword, token });
            process.stdout.write(`${colors.green}✓${colors.reset} Created user ${i + 1}/${count}\r`);
        } catch (error) {
            console.log(`${colors.yellow}✗ Failed to create user ${username}: ${error.message}${colors.reset}`);
        }
    }

    console.log(`\n${colors.green}✓ Successfully created ${users.length}/${count} test users${colors.reset}\n`);
    return users;
}

async function deleteTestUsers(users) {
    printHeader(`Cleaning Up ${users.length} Test Users`);

    const url = new URL(`${BASE_URL}/user/delete`);
    const client = url.protocol === 'https:' ? https : http;
    let deletedCount = 0;

    for (let i = 0; i < users.length; i++) {
        const user = users[i];
        const postData = JSON.stringify({ user_name: user.username });
        const options = {
            hostname: url.hostname,
            port: url.port || (url.protocol === 'https:' ? 443 : 80),
            path: url.pathname,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(postData),
                'Authorization': `Bearer ${user.token}`
            }
        };

        try {
            await new Promise((resolve, reject) => {
                const req = client.request(options, (res) => {
                    let data = '';
                    res.on('data', (chunk) => { data += chunk; });
                    res.on('end', () => {
                        if (res.statusCode === 200 || res.statusCode === 404) {
                            deletedCount++;
                            resolve();
                        } else {
                            reject(new Error(`Delete failed with status ${res.statusCode}`));
                        }
                    });
                });

                req.on('error', (e) => {
                    reject(new Error(`Delete request error: ${e.message}`));
                });

                req.write(postData);
                req.end();
            });

            process.stdout.write(`${colors.green}✓${colors.reset} Deleted user ${i + 1}/${users.length}\r`);
        } catch (error) {
            console.log(`${colors.yellow}✗ Failed to delete user ${user.username}: ${error.message}${colors.reset}`);
        }
    }

    console.log(`\n${colors.green}✓ Successfully deleted ${deletedCount}/${users.length} test users${colors.reset}\n`);
}

async function testLoginEndpoint() {
    printHeader('Test 1: Login Endpoint (CPU-Intensive - Argon2 Hashing)');
    console.log(`Target: ${BASE_URL}/common/login`);
    console.log(`Expected: 5-20 RPS (CPU-bound due to password hashing)\n`);

    const instance = autocannon({
        url: `${BASE_URL}/common/login`,
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(TEST_USER),
        connections: 10,
        duration: 10,
    });

    return new Promise((resolve) => {
        autocannon.track(instance, { renderProgressBar: true });
        instance.on('done', (result) => {
            printResults(result);
            resolve(result);
        });
    });
}

async function testCachedRead(tokens) {
    printHeader('Test 2: Get All Cards (Cached Read - Redis)');
    console.log(`Target: ${BASE_URL}/card/get_all_cards`);
    console.log(`Expected: 200-1000 RPS (should hit Redis cache)`);

    const tokenArray = Array.isArray(tokens) ? tokens : [tokens];
    console.log(`Using ${tokenArray.length} user token(s)\n`);

    let tokenIndex = 0;
    const instance = autocannon({
        url: `${BASE_URL}/card/get_all_cards`,
        method: 'GET',
        setupClient: (client) => {
            client.setHeaders({
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${tokenArray[tokenIndex++ % tokenArray.length]}`
            });
        },
        connections: 50,
        duration: 10,
    });

    return new Promise((resolve) => {
        autocannon.track(instance, { renderProgressBar: true });
        instance.on('done', (result) => {
            printResults(result);
            resolve(result);
        });
    });
}


async function testDatabaseRead(tokens) {
    printHeader('Test 3: Get Single Card (Database Read)');
    console.log(`Target: ${BASE_URL}/card/get_card`);
    console.log(`Expected: 100-300 RPS (SQLite read)`);

    const tokenArray = Array.isArray(tokens) ? tokens : [tokens];
    console.log(`Using ${tokenArray.length} user token(s)\n`);

    let tokenIndex = 0;
    const instance = autocannon({
        url: `${BASE_URL}/card/get_card`,
        method: 'POST',
        setupClient: (client) => {
            client.setHeaders({
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${tokenArray[tokenIndex++ % tokenArray.length]}`
            });
            client.setBody(JSON.stringify({ card_id: 'test-card-id' }));
        },
        connections: 30,
        duration: 10,
    });

    return new Promise((resolve) => {
        autocannon.track(instance, { renderProgressBar: true });
        instance.on('done', (result) => {
            printResults(result);
            resolve(result);
        });
    });
}

async function testHealthCheck() {
    printHeader('Test 4: Health Check (Minimal Load)');
    console.log(`Target: ${BASE_URL}/`);
    console.log(`Expected: 1000+ RPS (no database, no auth)\n`);

    const instance = autocannon({
        url: `${BASE_URL}/`,
        method: 'GET',
        connections: 100,
        duration: 10,
    });

    return new Promise((resolve) => {
        autocannon.track(instance, { renderProgressBar: true });
        instance.on('done', (result) => {
            printResults(result);
            resolve(result);
        });
    });
}

function printSummary(results) {
    printHeader('LOAD TEST SUMMARY');

    console.log(`${colors.bright}Environment:${colors.reset} ${ENV} (${BASE_URL})`);
    console.log(`${colors.bright}Date:${colors.reset} ${new Date().toISOString()}\n`);

    console.log(`${colors.yellow}Performance Breakdown:${colors.reset}`);
    results.forEach((result, index) => {
        const testNames = [
            'Login (Argon2 Hashing)',
            'Cached Reads (Redis)',
            'Database Reads (SQLite)',
            'Health Check'
        ];
        console.log(`\n  ${index + 1}. ${testNames[index]}`);
        console.log(`     RPS:        ${result.requests.average.toFixed(0)}`);
        console.log(`     Latency:    ${result.latency.mean.toFixed(2)} ms (p99: ${result.latency.p99.toFixed(2)} ms)`);
        console.log(`     Error Rate: ${((result.errors / result.requests.total) * 100 || 0).toFixed(2)}%`);
    });

    console.log(`\n${colors.bright}${colors.green}Recommendations:${colors.reset}`);
    const cachedRPS = results[1]?.requests.average || 0;
    const dbRPS = results[2]?.requests.average || 0;
    const healthRPS = results[3]?.requests.average || 0;

    if (cachedRPS < 200) {
        console.log(`  ${colors.yellow}⚠${colors.reset} Cached read RPS is low. Check Redis connection.`);
    }
    if (dbRPS < 100) {
        console.log(`  ${colors.yellow}⚠${colors.reset} Database read RPS is low. Consider SQLite optimizations.`);
    }
    if (healthRPS < 500) {
        console.log(`  ${colors.yellow}⚠${colors.reset} Basic endpoint RPS is low. Check server resources.`);
    }

    console.log(`\n${colors.cyan}Note: These tests simulate load from a single machine.${colors.reset}`);
    console.log(`${colors.cyan}Real-world distributed load may show different results.${colors.reset}\n`);
}

async function main() {
    try {
        console.log(`${colors.bright}Flinderax Backend Load Test${colors.reset}`);
        console.log(`Testing against: ${colors.blue}${BASE_URL}${colors.reset}`);

        if (MULTI_USER_CONFIG.enabled) {
            console.log(`${colors.yellow}Multi-user mode: ${MULTI_USER_CONFIG.userCount} concurrent users${colors.reset}\n`);
        } else {
            console.log(`${colors.cyan}Single-user mode (use --multi-user <count> for multi-user simulation)${colors.reset}\n`);
        }

        let users = [];
        let tokens = [];

        if (MULTI_USER_CONFIG.enabled) {
            // Create multiple test users
            users = await createTestUsers(MULTI_USER_CONFIG.userCount);
            tokens = users.map(u => u.token);

            if (tokens.length === 0) {
                throw new Error('Failed to create any test users');
            }
        } else {
            // Use single test user
            const token = await getAuthToken();
            tokens = [token];
        }

        // Run all tests
        const results = [];
        results.push(await testLoginEndpoint());
        results.push(await testCachedRead(tokens));
        results.push(await testDatabaseRead(tokens));
        results.push(await testHealthCheck());

        // Clean up test users if in multi-user mode
        if (MULTI_USER_CONFIG.enabled && users.length > 0) {
            await deleteTestUsers(users);
        }

        // Print summary
        printSummary(results);

    } catch (error) {
        console.error(`${colors.yellow}Error during load test:${colors.reset}`, error.message);
        process.exit(1);
    }
}

// Check if autocannon is installed
try {
    require.resolve('autocannon');
} catch (e) {
    console.error(`${colors.yellow}Error: autocannon is not installed${colors.reset}`);
    console.log(`\nPlease install it with: ${colors.green}npm install autocannon${colors.reset}\n`);
    process.exit(1);
}

main();
