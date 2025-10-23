/**
 * Performance Validation Load Test
 *
 * Purpose: Validate SLA performance targets under load
 * Targets:
 *   - p95 latency <200ms
 *   - p99 latency <500ms
 *   - Error rate <0.1%
 *   - Uptime 99.9%
 *
 * Test Scenarios:
 *   1. Baseline (100 VUs, 5 min)
 *   2. Load (500 VUs, 10 min)
 *   3. Stress (1000 VUs, 5 min)
 *   4. Spike (2000 VUs, 2 min)
 *
 * Run: k6 run --out json=performance-results.json scenarios/performance-validation.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom Metrics
const errorRate = new Rate('errors');
const apiLatency = new Trend('api_latency', true);
const requestCounter = new Counter('requests_total');

// SLA Thresholds
export const options = {
  scenarios: {
    baseline: {
      executor: 'constant-vus',
      vus: 100,
      duration: '5m',
      startTime: '0s',
      gracefulStop: '30s',
      tags: { scenario: 'baseline' },
    },
    load: {
      executor: 'ramping-vus',
      startVUs: 100,
      stages: [
        { duration: '2m', target: 300 },
        { duration: '5m', target: 500 },
        { duration: '3m', target: 100 },
      ],
      startTime: '5m',
      gracefulStop: '30s',
      tags: { scenario: 'load' },
    },
    stress: {
      executor: 'ramping-vus',
      startVUs: 500,
      stages: [
        { duration: '2m', target: 1000 },
        { duration: '3m', target: 1000 },
      ],
      startTime: '15m',
      gracefulStop: '30s',
      tags: { scenario: 'stress' },
    },
    spike: {
      executor: 'ramping-vus',
      startVUs: 100,
      stages: [
        { duration: '30s', target: 2000 },
        { duration: '1m', target: 2000 },
        { duration: '30s', target: 100 },
      ],
      startTime: '20m',
      gracefulStop: '30s',
      tags: { scenario: 'spike' },
    },
  },
  thresholds: {
    // SLA Requirements
    'http_req_duration{scenario:baseline}': ['p(95)<200', 'p(99)<500'],
    'http_req_duration{scenario:load}': ['p(95)<200', 'p(99)<500'],
    'http_req_duration{scenario:stress}': ['p(95)<300', 'p(99)<800'],  // Relaxed under stress
    'http_req_duration{scenario:spike}': ['p(95)<500', 'p(99)<1000'],   // Relaxed under spike

    // Error Rates
    'errors{scenario:baseline}': ['rate<0.001'],  // <0.1%
    'errors{scenario:load}': ['rate<0.001'],
    'errors{scenario:stress}': ['rate<0.01'],     // <1% acceptable under stress
    'errors{scenario:spike}': ['rate<0.05'],       // <5% acceptable under spike

    // Success Rates
    'http_req_failed{scenario:baseline}': ['rate<0.001'],
    'http_req_failed{scenario:load}': ['rate<0.001'],

    // Request Counts (ensure test runs)
    'requests_total': ['count>5000'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// Test user credentials (staging environment)
const TEST_USERS = [
  { username: 'test_buyer_1', password: 'test_password_123' },
  { username: 'test_vendor_1', password: 'test_password_456' },
  { username: 'test_arbiter_1', password: 'test_password_789' },
];

/**
 * Setup: Create test users if needed
 */
export function setup() {
  console.log('Performance Validation Test Starting...');
  console.log(`Base URL: ${BASE_URL}`);
  console.log(`Test Duration: ~22 minutes`);
  return { startTime: Date.now() };
}

/**
 * Main test function
 */
export default function (data) {
  const user = TEST_USERS[Math.floor(Math.random() * TEST_USERS.length)];

  // Test 1: Health Check (10% of requests)
  if (Math.random() < 0.1) {
    testHealthEndpoint();
  }

  // Test 2: Authentication (20% of requests)
  else if (Math.random() < 0.3) {
    testAuthentication(user);
  }

  // Test 3: Listings Browse (40% of requests)
  else if (Math.random() < 0.7) {
    testListingsBrowse();
  }

  // Test 4: Listing Details (20% of requests)
  else if (Math.random() < 0.9) {
    testListingDetails();
  }

  // Test 5: Search (10% of requests)
  else {
    testSearch();
  }

  // Random think time (1-3 seconds)
  sleep(Math.random() * 2 + 1);
}

