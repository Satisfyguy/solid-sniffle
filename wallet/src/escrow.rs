//! Escrow management for Monero Marketplace
//!
//! This module provides the EscrowManager for handling escrow operations
//! including creation, funding, release, refund, and dispute resolution.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::client::MoneroClient;
use monero_marketplace_common::{
    Amount, Error, Escrow, EscrowData, EscrowId, EscrowResult, EscrowState, MoneroAddress, TxHash,
    UserId,
};

/// In-memory storage for escrows (will be replaced with database in Phase 2)
type EscrowStorage = Arc<RwLock<HashMap<EscrowId, Escrow>>>;

/// Manages escrow operations and state transitions
pub struct EscrowManager {
    /// In-memory storage for escrows
    escrows: EscrowStorage,
    /// Monero client for blockchain operations (TODO: will be used for transaction verification)
    #[allow(dead_code)]
    monero_client: Arc<MoneroClient>,
}

impl EscrowManager {
    /// Create a new EscrowManager
    pub fn new(monero_client: Arc<MoneroClient>) -> Self {
        Self {
            escrows: Arc::new(RwLock::new(HashMap::new())),
            monero_client,
        }
    }

    /// Create a new escrow
    pub async fn create_escrow(
        &self,
        buyer: UserId,
        seller: UserId,
        arbiter: UserId,
        amount: Amount,
        multisig_address: MoneroAddress,
    ) -> Result<EscrowResult> {
        info!(
            "Creating new escrow: buyer={}, seller={}, amount={}",
            buyer, seller, amount
        );

        // Validate inputs
        if amount == 0 {
            return Err(Error::InvalidAmount("Amount must be greater than 0".to_string()).into());
        }

        if buyer == seller {
            return Err(
                Error::InvalidUser("Buyer and seller cannot be the same".to_string()).into(),
            );
        }

        if buyer == arbiter || seller == arbiter {
            return Err(Error::InvalidUser(
                "Arbiter must be different from buyer and seller".to_string(),
            )
            .into());
        }

        // Generate unique escrow ID
        let escrow_id = self.generate_escrow_id();

        // Create escrow data
        let escrow_data = EscrowData {
            buyer,
            seller,
            arbiter,
            amount,
            multisig_address,
        };

        // Create escrow
        let escrow = Escrow::new(escrow_id.clone(), escrow_data);

        // Store escrow
        {
            let mut escrows = self.escrows.write().await;
            escrows.insert(escrow_id.clone(), escrow.clone());
        }

        info!("Escrow created successfully: {}", escrow_id);
        Ok(EscrowResult::Created(escrow))
    }

    /// Fund an escrow (buyer deposits funds to multisig address)
    pub async fn fund_escrow(
        &self,
        escrow_id: &EscrowId,
        funding_tx_hash: TxHash,
    ) -> Result<EscrowResult> {
        info!("Funding escrow: {}", escrow_id);

        let mut escrow = self.get_escrow(escrow_id).await?;

        // Validate escrow can be funded
        if !escrow.can_be_funded() {
            return Err(Error::InvalidState(format!(
                "Escrow {} cannot be funded in state {:?}",
                escrow_id, escrow.state
            ))
            .into());
        }

        // Verify the transaction exists and is valid
        self.verify_funding_transaction(&funding_tx_hash, &escrow)
            .await
            .context("Failed to verify funding transaction")?;

        // Update escrow state
        escrow
            .transition_to(EscrowState::Funded)
            .map_err(|e| Error::InvalidState(e))?;
        escrow.funding_tx_hash = Some(funding_tx_hash.clone());

        // Store updated escrow
        {
            let mut escrows = self.escrows.write().await;
            escrows.insert(escrow_id.clone(), escrow);
        }

        info!(
            "Escrow funded successfully: {} with tx {}",
            escrow_id, funding_tx_hash
        );
        Ok(EscrowResult::Funded {
            escrow_id: escrow_id.clone(),
            tx_hash: funding_tx_hash,
        })
    }

