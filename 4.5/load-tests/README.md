# Load Testing with k6

This directory contains k6 scripts for load testing the Monero Marketplace application.

## Installation

To run these tests, you need to install k6. Follow the official k6 installation guide:

```bash
# For Debian/Ubuntu
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D65D7238B55E0A199AF3
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# For other OS, refer to: https://k6.io/docs/getting-started/installation/
```

## Running Tests

Navigate to the `4.5/load-tests` directory and run the desired script.

### HTTP Endpoints Test

This script tests various HTTP endpoints like listing retrieval and user registration.

```bash
k6 run scenarios/http-endpoints.js
```

### Escrow Flow Test

This script simulates a full escrow flow, including user registration, listing creation, and order placement.

```bash
k6 run scenarios/escrow-flow.js
```

## Generating HTML Reports

You can generate an HTML report from the test results using the `k6-html-reporter`.

1.  **Install `k6-html-reporter` (Node.js required):**
    ```bash
npm install -g k6-html-reporter
    ```

2.  **Run k6 test and output to JSON:**
    ```bash
k6 run scenarios/http-endpoints.js --out json=results.json
    ```

3.  **Generate HTML report:**
    ```bash
k6-html-reporter -f results.json -o report.html
    ```

This will create an `report.html` file in your current directory, which you can open in a web browser.