/**
 * Test: Health Endpoint
 */
function testHealthEndpoint() {
  const startTime = Date.now();
  const res = http.get(`${BASE_URL}/api/health`);
  const duration = Date.now() - startTime;

  requestCounter.add(1);
  apiLatency.add(duration);

  const success = check(res, {
    'health: status is 200': (r) => r.status === 200,
    'health: response time <100ms': (r) => r.timings.duration < 100,
  });

  errorRate.add(!success);
}

/**
 * Test: Authentication Flow
 */
function testAuthentication(user) {
  const payload = JSON.stringify({
    username: user.username,
    password: user.password,
  });

  const params = {
    headers: { 'Content-Type': 'application/json' },
  };

  const startTime = Date.now();
  const res = http.post(`${BASE_URL}/api/auth/login`, payload, params);
  const duration = Date.now() - startTime;

  requestCounter.add(1);
  apiLatency.add(duration);

  const success = check(res, {
    'auth: status is 200 or 401': (r) => [200, 401].includes(r.status),
    'auth: response time <300ms': (r) => r.timings.duration < 300,
    'auth: has valid response': (r) => r.body.length > 0,
  });

  errorRate.add(!success);
}

/**
 * Test: Browse Listings
 */
function testListingsBrowse() {
  const startTime = Date.now();
  const res = http.get(`${BASE_URL}/api/listings`);
  const duration = Date.now() - startTime;

  requestCounter.add(1);
  apiLatency.add(duration);

  const success = check(res, {
    'listings: status is 200': (r) => r.status === 200,
    'listings: response time <200ms': (r) => r.timings.duration < 200,
    'listings: has listings array': (r) => {
      try {
        const body = JSON.parse(r.body);
        return Array.isArray(body) || Array.isArray(body.listings);
      } catch {
        return false;
      }
    },
  });

  errorRate.add(!success);
}

/**
 * Test: Listing Details
 */
function testListingDetails() {
  // Simulate clicking on a random listing
  const listingId = `listing-${Math.floor(Math.random() * 100)}`;

  const startTime = Date.now();
  const res = http.get(`${BASE_URL}/api/listings/${listingId}`);
  const duration = Date.now() - startTime;

  requestCounter.add(1);
  apiLatency.add(duration);

  const success = check(res, {
    'listing detail: status is 200 or 404': (r) => [200, 404].includes(r.status),
    'listing detail: response time <300ms': (r) => r.timings.duration < 300,
  });

  errorRate.add(!success);
}

/**
 * Test: Search
 */
function testSearch() {
  const searchTerms = ['laptop', 'phone', 'book', 'camera', 'watch'];
  const term = searchTerms[Math.floor(Math.random() * searchTerms.length)];

  const startTime = Date.now();
  const res = http.get(`${BASE_URL}/api/listings/search?q=${term}`);
  const duration = Date.now() - startTime;

  requestCounter.add(1);
  apiLatency.add(duration);

  const success = check(res, {
    'search: status is 200': (r) => r.status === 200,
    'search: response time <500ms': (r) => r.timings.duration < 500,
  });

  errorRate.add(!success);
}

/**
 * Teardown: Summary Report
 */
export function teardown(data) {
  const endTime = Date.now();
  const duration = (endTime - data.startTime) / 1000;

  console.log('\n========================================');
  console.log('Performance Validation Test Complete');
  console.log('========================================');
  console.log(`Total Duration: ${duration.toFixed(2)}s`);
  console.log('\nThreshold Checks:');
  console.log('  - p95 latency <200ms (baseline/load)');
  console.log('  - p99 latency <500ms (baseline/load)');
  console.log('  - Error rate <0.1% (baseline/load)');
  console.log('\nResults: See k6 output above');
  console.log('========================================\n');
}
