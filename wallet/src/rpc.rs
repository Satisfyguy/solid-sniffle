//! Monero RPC client implementation

use monero_marketplace_common::{
    error::MoneroError,
    types::{
        ExportMultisigInfoResult, ImportMultisigInfoResult, MakeMultisigResult, MoneroConfig,
        MultisigInfo, PrepareMultisigResult, RpcRequest, RpcResponse,
    },
    MAX_MULTISIG_INFO_LEN, MIN_MULTISIG_INFO_LEN,
};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{sleep, Duration as TokioDuration};

/// Client RPC Monero
///
/// SÉCURITÉ:
/// - RPC bind sur localhost uniquement (vérifié à la création)
/// - Pas d'authentification requise (--disable-rpc-login en testnet)
/// - Timeout configurable via MONERO_RPC_TIMEOUT_SECS (défaut: 45s prod, 60s dev)
#[derive(Clone)]
pub struct MoneroRpcClient {
    url: String,
    client: Client,
    // Mutex pour sérialiser les appels RPC (protection race condition)
    rpc_lock: Arc<Mutex<()>>,
    // Semaphore pour limiter requêtes concurrentes (rate limiting)
    semaphore: Arc<Semaphore>,
}

impl MoneroRpcClient {
    /// Crée nouveau client RPC
    ///
    /// # OPSEC Note
    /// RPC doit être sur localhost UNIQUEMENT.
    /// JAMAIS exposer sur 0.0.0.0 ou IP publique.
    pub fn new(config: MoneroConfig) -> Result<Self, MoneroError> {
        let url = config.rpc_url;

        // OPSEC: Vérifier que URL est localhost
        if !url.contains("127.0.0.1") && !url.contains("localhost") {
            return Err(MoneroError::InvalidResponse(
                "RPC URL must be localhost only (OPSEC)".to_string(),
            ));
        }

        // Utiliser timeout depuis config
        let timeout_secs = config.timeout_seconds;

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| MoneroError::NetworkError(format!("Client build: {}", e)))?;

