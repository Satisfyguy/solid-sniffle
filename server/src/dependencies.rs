//! Module de gestion des dépendances externes
//! 
//! Ce module s'occupe de vérifier et de démarrer automatiquement les services
//! dépendants (Monero daemon et wallet RPCs) avant que le serveur ne commence
//! à traiter les requêtes.

use anyhow::{Context, Result};
use std::process::Command;
use tokio::time::{sleep, Duration};

/// Vérifie si un processus est en cours d'exécution
fn is_process_running(process_name: &str) -> bool {
    let output = Command::new("pgrep")
        .args(["-f", process_name])
        .output()
        .ok();

    match output {
        Some(output) => output.status.success() && !output.stdout.is_empty(),
        None => false,
    }
}

/// Vérifie si les RPCs sont disponibles en envoyant une requête simple
async fn check_rpc_availability() -> Result<bool> {
    use reqwest::Client;

    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .context("Failed to build HTTP client")?;

    // Tester les 3 RPCs
    let rpc_urls = [
        "http://127.0.0.1:18082/json_rpc",
        "http://127.0.0.1:18083/json_rpc", 
        "http://127.0.0.1:18084/json_rpc",
    ];

    for url in &rpc_urls {
        let response = client
            .post(url.to_string())  // Convert to owned String to satisfy IntoUrl
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": "health_check",
                "method": "get_version"
            }))
            .send()
            .await;

        match response {
            Ok(_) => continue, // OK, ce RPC est accessible
            Err(_) => return Ok(false), // Un RPC est inaccessible
        }
    }

    Ok(true)
}

/// Démarre les instances de wallet RPC
pub fn start_wallet_rpcs() -> Result<()> {
    println!("🚀 Starting 3 Monero Wallet RPC instances...");

    // Tuer les instances existantes
    let _ = Command::new("killall")
        .args(["-9", "monero-wallet-rpc"])
        .status();

    std::thread::sleep(Duration::from_millis(1000));

    // Créer le répertoire des wallets s'il n'existe pas
    std::fs::create_dir_all("./testnet-wallets")
        .context("Failed to create testnet-wallets directory")?;

    // Démarrer le Buyer RPC (port 18082)
    let _output1 = Command::new("monero-wallet-rpc")
        .args([
            "--rpc-bind-port", "18082",
            "--disable-rpc-login",
            "--wallet-dir", "./testnet-wallets",
            "--daemon-address", "http://127.0.0.1:18081",
            "--testnet",
            "--offline",
            "--log-level", "2"
        ])
        .spawn()
        .context("Failed to start buyer RPC")?;

    // Démarrer le Vendor RPC (port 18083)
    let _output2 = Command::new("monero-wallet-rpc")
        .args([
            "--rpc-bind-port", "18083",
            "--disable-rpc-login", 
            "--wallet-dir", "./testnet-wallets",
            "--daemon-address", "http://127.0.0.1:18081",
            "--testnet",
            "--offline",
            "--log-level", "2"
        ])
        .spawn()
        .context("Failed to start vendor RPC")?;

    // Démarrer le Arbiter RPC (port 18084)
    let _output3 = Command::new("monero-wallet-rpc")
        .args([
            "--rpc-bind-port", "18084",
            "--disable-rpc-login",
            "--wallet-dir", "./testnet-wallets", 
            "--daemon-address", "http://127.0.0.1:18081",
            "--testnet",
            "--offline",
            "--log-level", "2"
        ])
        .spawn()
        .context("Failed to start arbiter RPC")?;

    // Attendre un peu pour que les processus démarrent
    std::thread::sleep(Duration::from_millis(500));

    // Vérifier que les processus sont bien lancés
    if !is_process_running("monero-wallet-rpc.*18082") {
        return Err(anyhow::anyhow!("Failed to start buyer RPC on port 18082"));
    }
    if !is_process_running("monero-wallet-rpc.*18083") {
        return Err(anyhow::anyhow!("Failed to start vendor RPC on port 18083"));
    }
    if !is_process_running("monero-wallet-rpc.*18084") {
        return Err(anyhow::anyhow!("Failed to start arbiter RPC on port 18084"));
    }

    println!("✅ All 3 Wallet RPC instances running (18082, 18083, 18084)");
    Ok(())
}

/// Vérifie et démarre automatiquement les dépendances nécessaires
pub async fn ensure_dependencies() -> Result<()> {
    println!("🔍 Checking dependencies...");

    // Vérifier si le daemon est en cours d'exécution
    if !is_process_running("monerod.*testnet") {
        println!("🚀 Starting Monero daemon in testnet mode...");
        let daemon_result = std::process::Command::new("monerod")
            .args(["--testnet", "--detach", "--data-dir", "./testnet-data"])
            .status()
            .context("Failed to start monerod daemon")?;
        
        if !daemon_result.success() {
            return Err(anyhow::anyhow!("Failed to start monerod daemon"));
        }
        
        // Attendre un peu pour que le daemon démarre
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        println!("✅ Monero daemon started in testnet mode");
    } else {
        println!("✅ Monero daemon is running");
    }

    // Vérifier si les RPCs sont accessibles
    if check_rpc_availability().await.unwrap_or(false) {
        println!("✅ All RPC instances are accessible");
    } else {
        println!("⚠️ RPC instances not accessible, starting them...");
        start_wallet_rpcs()?;
        
        // Attendre suffisamment que les RPC soient prêts
        // Les RPCs prennent quelques secondes pour être opérationnels après le démarrage
        let mut success = false;
        for attempt in 1..=10 {
            sleep(Duration::from_secs(2)).await;
            if check_rpc_availability().await.unwrap_or(false) {
                println!("✅ All RPC instances started and accessible");
                success = true;
                break;
            }
            println!("⏳ Waiting for RPC instances to be ready... (attempt {}/10)", attempt);
        }
        
        if !success {
            return Err(anyhow::anyhow!("Failed to start RPC instances - timeout waiting for them to become responsive"));
        }
    }

    println!("✅ All dependencies verified!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_process_running() {
        // Test with a known process
        let running = is_process_running("systemd");
        // This might not always be true depending on the environment
        // But the function should execute without panicking
        println!("systemd running: {}", running);
    }
}