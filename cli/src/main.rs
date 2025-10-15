//! Monero Marketplace CLI
//! 
//! Command-line interface for the Monero Marketplace

use clap::{Parser, Subcommand};
use monero_marketplace_common::{
    error::Result,
    types::MoneroConfig,
};
use monero_marketplace_wallet::MoneroClient;
use tracing::{info, error};

/// Monero Marketplace CLI
#[derive(Parser)]
#[command(name = "monero-marketplace")]
#[command(about = "Monero Marketplace - Secure Escrow Platform")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Monero RPC URL
    #[arg(long, default_value = "http://127.0.0.1:18082/json_rpc")]
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
            println!("Wallet Status:");
            println!("  Multisig: {}", status.is_multisig);
            if let Some(threshold) = status.multisig_threshold {
                if let Some(total) = status.multisig_total {
                    println!("  Threshold: {}/{}", threshold, total);
                } else {
                    println!("  Threshold: {}/?", threshold);
                }
            }
            println!("  Balance: {} XMR", status.balance as f64 / 1e12);
            println!("  Unlocked: {} XMR", status.unlocked_balance as f64 / 1e12);
        }
        
        Commands::Info => {
            info!("Getting complete wallet information...");
            let wallet_info = client.get_wallet_info().await?;
            println!("Wallet Information:");
            println!("  Version: {}", wallet_info.version);
            println!("  Balance: {} XMR", wallet_info.balance as f64 / 1e12);
            println!("  Unlocked: {} XMR", wallet_info.unlocked_balance as f64 / 1e12);
            println!("  Multisig: {}", wallet_info.is_multisig);
            if let Some(threshold) = wallet_info.multisig_threshold {
                if let Some(total) = wallet_info.multisig_total {
                    println!("  Threshold: {}/{}", threshold, total);
                } else {
                    println!("  Threshold: {}/?", threshold);
                }
            }
            println!("  Block Height: {}", wallet_info.block_height);
            println!("  Daemon Block Height: {}", wallet_info.daemon_block_height);
        }
        
        Commands::Multisig { command } => {
            match command {
                MultisigCommands::Prepare => {
                    info!("Preparing multisig...");
                    let info = client.multisig().prepare_multisig().await?;
                    println!("Multisig info: {}", info.info);
                }
                
                MultisigCommands::Make { info } => {
                    info!("Making multisig with {} infos...", info.len());
                    let result = client.multisig().make_multisig(info).await?;
                    println!("Multisig info: {}", result.info);
                }
                
                MultisigCommands::Export => {
                    info!("Exporting multisig info...");
                    let info = client.multisig().export_multisig_info().await?;
                    println!("Multisig info: {}", info.info);
                }
                
                MultisigCommands::Import { info } => {
                    info!("Importing {} multisig infos...", info.len());
                    let outputs = client.multisig().import_multisig_info(info).await?;
                    println!("Imported multisig info, {} outputs", outputs);
                }
                
                MultisigCommands::Check => {
                    let is_multisig = client.multisig().is_multisig().await?;
                    println!("Wallet is multisig: {}", is_multisig);
                }
            }
        }
        
        Commands::Test => {
            info!("Testing RPC connection...");
            match client.rpc().get_version().await {
                Ok(version) => {
                    println!("✅ RPC connection successful");
                    println!("Monero version: {}", version);
                }
                Err(e) => {
                    error!("❌ RPC connection failed: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}
