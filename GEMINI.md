# Gemini Context: Monero Marketplace

This document provides a comprehensive overview of the Monero Marketplace project, its architecture, and development conventions to be used as a guide for AI-assisted development.

## 1. Project Overview

This project is a decentralized marketplace that uses a 2-of-3 Monero multisignature escrow system and operates as a Tor hidden service for anonymity. The primary goal is to create a secure and private trading platform.

The project places a very strong emphasis on **Operational Security (OPSEC)** and avoiding "security theatre." All network traffic is routed through Tor, and the Monero RPC wallet is strictly isolated to `localhost`.

**Key Technologies:**
*   **Language:** Rust (stable, 2021 edition)
*   **Anonymity Network:** Tor (SOCKS5 proxy)
*   **Cryptocurrency:** Monero (via local RPC wallet)
*   **Core Libraries:**
    *   `tokio`: Asynchronous runtime
    *   `reqwest`: HTTP client (for Tor and Monero RPC)
    *   `serde`: Data serialization/deserialization
    *   `tracing`: Structured logging
    *   `clap`: Command-line argument parsing
    *   `anyhow` & `thiserror`: Robust error handling

## 2. Project Structure

The project is organized as a Rust workspace with three main crates:

*   `C:\Users\Lenovo\monero-marketplace\common\`: Contains shared code, including error types (`MoneroError`, `TorError`), data structures (`MultisigInfo`), and constants.
*   `C:\Users\Lenovo\monero-marketplace\wallet\`: Implements the core logic for interacting with the Monero wallet RPC and managing Tor connections.
*   `C:\Users\Lenovo\monero-marketplace\cli\`: Provides a command-line interface for the marketplace (currently a work in progress).

## 3. Building and Running

The project uses PowerShell scripts for various development and maintenance tasks. All commands should be run from the project root directory (`C:\Users\Lenovo\monero-marketplace`).

**Prerequisites:**
*   Rust toolchain
*   Tor (daemon or Tor Browser running)
*   Monero CLI tools (for testnet)

**Common Commands:**

*   **Build the project:**
    ```powershell
    cargo build
    ```

*   **Run all tests:**
    ```powershell
    cargo test --workspace
    ```

*   **Setup Monero testnet:**
    *   This script automates the setup of a local Monero testnet environment.
    ```powershell
    .\scripts\setup-monero-testnet.ps1
    ```

*   **Check project status and security metrics:**
    ```powershell
    .\scripts\metrics-dashboard.ps1
    ```

## 4. Development Conventions

The project enforces a strict, security-first development workflow. Adherence to these rules is mandatory.

### 4.1. Core Workflow

Development follows a "spec-first" process:

1.  **Create a Spec:** Before writing any code, create a specification markdown file.
    ```powershell
    .\scripts\new-spec.ps1 <function-name>
    ```
2.  **Implement the Code:** Write the Rust implementation based on the spec.
3.  **Perform a Reality Check:** For any network-related function, generate and validate a "Reality Check" to document and test its behavior over Tor.
    ```powershell
    # Generate the check
    .\scripts\auto-reality-check-tor.ps1 <function-name>

    # Validate the check
    .\scripts\validate-reality-check-tor.ps1 <function-name>
    ```

### 4.2. "Anti-Security Theatre" Coding Rules

These rules are strictly enforced by pre-commit hooks and CI pipelines.

*   **NO `.unwrap()`:** Never use `.unwrap()`. Use `anyhow::Context` to provide context for potential errors.
    ```rust
    // Correct
    use anyhow::Context;
    let data = some_operation().context("Descriptive error message")?;
    ```

*   **NO `println!`:** Do not use `println!` for logging. Use the `tracing` library.
    ```rust
    // Correct
    use tracing::{info, warn, error};
    info!("Starting operation...");
    ```

*   **NO Magic Numbers:** Define constants for literal values.
    ```rust
    // Correct
    const XMR_TO_ATOMIC: u64 = 1_000_000_000_000;
    ```

*   **NO Placeholders:** Do not commit code with `TODO`, `FIXME`, or similar placeholders. Implement the functionality fully.

*   **NO Hardcoded Secrets:** Load secrets and configuration from environment variables or a configuration file.

### 4.3. OPSEC Rules

*   **Isolate RPC:** The Monero wallet RPC must only bind to `127.0.0.1`.
*   **Route via Tor:** All external network calls must be proxied through the Tor SOCKS5 port (`127.0.0.1:9050`).
*   **No Sensitive Logs:** Never log sensitive data like `.onion` addresses, private keys, or IP addresses.