    /// Release funds to seller (2-of-3 multisig: buyer + seller)
    pub async fn release_funds(
        &self,
        escrow_id: &EscrowId,
        requester: &UserId,
    ) -> Result<EscrowResult> {
        info!("Releasing funds for escrow: {} by {}", escrow_id, requester);

        let escrow = self.get_escrow(escrow_id).await?;

        // Validate requester is buyer or seller
        if requester != &escrow.data.buyer && requester != &escrow.data.seller {
            return Err(Error::Unauthorized(format!(
                "User {} is not authorized to release funds for escrow {}",
                requester, escrow_id
            ))
            .into());
        }

        // Validate escrow can be released
        if !escrow.can_be_released() {
            return Err(Error::InvalidState(format!(
                "Escrow {} cannot be released in state {:?}",
                escrow_id, escrow.state
            ))
            .into());
        }

        // Create transaction to release funds to seller
        let release_tx_hash = self
            .create_release_transaction(&escrow)
            .await
            .context("Failed to create release transaction")?;

        // Update escrow state
        let mut updated_escrow = escrow.clone();
        updated_escrow
            .transition_to(EscrowState::Released)
            .map_err(|e| Error::InvalidState(e))?;
        updated_escrow.release_tx_hash = Some(release_tx_hash.clone());

        // Store updated escrow
        {
            let mut escrows = self.escrows.write().await;
            escrows.insert(escrow_id.clone(), updated_escrow);
        }

        info!(
            "Funds released successfully: {} with tx {}",
            escrow_id, release_tx_hash
        );
        Ok(EscrowResult::Released {
            escrow_id: escrow_id.clone(),
            tx_hash: release_tx_hash,
        })
    }

    /// Refund funds to buyer (2-of-3 multisig: buyer + arbiter)
    pub async fn refund_buyer(
        &self,
        escrow_id: &EscrowId,
        requester: &UserId,
    ) -> Result<EscrowResult> {
        info!("Refunding buyer for escrow: {} by {}", escrow_id, requester);

        let escrow = self.get_escrow(escrow_id).await?;

        // Validate requester is buyer or arbiter
        if requester != &escrow.data.buyer && requester != &escrow.data.arbiter {
            return Err(Error::Unauthorized(format!(
                "User {} is not authorized to refund funds for escrow {}",
                requester, escrow_id
            ))
            .into());
        }

        // Validate escrow can be refunded
        if !escrow.can_be_refunded() {
            return Err(Error::InvalidState(format!(
                "Escrow {} cannot be refunded in state {:?}",
                escrow_id, escrow.state
            ))
            .into());
        }

        // Create transaction to refund funds to buyer
        let refund_tx_hash = self
            .create_refund_transaction(&escrow)
            .await
            .context("Failed to create refund transaction")?;

        // Update escrow state
        let mut updated_escrow = escrow.clone();
        updated_escrow
            .transition_to(EscrowState::Refunded)
            .map_err(|e| Error::InvalidState(e))?;
        updated_escrow.refund_tx_hash = Some(refund_tx_hash.clone());

        // Store updated escrow
        {
            let mut escrows = self.escrows.write().await;
            escrows.insert(escrow_id.clone(), updated_escrow);
        }

        info!(
            "Buyer refunded successfully: {} with tx {}",
            escrow_id, refund_tx_hash
        );
        Ok(EscrowResult::Refunded {
            escrow_id: escrow_id.clone(),
            tx_hash: refund_tx_hash,
        })
    }

