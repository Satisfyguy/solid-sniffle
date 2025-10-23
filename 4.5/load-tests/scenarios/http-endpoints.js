import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';

// Load test data (e.g., user credentials for registration)
const users = new SharedArray('users', function () {
  return JSON.parse(open('./users.json')).users;
});

export const options = {
  stages: [
    { duration: '5m', target: 10 },  // Ramp-up to 10 users over 5 minutes
    { duration: '10m', target: 50 }, // Stay at 50 users for 10 minutes
    { duration: '5m', target: 100 }, // Ramp-up to 100 users over 5 minutes
  ],
  thresholds: {
    'http_req_duration': ['p95<200'], // 95% of requests must complete within 200ms
    'http_req_failed': ['rate<0.05'], // Error rate must be less than 5%
  },
  ext: {
    influxdb: {
      address: 'http://localhost:8086/k6', // InfluxDB URL
      username: 'k6',
      password: 'k6',
    },
  },
};

export default function () {
  // Test: GET /api/listings
  let res = http.get('http://localhost:8080/api/listings');
  check(res, { 'GET /api/listings status is 200': (r) => r.status === 200 });
  sleep(1);

  // Test: Search (example)
  res = http.get('http://localhost:8080/api/listings?q=monero');
  check(res, { 'GET /api/listings?q=monero status is 200': (r) => r.status === 200 });
  sleep(1);

  // Test: Register user (example - assuming a /register endpoint)
  // Note: In a real scenario, you'd want to ensure unique users or handle existing ones.
  const user = users[__VU % users.length];
  res = http.post(
    'http://localhost:8080/api/register',
    JSON.stringify({
      username: user.username + __VU,
      email: `user${__VU}@example.com`,
      password: user.password,
    }),
    {
      headers: { 'Content-Type': 'application/json' },
    },
  );
  check(res, { 'POST /api/register status is 201': (r) => r.status === 201 || r.status === 409 }); // 409 for already exists
  sleep(1);
}
