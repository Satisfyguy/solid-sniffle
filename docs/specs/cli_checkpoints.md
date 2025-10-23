# Spec: CLI Checkpoints

**Version:** 1.0
**Status:** Proposed
**Date:** 2025-10-17

## 1. Overview

This document specifies the implementation of a checkpoint system for the `monero-marketplace-cli`. This feature will allow users to save and resume multi-step workflows, particularly the multisig wallet creation and transaction signing processes. This enhances usability and resilience by preventing the loss of progress during complex operations that require input from multiple parties over time.

## 2. Goals

-   Allow users to pause and resume multisig workflows using a simple session identifier.
-   Persist the state of a workflow to the filesystem in a human-readable format (JSON).
-   Provide commands to manage these saved states (checkpoints).
-   Ensure the system is intuitive and integrates smoothly with existing commands without breaking the current stateless behavior.

## 3. Technical Design

### 3.1. Checkpoint Directory

A new directory named `.checkpoints/` will be created in the project root. This directory will store all checkpoint files. It should be added to the project's `.gitignore` file to prevent accidental commits of user session data.

### 3.2. Checkpoint File Format

Each checkpoint will be a single JSON file named after a user-defined `Session ID`. For example, a session named `my-first-escrow` will have its state saved in `.checkpoints/my-first-escrow.json`.

### 3.3. Checkpoint Data Structure

A `Checkpoint` struct will be defined in `common/src/types.rs` to represent the state. This ensures the structure is available to all crates if needed.

```rust
// in common/src/types.rs

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WorkflowStep {
    Initiated,
    Prepared,
    Made,
    SyncedRound1,
    SyncedRound2,
    Ready,
    // Transaction-related steps
    TxCreationStarted,
    TxCreated,
    TxSigned,
    TxFinalized,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Checkpoint {
    pub session_id: String,
    pub current_step: WorkflowStep,
    pub last_updated: String, // ISO 8601 timestamp
    pub multisig_address: Option<String>,
    pub required_signatures: Option<u32>,
    // Stores this wallet's own generated multisig info/keys
    pub local_multisig_info: Option<String>,
    // Stores multisig info received from other participants
    pub remote_multisig_infos: Vec<String>,
    // Stores data related to a transaction being created/signed
    pub transaction_data: Option<TransactionCheckpointData>,
    // Generic key-value store for future use or user notes
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionCheckpointData {
    pub unsigned_tx_set: Option<String>,
    pub collected_signatures: Vec<String>,
    pub tx_hash: Option<String>,
}

impl Checkpoint {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            current_step: WorkflowStep::Initiated,
            last_updated: chrono::Utc::now().to_rfc3339(),
            multisig_address: None,
            required_signatures: None,
            local_multisig_info: None,
            remote_multisig_infos: Vec::new(),
            transaction_data: None,
            metadata: HashMap::new(),
        }
    }
}
```

### 3.4. CLI Command Modifications

The following commands in the `cli` crate will be modified to accept a new optional argument: `--session <SESSION_ID>`.

-   `monero-marketplace multisig prepare --session <ID>`
-   `monero-marketplace multisig make --session <ID> ...`
-   `monero-marketplace multisig export --session <ID>`
-   `monero-marketplace multisig import --session <ID> ...`

**Behavior:**
1.  If `--session` is used and a checkpoint file exists, the command will load the state from that file.
2.  The command will perform its action based on the loaded state (e.g., `make` will use the `local_multisig_info` from the checkpoint).
3.  Upon successful completion, the command will update the checkpoint file with the new state (e.g., advancing `current_step`, adding new data, updating `last_updated`).
4.  If `--session` is used and no file exists, a new checkpoint file will be created with the initial state.
5.  If `--session` is NOT used, the commands will behave as they currently do (stateless operation), ensuring backward compatibility.

### 3.5. New `checkpoint` Command

A new top-level command will be added: `monero-marketplace checkpoint`.

**Subcommands:**

-   `checkpoint list`
    -   **Action:** Lists all available session IDs from the `.checkpoints/` directory.
    -   **Output:** A table with `Session ID`, `Current Step`, and `Last Updated`.

-   `checkpoint show <SESSION_ID>`
    -   **Action:** Displays the detailed content of a specific checkpoint file in a human-readable, formatted JSON.

-   `checkpoint delete <SESSION_ID>`
    -   **Action:** Deletes a specified checkpoint file after a confirmation prompt to prevent accidental data loss.

## 4. Implementation Plan

1.  **Update `.gitignore`:** Add `.checkpoints/` to the `.gitignore` file.
2.  **Implement `Checkpoint` Structs:** Add the `Checkpoint`, `WorkflowStep`, and `TransactionCheckpointData` structs to `common/src/types.rs`. The `chrono` crate will need to be added as a dependency to `common` for timestamping.
3.  **Create Checkpoint Manager:** Create a new module `cli/src/checkpoint.rs`. This module will contain the core logic for file operations:
    -   `load_checkpoint(session_id: &str) -> Result<Checkpoint>`
    -   `save_checkpoint(checkpoint: &Checkpoint) -> Result<()>`
    -   `list_checkpoints() -> Result<Vec<Checkpoint>>`
    -   `delete_checkpoint(session_id: &str) -> Result<()>`
4.  **Modify `cli/src/main.rs`:**
    -   Add the `--session` argument to the relevant `multisig` subcommands.
    -   Update the handlers for these commands to call the checkpoint manager to load/save state. The logic will branch based on the presence of the `--session` argument.
    -   Add the new `checkpoint` command and its subcommands (`list`, `show`, `delete`).
5.  **Write Tests:** Add unit tests for the checkpoint manager (e.g., test loading/saving valid and invalid files) and integration tests for the CLI commands using the `--session` flag to verify state transitions.
