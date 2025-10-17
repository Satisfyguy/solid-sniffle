//! Monero Marketplace CLI
//!
//! Command-line interface for the Monero Marketplace

mod checkpoint;

use anyhow::Result;
use clap::{Parser, Subcommand};
use monero_marketplace_common::{
    types::{MoneroConfig, WorkflowStep},
    MONERO_RPC_URL,
};
use monero_marketplace_wallet::MoneroClient;
use tracing::{error, info, warn};

/// Monero Marketplace CLI
#[derive(Parser)]
#[command(name = "monero-marketplace")]
#[command(about = "Monero Marketplace - Secure Escrow Platform")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Monero RPC URL
    #[arg(long, default_value = MONERO_RPC_URL)]
    rpc_url: String,

    /// RPC timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Get wallet status
    Status,
    /// Get complete wallet information
    Info,
    /// Multisig operations
    Multisig {
        #[command(subcommand)]
        command: MultisigCommands,
    },
    /// Manage checkpoints for multisig workflows
    Checkpoint {
        #[command(subcommand)]
        command: CheckpointCommands,
    },
    /// Test RPC connection
    Test,
}

#[derive(Subcommand)]
enum MultisigCommands {
    /// Prepare multisig (initial step)
    Prepare {
        /// Session ID for the workflow
        #[arg(long)]
        session: Option<String>,
    },
    /// Create the multisig wallet from prepared info
    Make {
        /// Session ID for the workflow
        #[arg(long)]
        session: Option<String>,
        /// Threshold (number of signatures required, e.g., 2 for 2-of-3)
        #[arg(short, long, default_value = "2")]
        threshold: u32,
        /// Multisig info from other participants
        #[arg(short, long)]
        info: Vec<String>,
    },
    /// Export multisig info for other participants
    Export {
        /// Session ID for the workflow
        #[arg(long)]
        session: Option<String>,
    },
    /// Import multisig info from other participants
    Import {
        /// Session ID for the workflow
        #[arg(long)]
        session: Option<String>,
        /// Multisig info to import
        #[arg(short, long)]
        info: Vec<String>,
    },
    /// Check if wallet is multisig
    Check,
}