    /// Open a dispute for an escrow
    pub async fn open_dispute(
        &self,
        escrow_id: &EscrowId,
        disputed_by: &UserId,
        reason: String,
    ) -> Result<EscrowResult> {
        info!(
            "Opening dispute for escrow: {} by {} - reason: {}",
            escrow_id, disputed_by, reason
        );

        let escrow = self.get_escrow(escrow_id).await?;

        // Validate requester is buyer or seller
        if disputed_by != &escrow.data.buyer && disputed_by != &escrow.data.seller {
            return Err(Error::Unauthorized(format!(
                "User {} is not authorized to open dispute for escrow {}",
                disputed_by, escrow_id
            ))
            .into());
        }

        // Validate escrow can be disputed
        if !escrow.can_be_disputed() {
            return Err(Error::InvalidState(format!(
                "Escrow {} cannot be disputed in state {:?}",
                escrow_id, escrow.state
            ))
            .into());
        }

        // Update escrow state
        let mut updated_escrow = escrow.clone();
        updated_escrow
            .transition_to(EscrowState::Disputed)
            .map_err(|e| Error::InvalidState(e))?;
        updated_escrow.dispute_reason = Some(reason.clone());
        updated_escrow.disputed_by = Some(disputed_by.clone());

        // Store updated escrow
        {
            let mut escrows = self.escrows.write().await;
            escrows.insert(escrow_id.clone(), updated_escrow);
        }

        info!(
            "Dispute opened successfully: {} by {}",
            escrow_id, disputed_by
        );
        Ok(EscrowResult::Disputed {
            escrow_id: escrow_id.clone(),
            reason,
        })
    }

    /// Get escrow status
    pub async fn get_escrow_status(&self, escrow_id: &EscrowId) -> Result<Escrow> {
        self.get_escrow(escrow_id).await
    }

    /// List all escrows (for debugging/admin purposes)
    pub async fn list_escrows(&self) -> Result<Vec<Escrow>> {
        let escrows = self.escrows.read().await;
        Ok(escrows.values().cloned().collect())
    }

    // ============================================================================
    // PRIVATE HELPER METHODS
    // ============================================================================

    /// Get an escrow by ID
    async fn get_escrow(&self, escrow_id: &EscrowId) -> Result<Escrow> {
        let escrows = self.escrows.read().await;
        escrows
            .get(escrow_id)
            .cloned()
            .ok_or_else(|| Error::EscrowNotFound(escrow_id.clone()).into())
    }

    /// Generate a unique escrow ID
    fn generate_escrow_id(&self) -> EscrowId {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        format!(
            "escrow_{}_{}",
            timestamp,
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        )
    }

    /// Verify that a funding transaction is valid
    async fn verify_funding_transaction(&self, tx_hash: &TxHash, escrow: &Escrow) -> Result<()> {
        // TODO: Implement actual transaction verification
        // For now, we'll just check that the transaction hash is not empty
        if tx_hash.is_empty() {
            return Err(Error::InvalidTransaction("Empty transaction hash".to_string()).into());
        }

        // TODO: Verify transaction exists on blockchain
        // TODO: Verify transaction amount matches escrow amount
        // TODO: Verify transaction destination is the multisig address

        info!(
            "Funding transaction verified: {} for escrow {}",
            tx_hash, escrow.id
        );
        Ok(())
    }

