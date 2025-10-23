import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';

const users = new SharedArray('users', function () {
  return JSON.parse(open('./users.json')).users;
});

export const options = {
  vus: 10, // 10 virtual users
  duration: '5m', // for 5 minutes
  thresholds: {
    'http_req_duration': ['p95<500'], // 95% of requests must complete within 500ms
    'http_req_failed': ['rate<0.01'], // Error rate must be less than 1%
  },
};

export default function () {
  const user = users[__VU % users.length];
  const uniqueUsername = `${user.username}-${__VU}-${__ITER}`;
  const uniqueEmail = `${user.email.split('@')[0]}-${__VU}-${__ITER}@${user.email.split('@')[1]}`;

  let res;

  // 1. Register Buyer
  res = http.post(
    'http://localhost:8080/api/register',
    JSON.stringify({
      username: uniqueUsername,
      email: uniqueEmail,
      password: user.password,
    }),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { name: 'Register Buyer' },
    },
  );
  check(res, { 'Register Buyer status is 201 or 409': (r) => r.status === 201 || r.status === 409 });
  sleep(1);

  // Assuming successful registration, log in to get a token (simplified)
  // In a real scenario, you'd parse the login response for a token
  res = http.post(
    'http://localhost:8080/api/login',
    JSON.stringify({
      username: uniqueUsername,
      password: user.password,
    }),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { name: 'Login Buyer' },
    },
  );
  check(res, { 'Login Buyer status is 200': (r) => r.status === 200 });
  const authToken = res.json('token'); // Assuming token is returned directly
  sleep(1);

  // 2. Create Listing (as a vendor - for simplicity, using the same user for now)
  // In a real scenario, you'd have separate vendor users and login flow
  res = http.post(
    'http://localhost:8080/api/listings',
    JSON.stringify({
      title: `Test Listing ${__VU}-${__ITER}`,
      description: 'A test listing for load testing.',
      price: 0.1,
      currency: 'XMR',
    }),
    {
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${authToken}` },
      tags: { name: 'Create Listing' },
    },
  );
  check(res, { 'Create Listing status is 201': (r) => r.status === 201 });
  const listingId = res.json('id'); // Assuming listing ID is returned
  sleep(1);

  // 3. Create Order
  res = http.post(
    `http://localhost:8080/api/listings/${listingId}/order`,
    JSON.stringify({
      quantity: 1,
    }),
    {
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${authToken}` },
      tags: { name: 'Create Order' },
    },
  );
  check(res, { 'Create Order status is 201': (r) => r.status === 201 });
  sleep(1);
}