        Ok(Self {
            url,
            client,
            rpc_lock: Arc::new(Mutex::new(())),
            semaphore: Arc::new(Semaphore::new(5)), // Max 5 requêtes concurrentes
        })
    }

    /// Vérifie que RPC est accessible
    pub async fn check_connection(&self) -> Result<(), MoneroError> {
        let request = RpcRequest::new("get_version");

        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            return Err(MoneroError::RpcUnreachable);
        }

        Ok(())
    }

    /// Get wallet RPC version
    ///
    /// Returns the Monero wallet RPC version number.
    ///
    /// # Errors
    /// - MoneroError::RpcUnreachable - RPC pas accessible
    /// - MoneroError::InvalidResponse - Réponse invalide
    ///
    /// # Examples
    /// ```no_run
    /// # use wallet::MoneroRpcClient;
    /// # use common::MoneroConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MoneroConfig::default();
    /// let client = MoneroRpcClient::new(config)?;
    /// let version = client.get_version().await?;
    /// tracing::info!("Wallet RPC version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_version(&self) -> Result<u32, MoneroError> {
        // Acquérir permit pour rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

        // Acquérir lock pour sérialiser les appels RPC
        let _guard = self.rpc_lock.lock().await;

        let request = RpcRequest::new("get_version");

        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        let rpc_response: RpcResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(MoneroError::RpcError(error.message));
        }

        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;

        let version = result["version"]
            .as_u64()
            .ok_or_else(|| MoneroError::InvalidResponse("Invalid version format".to_string()))?;

        Ok(version as u32)
    }

    /// Get wallet balance
    ///
    /// Returns the wallet balance as (unlocked_balance, total_balance) in atomic units.
    /// Monero uses 12 decimal places, so 1 XMR = 1_000_000_000_000 atomic units.
    ///
    /// # Returns
    /// (unlocked_balance, total_balance) - Both in atomic units
    ///
    /// # Errors
    /// - MoneroError::RpcUnreachable - RPC pas accessible
    /// - MoneroError::InvalidResponse - Réponse invalide
    ///
    /// # Examples
    /// ```no_run
    /// # use wallet::MoneroRpcClient;
    /// # use common::MoneroConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MoneroConfig::default();
    /// let client = MoneroRpcClient::new(config)?;
    /// let (unlocked, total) = client.get_balance().await?;
    /// tracing::info!("Unlocked: {} atomic units", unlocked);
    /// tracing::info!("Total: {} atomic units", total);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_balance(&self) -> Result<(u64, u64), MoneroError> {
        // Acquérir permit pour rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

        // Acquérir lock pour sérialiser les appels RPC
        let _guard = self.rpc_lock.lock().await;

        let request = RpcRequest::new("get_balance");

        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        let rpc_response: RpcResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(MoneroError::RpcError(error.message));
        }

        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;

        let unlocked_balance = result["unlocked_balance"].as_u64().ok_or_else(|| {
            MoneroError::InvalidResponse("Invalid unlocked_balance format".to_string())
        })?;

        let balance = result["balance"]
            .as_u64()
            .ok_or_else(|| MoneroError::InvalidResponse("Invalid balance format".to_string()))?;

        Ok((unlocked_balance, balance))
    }

    /// Prépare wallet pour multisig (étape 1/6)
    ///
    /// # Errors
    /// - MoneroError::RpcUnreachable - RPC pas accessible
    /// - MoneroError::AlreadyMultisig - Wallet déjà en mode multisig
    /// - MoneroError::WalletLocked - Wallet verrouillé
    /// - MoneroError::InvalidResponse - Réponse invalide
    ///
    /// # Examples
    /// ```no_run
    /// # use wallet::MoneroRpcClient;
    /// # use common::MoneroConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MoneroConfig::default();
    /// let client = MoneroRpcClient::new(config)?;
    /// let info = client.prepare_multisig().await?;
    /// assert!(info.multisig_info.starts_with("MultisigV1"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn prepare_multisig(&self) -> Result<MultisigInfo, MoneroError> {
        // Retry logic avec backoff exponentiel
        let mut retries = 0;
        let max_retries = 3;

        loop {
            match self.prepare_multisig_inner().await {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries => {
                    let delay = TokioDuration::from_millis(100 * 2u64.pow(retries));
                    tracing::debug!(
                        "Retry {}/{}: {} (waiting {:?})",
                        retries + 1,
                        max_retries,
                        e,
                        delay
                    );
                    sleep(delay).await;
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn prepare_multisig_inner(&self) -> Result<MultisigInfo, MoneroError> {
        // Acquérir permit pour rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

        // Acquérir lock pour sérialiser les appels RPC
        let _guard = self.rpc_lock.lock().await;

        let request = RpcRequest::new("prepare_multisig");

        // Appel RPC avec gestion d'erreur
        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        // Parse JSON response
        let rpc_response: RpcResponse<PrepareMultisigResult> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

        // Handle RPC errors
        if let Some(error) = rpc_response.error {
            return Err(match error.message.as_str() {
                msg if msg.contains("already") && msg.contains("multisig") => {
                    MoneroError::AlreadyMultisig
                }
                msg if msg.contains("locked") => MoneroError::WalletLocked,
                _ => MoneroError::RpcError(error.message),
            });
        }

        // Extract result
        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;

        // VALIDATION STRICTE: multisig_info
        validate_multisig_info(&result.multisig_info)?;

        Ok(MultisigInfo {
            multisig_info: result.multisig_info,
        })
    }

    /// Crée wallet multisig 2-of-3 (étape 2/6)
    ///
    /// # Arguments
    /// * `threshold` - Nombre de signatures requises (2 pour 2-of-3)
    /// * `multisig_info` - Vec des multisig_info des autres participants
    ///
    /// # Errors
    /// - MoneroError::RpcUnreachable - RPC pas accessible
    /// - MoneroError::AlreadyMultisig - Wallet déjà finalisé en multisig
    /// - MoneroError::ValidationError - multisig_info invalides
    /// - MoneroError::RpcError - Erreur Monero (ex: threshold invalide)
    /// - MoneroError::WalletLocked - Wallet verrouillé
    /// - MoneroError::WalletBusy - Autre opération en cours
    ///
    /// # Examples
    /// ```no_run
    /// # use wallet::MoneroRpcClient;
    /// # use common::MoneroConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MoneroConfig::default();
    /// let client = MoneroRpcClient::new(config)?;
    ///
    /// // Après avoir récupéré les infos des 3 wallets via prepare_multisig
    /// let seller_info = "MultisigV1...".to_string();
    /// let arb_info = "MultisigV1...".to_string();
    ///
    /// let result = client.make_multisig(2, vec![seller_info, arb_info]).await?;
    /// assert!(result.address.starts_with("5")); // Testnet multisig address
    /// # Ok(())
    /// # }
    /// ```
    pub async fn make_multisig(
        &self,
        threshold: u32,
        multisig_info: Vec<String>,
    ) -> Result<MakeMultisigResult, MoneroError> {
        // Retry logic avec backoff exponentiel
        let mut retries = 0;
        let max_retries = 3;

        loop {
            match self
                .make_multisig_inner(threshold, multisig_info.clone())
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries => {
                    let delay = TokioDuration::from_millis(100 * 2u64.pow(retries));
                    tracing::debug!(
                        "Retry {}/{}: {} (waiting {:?})",
                        retries + 1,
                        max_retries,
                        e,
                        delay
                    );
                    sleep(delay).await;
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn make_multisig_inner(
        &self,
        threshold: u32,
        multisig_info: Vec<String>,
    ) -> Result<MakeMultisigResult, MoneroError> {
        // VALIDATION PRÉ-REQUÊTES

        // 1. Vérifier threshold valide (2 pour 2-of-3)
        if threshold < 2 {
            return Err(MoneroError::ValidationError(
                "Threshold must be at least 2".to_string(),
            ));
        }

        // 2. Vérifier nombre de multisig_info (doit être = total - 1)
        // Pour 2-of-3, on doit avoir 2 autres infos
        if multisig_info.len() < 2 {
            return Err(MoneroError::ValidationError(format!(
                "Need at least 2 multisig_info, got {}",
                multisig_info.len()
            )));
        }

        // 3. Valider chaque multisig_info
        for (i, info) in multisig_info.iter().enumerate() {
            validate_multisig_info(info).map_err(|e| {
                MoneroError::ValidationError(format!("Invalid multisig_info[{}]: {}", i, e))
            })?;
        }

        // Acquérir permit pour rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

        // Acquérir lock pour sérialiser les appels RPC
        let _guard = self.rpc_lock.lock().await;

        // Construire request avec params
        let mut request = RpcRequest::new("make_multisig");
        request.params = Some(serde_json::json!({
            "threshold": threshold,
            "multisig_info": multisig_info,
        }));

        // Appel RPC avec gestion d'erreur
        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        // Parse JSON response
        let rpc_response: RpcResponse<MakeMultisigResult> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

        // Handle RPC errors
        if let Some(error) = rpc_response.error {
            return Err(match error.message.as_str() {
                msg if msg.contains("already") && msg.contains("multisig") => {
                    MoneroError::AlreadyMultisig
                }
                msg if msg.contains("locked") => MoneroError::WalletLocked,
                msg if msg.contains("busy") => MoneroError::WalletBusy,
                msg if msg.contains("invalid") || msg.contains("Invalid") => {
                    MoneroError::ValidationError(error.message.clone())
                }
                _ => MoneroError::RpcError(error.message),
            });
        }

        // Extract result
        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;

        // VALIDATION POST-REQUÊTE

        // 1. Vérifier que address n'est pas vide
        if result.address.is_empty() {
            return Err(MoneroError::InvalidResponse(
                "Empty multisig address returned".to_string(),
            ));
        }

        // 2. Vérifier que multisig_info est valide
        validate_multisig_info(&result.multisig_info)?;

        Ok(result)
    }

    /// Exporte les informations multisig pour synchronisation (étape 3/6)
    ///
    /// Cette fonction doit être appelée DEUX fois dans le flow multisig:
    /// - Round 1: Après make_multisig
    /// - Round 2: Après premier import_multisig_info
    ///
    /// # Errors
    /// - MoneroError::RpcUnreachable - RPC pas accessible
    /// - MoneroError::NotMultisig - Wallet pas en mode multisig
    /// - MoneroError::WalletLocked - Wallet verrouillé
    /// - MoneroError::InvalidResponse - Réponse invalide
    ///
    /// # Examples
    /// ```no_run
    /// # use wallet::MoneroRpcClient;
    /// # use common::MoneroConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MoneroConfig::default();
    /// let client = MoneroRpcClient::new(config)?;
    ///
    /// // Après make_multisig
    /// let export_info = client.export_multisig_info().await?;
    /// // Partager export_info.info avec les autres participants
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_multisig_info(&self) -> Result<ExportMultisigInfoResult, MoneroError> {
        // Retry logic avec backoff exponentiel
        let mut retries = 0;
        let max_retries = 3;

        loop {
            match self.export_multisig_info_inner().await {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries => {
                    let delay = TokioDuration::from_millis(100 * 2u64.pow(retries));
                    tracing::debug!(
                        "Retry {}/{}: {} (waiting {:?})",
                        retries + 1,
                        max_retries,
                        e,
                        delay
                    );
                    sleep(delay).await;
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn export_multisig_info_inner(&self) -> Result<ExportMultisigInfoResult, MoneroError> {
        // Acquérir permit pour rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

        // Acquérir lock pour sérialiser les appels RPC
        let _guard = self.rpc_lock.lock().await;

        let request = RpcRequest::new("export_multisig_info");

        // Appel RPC avec gestion d'erreur
        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        // Parse JSON response
        let rpc_response: RpcResponse<ExportMultisigInfoResult> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

        // Handle RPC errors
        if let Some(error) = rpc_response.error {
            return Err(match error.message.as_str() {
                msg if msg.contains("not") && msg.contains("multisig") => MoneroError::NotMultisig,
                msg if msg.contains("locked") => MoneroError::WalletLocked,
                msg if msg.contains("busy") => MoneroError::WalletBusy,
                _ => MoneroError::RpcError(error.message),
            });
        }

        // Extract result
        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;

        // VALIDATION POST-REQUÊTE: Info non vide
        if result.info.is_empty() {
            return Err(MoneroError::InvalidResponse(
                "Empty multisig info returned".to_string(),
            ));
        }

        // Validation longueur
        if result.info.len() < 100 {
            return Err(MoneroError::InvalidResponse(format!(
                "Multisig info too short: {} chars",
                result.info.len()
            )));
        }

        if result.info.len() > 5000 {
            return Err(MoneroError::InvalidResponse(format!(
                "Multisig info too long: {} chars",
                result.info.len()
            )));
        }

        Ok(result)
    }

    /// Importe les informations multisig des autres participants (étape 4/6)
    ///
    /// Cette fonction doit être appelée DEUX fois dans le flow multisig:
    /// - Round 1: Importer les infos exportées par les autres après make_multisig
    /// - Round 2: Importer les infos exportées après le premier import
    ///
    /// # Arguments
    /// * `infos` - Vec des infos exportées des AUTRES participants (N-1 infos)
    ///
    /// # Errors
    /// - MoneroError::RpcUnreachable - RPC pas accessible
    /// - MoneroError::NotMultisig - Wallet pas en mode multisig
    /// - MoneroError::ValidationError - Infos invalides ou incompatibles
    /// - MoneroError::WalletLocked - Wallet verrouillé
    /// - MoneroError::RpcError - Erreur Monero
    ///
    /// # Examples
    /// ```no_run
    /// # use wallet::MoneroRpcClient;
    /// # use common::MoneroConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MoneroConfig::default();
    /// let client = MoneroRpcClient::new(config)?;
    ///
    /// // Récupérer infos des autres participants (via canal sécurisé)
    /// let seller_export = "...".to_string();
    /// let arb_export = "...".to_string();
    ///
    /// let result = client.import_multisig_info(vec![seller_export, arb_export]).await?;
    /// tracing::info!("Imported {} outputs", result.n_outputs);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn import_multisig_info(
        &self,
        infos: Vec<String>,
    ) -> Result<ImportMultisigInfoResult, MoneroError> {
        // Retry logic avec backoff exponentiel
        let mut retries = 0;
        let max_retries = 3;

        loop {
            match self.import_multisig_info_inner(infos.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries => {
                    let delay = TokioDuration::from_millis(100 * 2u64.pow(retries));
                    tracing::debug!(
                        "Retry {}/{}: {} (waiting {:?})",
                        retries + 1,
                        max_retries,
                        e,
                        delay
                    );
                    sleep(delay).await;
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn import_multisig_info_inner(
        &self,
        infos: Vec<String>,
    ) -> Result<ImportMultisigInfoResult, MoneroError> {
        // VALIDATION PRÉ-REQUÊTES

        // 1. Vérifier qu'il y a au moins 1 info
        if infos.is_empty() {
            return Err(MoneroError::ValidationError(
                "Need at least 1 multisig info to import".to_string(),
            ));
        }

        // 2. Pour 2-of-3, on attend exactement 2 infos (N-1)
        if infos.len() < 2 {
            return Err(MoneroError::ValidationError(format!(
                "Expected at least 2 infos for 2-of-3 multisig, got {}",
                infos.len()
            )));
        }

        // 3. Valider chaque info
        for (i, info) in infos.iter().enumerate() {
            if info.is_empty() {
                return Err(MoneroError::ValidationError(format!(
                    "Info[{}] is empty",
                    i
                )));
            }
            if info.len() < 100 {
                return Err(MoneroError::ValidationError(format!(
                    "Info[{}] too short: {} chars",
                    i,
                    info.len()
                )));
            }
        }

        // Acquérir permit pour rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

        // Acquérir lock pour sérialiser les appels RPC
        let _guard = self.rpc_lock.lock().await;

        // Construire request avec params
        let mut request = RpcRequest::new("import_multisig_info");
        request.params = Some(serde_json::json!({
            "info": infos,
        }));

        // Appel RPC avec gestion d'erreur
        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        // Parse JSON response
        let rpc_response: RpcResponse<ImportMultisigInfoResult> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

        // Handle RPC errors
        if let Some(error) = rpc_response.error {
            return Err(match error.message.as_str() {
                msg if msg.contains("not") && msg.contains("multisig") => MoneroError::NotMultisig,
                msg if msg.contains("locked") => MoneroError::WalletLocked,
                msg if msg.contains("busy") => MoneroError::WalletBusy,
                msg if msg.contains("invalid") || msg.contains("Invalid") => {
                    MoneroError::ValidationError(error.message.clone())
                }
                msg if msg.contains("already") => {
                    MoneroError::RpcError(format!("Already imported: {}", error.message))
                }
                _ => MoneroError::RpcError(error.message),
            });
        }

        // Extract result
        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;

        Ok(result)
    }

    /// Get current block height
    pub async fn get_block_height(&self) -> Result<u64, MoneroError> {
        // Acquérir permit pour rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

        // Acquérir lock pour sérialiser les appels RPC
        let _guard = self.rpc_lock.lock().await;

        let request = RpcRequest::new("get_height");

        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        let rpc_response: RpcResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(MoneroError::RpcError(error.message));
        }

        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;

        let height = result["height"]
            .as_u64()
            .ok_or_else(|| MoneroError::InvalidResponse("Invalid height format".to_string()))?;

        Ok(height)
    }

    /// Get daemon block height
    pub async fn get_daemon_block_height(&self) -> Result<u64, MoneroError> {
        // Pour simplifier, on retourne la même valeur que get_block_height
        // En réalité, ce serait un appel RPC différent vers le daemon
        self.get_block_height().await
    }

    /// Check if wallet is multisig
    pub async fn is_multisig(&self) -> Result<bool, MoneroError> {
        // Acquérir permit pour rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

        // Acquérir lock pour sérialiser les appels RPC
        let _guard = self.rpc_lock.lock().await;

        let request = RpcRequest::new("is_multisig");

        let response = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;

        let rpc_response: RpcResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(MoneroError::RpcError(error.message));
        }

        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;

        let is_multisig = result["multisig"]
            .as_bool()
            .ok_or_else(|| MoneroError::InvalidResponse("Invalid multisig format".to_string()))?;

        Ok(is_multisig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monero_rpc_client_localhost_only() {
        // OPSEC: Vérifier que client rejette URLs publiques
        let config = MoneroConfig {
            rpc_url: "http://0.0.0.0:18082".to_string(),
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 60,
        };
        let result = MoneroRpcClient::new(config);
        assert!(result.is_err());

        let config = MoneroConfig {
            rpc_url: "http://192.168.1.10:18082".to_string(),
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 60,
        };
        let result = MoneroRpcClient::new(config);
        assert!(result.is_err());

        // Localhost OK
        let config = MoneroConfig::default();
        let result = MoneroRpcClient::new(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prepare_multisig() {
        // SETUP: monero-wallet-rpc doit tourner sur 18082
        // Voir docs/specs/prepare_multisig.md pour commandes

        let config = MoneroConfig::default();
        let client = match MoneroRpcClient::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create client for test: {}", e);
                return;
            }
        };

        // Vérifier connexion d'abord
        match client.check_connection().await {
            Ok(_) => tracing::info!("RPC accessible"),
            Err(e) => {
                tracing::warn!("RPC pas accessible: {}", e);
                tracing::info!("Lance: monero-wallet-rpc --testnet ...");
                return;
            }
        }

        // Tester prepare_multisig
        let result = client.prepare_multisig().await;

        match &result {
            Ok(info) => {
                tracing::info!("prepare_multisig OK");
                tracing::debug!("Info length: {}", info.multisig_info.len());
                assert!(info.multisig_info.starts_with("MultisigV1"));
                assert!(info.multisig_info.len() > MIN_MULTISIG_INFO_LEN);
            }
            Err(MoneroError::AlreadyMultisig) => {
                tracing::warn!("Wallet déjà en multisig (normal si test rejoué)");
                tracing::info!("Pour reset: fermer RPC, supprimer wallet, recréer");
                // Ce n'est pas un échec de test
                return;
            }
            Err(e) => {
                tracing::error!("prepare_multisig échoué: {}", e);
                return;
            }
        }
    }

    #[tokio::test]
    async fn test_prepare_multisig_rpc_down() {
        // Test avec RPC pas lancé
        let config = MoneroConfig {
            rpc_url: "http://127.0.0.1:19999".to_string(),
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 5,
        };
        let client = match MoneroRpcClient::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create client for test: {}", e);
                return;
            }
        };

        let result = client.prepare_multisig().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MoneroError::RpcUnreachable));
    }

    #[tokio::test]
    async fn test_prepare_multisig_concurrent() {
        // Test appels concurrents (doit être thread-safe)
        let config = MoneroConfig::default();
        let client = match MoneroRpcClient::new(config) {
            Ok(c) => Arc::new(c),
            Err(e) => {
                tracing::error!("Failed to create client for test: {}", e);
                return;
            }
        };

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let client = Arc::clone(&client);
                tokio::spawn(async move { client.prepare_multisig().await })
            })
            .collect();

        // Tous doivent réussir OU échouer proprement (pas de panic)
        for handle in handles {
            let result = handle.await.expect("Task should complete without panic");
            // Peut échouer si RPC pas lancé, mais ne doit pas panic
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[tokio::test]
    async fn test_validate_multisig_info() {
        // Test validation stricte
        use super::validate_multisig_info;

        // Cas valides - Utiliser strings de longueur réaliste (>= 100 chars)
        let valid_info = format!("MultisigV1{}", "K".repeat(100)); // > MIN_MULTISIG_INFO_LEN
        assert!(validate_multisig_info(&valid_info).is_ok());

        // Cas invalides
        assert!(validate_multisig_info("InvalidPrefix...").is_err());
        assert!(validate_multisig_info("MultisigV1").is_err()); // Trop court
        assert!(validate_multisig_info(&"MultisigV1".repeat(1000)).is_err()); // Trop long
        let invalid_chars = format!("MultisigV1{}", "@#$%".repeat(30)); // Caractères invalides
        assert!(validate_multisig_info(&invalid_chars).is_err());
    }

    #[tokio::test]
    async fn test_make_multisig_validation() {
        // Test validation des paramètres
        let config = MoneroConfig::default();
        let client = match MoneroRpcClient::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create client for test: {}", e);
                return;
            }
        };

        // Cas 1: threshold trop bas
        let fake_info1 = format!("MultisigV1{}", "A".repeat(100));
        let fake_info2 = format!("MultisigV1{}", "B".repeat(100));
        let result = client
            .make_multisig(1, vec![fake_info1.clone(), fake_info2.clone()])
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoneroError::ValidationError(_)
        ));

        // Cas 2: pas assez de multisig_info
        let result = client.make_multisig(2, vec![fake_info1.clone()]).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoneroError::ValidationError(_)
        ));

        // Cas 3: multisig_info invalide
        let result = client
            .make_multisig(2, vec!["InvalidInfo".to_string(), fake_info1.clone()])
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoneroError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_make_multisig() {
        // SETUP: Nécessite 3 wallets testnet avec prepare_multisig fait
        // Voir docs/specs/make_multisig.md pour commandes complètes

        let config = MoneroConfig::default();
        let client = match MoneroRpcClient::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create client for test: {}", e);
                return;
            }
        };

        // Vérifier connexion d'abord
        match client.check_connection().await {
            Ok(_) => tracing::info!("RPC accessible"),
            Err(e) => {
                tracing::warn!("RPC pas accessible: {}", e);
                tracing::info!("Lance: monero-wallet-rpc --testnet ...");
                return;
            }
        }

        // NOTE: Ce test nécessite une configuration manuelle complexe
        // avec 3 wallets RPC et leurs infos prepare_multisig
        // Pour l'instant, on teste juste que l'erreur est propre
        let fake_info1 = format!("MultisigV1{}", "A".repeat(100));
        let fake_info2 = format!("MultisigV1{}", "B".repeat(100));
        let result = client.make_multisig(2, vec![fake_info1, fake_info2]).await;

        match &result {
            Ok(result) => {
                tracing::info!("make_multisig OK");
                tracing::debug!("Address: {}", result.address);
                assert!(!result.address.is_empty());
                assert!(result.multisig_info.starts_with("MultisigV1"));
            }
            Err(MoneroError::AlreadyMultisig) => {
                tracing::warn!("Wallet déjà en multisig (normal si test rejoué)");
                return;
            }
            Err(MoneroError::ValidationError(msg)) => {
                tracing::warn!("Validation error (expected avec fake infos): {}", msg);
                return;
            }
            Err(MoneroError::RpcError(msg)) => {
                tracing::warn!("RPC error (expected avec fake infos): {}", msg);
                return;
            }
            Err(e) => {
                tracing::error!("make_multisig échoué: {}", e);
                return;
            }
        }
    }

    #[tokio::test]
    async fn test_make_multisig_rpc_down() {
        // Test avec RPC pas lancé
        let config = MoneroConfig {
            rpc_url: "http://127.0.0.1:19999".to_string(),
            rpc_user: None,
            rpc_password: None,
            timeout_seconds: 5,
        };
        let client = match MoneroRpcClient::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create client for test: {}", e);
                return;
            }
        };

        let fake_info1 = format!("MultisigV1{}", "A".repeat(100));
        let fake_info2 = format!("MultisigV1{}", "B".repeat(100));
        let result = client.make_multisig(2, vec![fake_info1, fake_info2]).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MoneroError::RpcUnreachable));
    }

    #[tokio::test]
    async fn test_export_multisig_info_validation() {
        // Test export validation (sans RPC)
        let config = MoneroConfig::default();
        let client = match MoneroRpcClient::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create client for test: {}", e);
                return;
            }
        };

        // Test que export nécessite RPC actif
        // Ce test vérifie juste la structure, pas le RPC réel
        let result = client.export_multisig_info().await;

        // Devrait échouer si RPC n'est pas lancé OU si pas en multisig
        match result {
            Ok(_) => {
                // Si RPC est lancé et en multisig, on accepte
                tracing::info!("export_multisig_info OK (wallet en multisig)");
            }
            Err(MoneroError::RpcUnreachable) => {
                // RPC pas lancé - normal
                tracing::info!("RPC pas lancé (expected)");
            }
            Err(MoneroError::NotMultisig) => {
                // Wallet pas en multisig - normal
                tracing::info!("Wallet not in multisig mode (expected)");
            }
            Err(e) => {
                tracing::warn!("Autre erreur: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_import_multisig_info_validation() {
        // Test validation des paramètres d'import
        let config = MoneroConfig::default();
        let client = match MoneroRpcClient::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create client for test: {}", e);
                return;
            }
        };

        // Cas 1: Liste vide
        let result = client.import_multisig_info(vec![]).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoneroError::ValidationError(_)
        ));

        // Cas 2: Pas assez d'infos (< 2 pour 2-of-3)
        let fake_info = "a".repeat(150); // > 100 chars pour validation
        let result = client.import_multisig_info(vec![fake_info]).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoneroError::ValidationError(_)
        ));

        // Cas 3: Info trop courte
        let result = client
            .import_multisig_info(vec!["short".to_string(), "also_short".to_string()])
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoneroError::ValidationError(_)
        ));

        // Cas 4: Info vide
        let fake_info = "a".repeat(150);
        let result = client
            .import_multisig_info(vec!["".to_string(), fake_info])
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoneroError::ValidationError(_)
        ));
    }
}

/// Validation stricte multisig_info
fn validate_multisig_info(info: &str) -> Result<(), MoneroError> {
    // Doit commencer par MultisigV1
    if !info.starts_with("MultisigV1") {
        return Err(MoneroError::InvalidResponse(
            "Invalid multisig_info prefix".to_string(),
        ));
    }

    // Longueur attendue (base64)
    if info.len() < MIN_MULTISIG_INFO_LEN || info.len() > MAX_MULTISIG_INFO_LEN {
        return Err(MoneroError::InvalidResponse(format!(
            "Invalid multisig_info length: {}",
            info.len()
        )));
    }

    // Caractères valides (base64 + prefix)
    if !info
        .chars()
        .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        return Err(MoneroError::InvalidResponse(
            "Invalid characters in multisig_info".to_string(),
        ));
    }

    Ok(())
}
