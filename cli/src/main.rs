//! Monero Marketplace CLI
//!
//! Command-line interface for the Monero Marketplace

use clap::{Parser, Subcommand};
use monero_marketplace_common::{types::MoneroConfig, MONERO_RPC_URL};
use monero_marketplace_wallet::MoneroClient;
use tracing::{error, info};

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
    /// Test RPC connection
    Test,
}

#[derive(Subcommand)]
enum MultisigCommands {
    /// Prepare multisig
    Prepare,
    /// Make multisig
    Make {
        /// Threshold (number of signatures required, e.g., 2 for 2-of-3)
        #[arg(short, long, default_value = "2")]
        threshold: u32,
        /// Multisig info from other participants
        #[arg(short, long)]
        info: Vec<String>,
    },
    /// Export multisig info
    Export,
    /// Import multisig info
    Import {
        /// Multisig info to import
        #[arg(short, long)]
        info: Vec<String>,
    },
    /// Check if wallet is multisig
    Check,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
            MultisigCommands::Prepare => {
                info!("Preparing multisig...");
                let result = client.multisig().prepare_multisig().await?;
                info!("Multisig info: {}", result.multisig_info);
            }

            MultisigCommands::Make { threshold, info } => {
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

            MultisigCommands::Export => {
                info!("Exporting multisig info...");
                let info = client.multisig().export_multisig_info().await?;
                info!("Multisig info: {}", info.info);
            }

            MultisigCommands::Import { info } => {
                info!("Importing {} multisig infos...", info.len());
                let result = client.multisig().import_multisig_info(info).await?;
                info!("Imported multisig info, {} outputs", result.n_outputs);
            }

            MultisigCommands::Check => {
                let is_multisig = client.multisig().is_multisig().await?;
                info!("Wallet is multisig: {}", is_multisig);
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