#[derive(Subcommand)]
enum CheckpointCommands {
    /// List all saved checkpoints
    List,
    /// Show details of a specific checkpoint
    Show {
        /// The session ID of the checkpoint to show
        session_id: String,
    },
    /// Delete a specific checkpoint
    Delete {
        /// The session ID of the checkpoint to delete
        session_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Create Monero client
    let config = MoneroConfig {
        rpc_url: cli.rpc_url,
        rpc_user: None,
        rpc_password: None,
        timeout_seconds: cli.timeout,
    };

    let client = MoneroClient::new(config)?;

    // Execute command
    match cli.command {
        Commands::Status => {
            info!("Getting wallet status...");
            let status = client.get_wallet_status().await?;
            info!("Wallet Status:");
            info!("  Multisig: {}", status.is_multisig);
            if let Some(threshold) = status.multisig_threshold {
                if let Some(total) = status.multisig_total {
                    info!("  Threshold: {}/{}", threshold, total);
                } else {
                    info!("  Threshold: {}/?", threshold);
                }
            }
            info!("  Balance: {} XMR", status.balance as f64 / 1e12);
            info!("  Unlocked: {} XMR", status.unlocked_balance as f64 / 1e12);
        }

        Commands::Info => {
            info!("Getting complete wallet information...");
            let wallet_info = client.get_wallet_info().await?;
            info!("Wallet Information:");
            info!("  Version: {}", wallet_info.version);
            info!("  Balance: {} XMR", wallet_info.balance as f64 / 1e12);
            info!(
                "  Unlocked: {} XMR",
                wallet_info.unlocked_balance as f64 / 1e12
            );
            info!("  Multisig: {}", wallet_info.is_multisig);
            if let Some(threshold) = wallet_info.multisig_threshold {
                if let Some(total) = wallet_info.multisig_total {
                    info!("  Threshold: {}/{}", threshold, total);
                } else {
                    info!("  Threshold: {}/?", threshold);
                }
            }
            info!("  Block Height: {}", wallet_info.block_height);
            info!("  Daemon Block Height: {}", wallet_info.daemon_block_height);
        }

        Commands::Multisig { command } => match command {
            MultisigCommands::Prepare { session } => {
                if let Some(session_id) = session {
                    let mut checkpoint = checkpoint::load_checkpoint(&session_id)?;
                    info!("Preparing multisig for session '{}'...", session_id);

                    let result = client.multisig().prepare_multisig().await?;
                    info!("Multisig info: {}", result.multisig_info);

                    checkpoint.current_step = WorkflowStep::Prepared;
                    checkpoint.local_multisig_info = Some(result.multisig_info);
                    checkpoint::save_checkpoint(&checkpoint)?;
                    info!("Checkpoint '{}' saved.", session_id);
                } else {
                    info!("Preparing multisig (stateless)...");
                    let result = client.multisig().prepare_multisig().await?;
                    info!("Multisig info: {}", result.multisig_info);
                }
            }

            MultisigCommands::Make { session, threshold, mut info } => {
                if let Some(session_id) = session {
                    let mut checkpoint = checkpoint::load_checkpoint(&session_id)?;
                    info!("Making multisig for session '{}'...", session_id);

                    // Combine infos from command line and checkpoint
                    info.extend(checkpoint.remote_multisig_infos.clone());
                    info.dedup(); // Remove duplicates

                    let total_participants = info.len() + 1;
                    info!(
                        "Making {}-of-{} multisig with {} remote infos...",
                        threshold,
                        total_participants,
                        info.len()
                    );

                    let result = client.multisig().make_multisig(threshold, info.clone()).await?;
                    info!("Multisig address: {}", result.address);
                    info!("Multisig info for sync: {}", result.multisig_info);

                    checkpoint.current_step = WorkflowStep::Made;
                    checkpoint.multisig_address = Some(result.address);
                    checkpoint.local_multisig_info = Some(result.multisig_info);
                    checkpoint.remote_multisig_infos = info;
                    checkpoint.required_signatures = Some(threshold);
                    checkpoint::save_checkpoint(&checkpoint)?;
                    info!("Checkpoint '{}' saved.", session_id);

                } else {
                    info!(
                        "Making {}-of-{} multisig with {} infos...",
                        threshold,
                        info.len() + 1,
                        info.len()
                    );
                    let result = client.multisig().make_multisig(threshold, info).await?;
                    info!("Multisig address: {}", result.address);
                    info!("Multisig info: {}", result.multisig_info);
                }
            }

            MultisigCommands::Export { session } => {
                if let Some(session_id) = session {
                    let checkpoint = checkpoint::load_checkpoint(&session_id)?;
                    info!("Exporting multisig info for session '{}'...", session_id);
                    if let Some(info) = checkpoint.local_multisig_info {
                        info!("Multisig info: {}", info);
                    } else {
                        warn!("No local multisig info found in checkpoint. Run 'prepare' or 'make' first.");
                    }
                } else {
                    info!("Exporting multisig info (stateless)...");
                    let info = client.multisig().export_multisig_info().await?;
                    info!("Multisig info: {}", info.info);
                }
            }

            MultisigCommands::Import { session, info } => {
                 if let Some(session_id) = session {
                    let mut checkpoint = checkpoint::load_checkpoint(&session_id)?;
                    info!("Importing {} multisig info(s) for session '{}'...", info.len(), session_id);

                    let result = client.multisig().import_multisig_info(info.clone()).await?;
                    info!("Imported multisig info, {} outputs", result.n_outputs);

                    checkpoint.remote_multisig_infos.extend(info);
                    checkpoint.remote_multisig_infos.dedup();
                    // A simple heuristic to advance the step
                    if checkpoint.current_step == WorkflowStep::Made {
                        checkpoint.current_step = WorkflowStep::SyncedRound1;
                    } else if checkpoint.current_step == WorkflowStep::SyncedRound1 {
                        checkpoint.current_step = WorkflowStep::SyncedRound2;
                    }

                    checkpoint::save_checkpoint(&checkpoint)?;
                    info!("Checkpoint '{}' saved.", session_id);
                 } else {
                    info!("Importing {} multisig infos (stateless)...", info.len());
                    let result = client.multisig().import_multisig_info(info).await?;
                    info!("Imported multisig info, {} outputs", result.n_outputs);
                 }
            }

            MultisigCommands::Check => {
                let is_multisig = client.multisig().is_multisig().await?;
                info!("Wallet is multisig: {}", is_multisig);
            }
        },

        Commands::Checkpoint { command } => match command {
            CheckpointCommands::List => {
                info!("Listing all checkpoints...");
                let checkpoints = checkpoint::list_checkpoints()?;
                if checkpoints.is_empty() {
                    info!("No checkpoints found.");
                } else {
                    for cp in checkpoints {
                        info!("  - Session: {}, Step: {:?}, Updated: {}", cp.session_id, cp.current_step, cp.last_updated);
                    }
                }
            }
            CheckpointCommands::Show { session_id } => {
                info!("Showing details for checkpoint '{}'...", session_id);
                let checkpoint = checkpoint::load_checkpoint(&session_id)?;
                let pretty_json = serde_json::to_string_pretty(&checkpoint)?;
                info!("\n{}", pretty_json);
            }
            CheckpointCommands::Delete { session_id } => {
                info!("Deleting checkpoint '{}'...", session_id);
                checkpoint::delete_checkpoint(&session_id)?;
                info!("Checkpoint '{}' deleted.", session_id);
            }
        },

        Commands::Test => {
            info!("Testing RPC connection...");
            match client.rpc().get_version().await {
                Ok(version) => {
                    info!("✅ RPC connection successful");
                    info!("Monero version: {}", version);
                }
                Err(e) => {
                    error!("❌ RPC connection failed: {}", e);
                    return Err(anyhow::anyhow!("RPC connection failed: {}", e));
                }
            }
        }
    }

    Ok(())
}