    /// Create a transaction to release funds to seller
    async fn create_release_transaction(&self, escrow: &Escrow) -> Result<TxHash> {
        // TODO: Implement actual multisig transaction creation
        // This would involve:
        // 1. Creating unsigned transaction (buyer + seller sign)
        // 2. Collecting signatures from buyer and seller
        // 3. Finalizing and broadcasting the transaction

        // For now, return a mock transaction hash
        let mock_tx_hash = format!(
            "release_{}_{}",
            escrow.id,
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        info!(
            "Release transaction created: {} for escrow {}",
            mock_tx_hash, escrow.id
        );
        Ok(mock_tx_hash)
    }

    /// Create a transaction to refund funds to buyer
    async fn create_refund_transaction(&self, escrow: &Escrow) -> Result<TxHash> {
        // TODO: Implement actual multisig transaction creation
        // This would involve:
        // 1. Creating unsigned transaction (buyer + arbiter sign)
        // 2. Collecting signatures from buyer and arbiter
        // 3. Finalizing and broadcasting the transaction

        // For now, return a mock transaction hash
        let mock_tx_hash = format!(
            "refund_{}_{}",
            escrow.id,
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        info!(
            "Refund transaction created: {} for escrow {}",
            mock_tx_hash, escrow.id
        );
        Ok(mock_tx_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MoneroClient;

    async fn create_test_escrow_manager() -> EscrowManager {
        use monero_marketplace_common::MoneroConfig;
        let config = MoneroConfig::default();
        let monero_client = Arc::new(MoneroClient::new(config).unwrap());
        EscrowManager::new(monero_client)
    }

    #[tokio::test]
    async fn test_create_escrow() -> Result<()> {
        let manager = create_test_escrow_manager().await;

        let result = manager
            .create_escrow(
                "buyer1".to_string(),
                "seller1".to_string(),
                "arbiter1".to_string(),
                1000000000000, // 1 XMR in atomic units
                "5TestMultisigAddress123456789".to_string(),
            )
            .await?;

        match result {
            EscrowResult::Created(escrow) => {
                assert_eq!(escrow.data.buyer, "buyer1");
                assert_eq!(escrow.data.seller, "seller1");
                assert_eq!(escrow.data.arbiter, "arbiter1");
                assert_eq!(escrow.data.amount, 1000000000000);
                assert_eq!(escrow.state, EscrowState::Created);
                assert!(escrow.funding_tx_hash.is_none());
                assert!(escrow.release_tx_hash.is_none());
                assert!(escrow.refund_tx_hash.is_none());
            }
            _ => panic!("Expected EscrowResult::Created"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_fund_escrow() -> Result<()> {
        let manager = create_test_escrow_manager().await;

        // Create escrow first
        let create_result = manager
            .create_escrow(
                "buyer1".to_string(),
                "seller1".to_string(),
                "arbiter1".to_string(),
                1000000000000,
                "5TestMultisigAddress123456789".to_string(),
            )
            .await?;

        let escrow_id = match create_result {
            EscrowResult::Created(escrow) => escrow.id,
            _ => panic!("Expected EscrowResult::Created"),
        };

        // Fund the escrow
        let fund_result = manager
            .fund_escrow(&escrow_id, "funding_tx_hash_123".to_string())
            .await?;

        match fund_result {
            EscrowResult::Funded {
                escrow_id: returned_id,
                tx_hash,
            } => {
                assert_eq!(returned_id, escrow_id);
                assert_eq!(tx_hash, "funding_tx_hash_123");
            }
            _ => panic!("Expected EscrowResult::Funded"),
        }

        // Verify escrow state
        let escrow = manager.get_escrow_status(&escrow_id).await?;
        assert_eq!(escrow.state, EscrowState::Funded);
        assert_eq!(
            escrow.funding_tx_hash,
            Some("funding_tx_hash_123".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_release_funds() -> Result<()> {
        let manager = create_test_escrow_manager().await;

        // Create and fund escrow
        let create_result = manager
            .create_escrow(
                "buyer1".to_string(),
                "seller1".to_string(),
                "arbiter1".to_string(),
                1000000000000,
                "5TestMultisigAddress123456789".to_string(),
            )
            .await?;

        let escrow_id = match create_result {
            EscrowResult::Created(escrow) => escrow.id,
            _ => panic!("Expected EscrowResult::Created"),
        };

        manager
            .fund_escrow(&escrow_id, "funding_tx_hash_123".to_string())
            .await?;

        // Release funds
        let release_result = manager
            .release_funds(&escrow_id, &"buyer1".to_string())
            .await?;

        match release_result {
            EscrowResult::Released {
                escrow_id: returned_id,
                tx_hash,
            } => {
                assert_eq!(returned_id, escrow_id);
                assert!(tx_hash.starts_with("release_"));
            }
            _ => panic!("Expected EscrowResult::Released"),
        }

        // Verify escrow state
        let escrow = manager.get_escrow_status(&escrow_id).await?;
        assert_eq!(escrow.state, EscrowState::Released);
        assert!(escrow.release_tx_hash.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_refund_buyer() -> Result<()> {
        let manager = create_test_escrow_manager().await;

        // Create and fund escrow
        let create_result = manager
            .create_escrow(
                "buyer1".to_string(),
                "seller1".to_string(),
                "arbiter1".to_string(),
                1000000000000,
                "5TestMultisigAddress123456789".to_string(),
            )
            .await?;

        let escrow_id = match create_result {
            EscrowResult::Created(escrow) => escrow.id,
            _ => panic!("Expected EscrowResult::Created"),
        };

        manager
            .fund_escrow(&escrow_id, "funding_tx_hash_123".to_string())
            .await?;

        // Refund buyer
        let refund_result = manager
            .refund_buyer(&escrow_id, &"buyer1".to_string())
            .await?;

        match refund_result {
            EscrowResult::Refunded {
                escrow_id: returned_id,
                tx_hash,
            } => {
                assert_eq!(returned_id, escrow_id);
                assert!(tx_hash.starts_with("refund_"));
            }
            _ => panic!("Expected EscrowResult::Refunded"),
        }

        // Verify escrow state
        let escrow = manager.get_escrow_status(&escrow_id).await?;
        assert_eq!(escrow.state, EscrowState::Refunded);
        assert!(escrow.refund_tx_hash.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_open_dispute() -> Result<()> {
        let manager = create_test_escrow_manager().await;

        // Create and fund escrow
        let create_result = manager
            .create_escrow(
                "buyer1".to_string(),
                "seller1".to_string(),
                "arbiter1".to_string(),
                1000000000000,
                "5TestMultisigAddress123456789".to_string(),
            )
            .await?;

        let escrow_id = match create_result {
            EscrowResult::Created(escrow) => escrow.id,
            _ => panic!("Expected EscrowResult::Created"),
        };

        manager
            .fund_escrow(&escrow_id, "funding_tx_hash_123".to_string())
            .await?;

        // Open dispute
        let dispute_result = manager
            .open_dispute(
                &escrow_id,
                &"buyer1".to_string(),
                "Product not as described".to_string(),
            )
            .await?;

        match dispute_result {
            EscrowResult::Disputed {
                escrow_id: returned_id,
                reason,
            } => {
                assert_eq!(returned_id, escrow_id);
                assert_eq!(reason, "Product not as described");
            }
            _ => panic!("Expected EscrowResult::Disputed"),
        }

        // Verify escrow state
        let escrow = manager.get_escrow_status(&escrow_id).await?;
        assert_eq!(escrow.state, EscrowState::Disputed);
        assert_eq!(
            escrow.dispute_reason,
            Some("Product not as described".to_string())
        );
        assert_eq!(escrow.disputed_by, Some("buyer1".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_state_transitions() -> Result<()> {
        let manager = create_test_escrow_manager().await;

        // Create escrow
        let create_result = manager
            .create_escrow(
                "buyer1".to_string(),
                "seller1".to_string(),
                "arbiter1".to_string(),
                1000000000000,
                "5TestMultisigAddress123456789".to_string(),
            )
            .await?;

        let escrow_id = match create_result {
            EscrowResult::Created(escrow) => escrow.id,
            _ => panic!("Expected EscrowResult::Created"),
        };

        // Try to release funds before funding (should fail)
        let result = manager
            .release_funds(&escrow_id, &"buyer1".to_string())
            .await;
        assert!(result.is_err());

        // Try to refund before funding (should fail)
        let result = manager
            .refund_buyer(&escrow_id, &"buyer1".to_string())
            .await;
        assert!(result.is_err());

        // Try to dispute before funding (should fail)
        let result = manager
            .open_dispute(
                &escrow_id,
                &"buyer1".to_string(),
                "Test dispute".to_string(),
            )
            .await;
        assert!(result.is_err());

        Ok(())
    }
}
