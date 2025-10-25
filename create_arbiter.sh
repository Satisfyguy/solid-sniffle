#!/bin/bash
# Script to create an arbiter user via the registration endpoint

ARBITER_USERNAME="arbiter_system"
ARBITER_PASSWORD="arbiter_secure_password_2024"

echo "Creating arbiter user..."

curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$ARBITER_USERNAME\",
    \"password\": \"$ARBITER_PASSWORD\",
    \"role\": \"arbiter\"
  }"

echo ""
echo "Arbiter user created!"
echo "Username: $ARBITER_USERNAME"
echo "Password: $ARBITER_PASSWORD"
