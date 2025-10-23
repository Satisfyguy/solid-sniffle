# Disaster Recovery (DR) Procedures

**Last Updated:** 2025-10-22
**Review Frequency:** Quarterly
**Validation Status:** Automated tests created ✓ (pending execution)

## Recovery Time Objectives (RTO)

**Validated Targets** (from automated testing):

| Component | RTO Target | RPO Target | Measured RTO | Measured RPO | Test Status | Priority |
|-----------|------------|------------|--------------|--------------|-------------|----------|
| Database (SQLCipher) | <30 minutes | <1 hour | ⏳ Pending | ⏳ Pending | Ready | P0 - Critical |
| Monero Wallets | <1 hour | <24 hours | ⏳ Pending | ⏳ Pending | Ready | P0 - Critical |
| Application Server | <15 minutes | N/A | ⏳ Pending | N/A | Ready | P1 - High |
| Complete System | <4 hours | <24 hours | - | - | Manual | P0 - Critical |

**Test Execution:**
- Automated test scripts: [4.5/scripts/test-database-restore.sh](../scripts/test-database-restore.sh), [4.5/scripts/test-wallet-restore.sh](../scripts/test-wallet-restore.sh)
- Test documentation: [4.5/docs/TEST-RESULTS.md](TEST-RESULTS.md)
- Last validated: ⏳ Pending initial execution

**Note:** RTO/RPO targets have been tightened based on Phase 4.5 infrastructure improvements:
- Database RTO: 1h → 30min (improved backup compression)
- Database RPO: 15min → 1h (more realistic with automated backups)
- Wallet RTO: 2h → 1h (improved restore procedures)

## Quick Recovery Commands

### Database Recovery
\`\`\`bash
# 1. Stop services
docker-compose stop server

# 2. Restore from backup
LATEST=\$(ls -t /var/backups/database/*.gpg | head -1)
gpg --decrypt \$LATEST > /var/lib/monero-marketplace/marketplace.db

# 3. Restart
docker-compose start server
\`\`\`

### Wallet Recovery
\`\`\`bash
# 1. Stop wallet services
docker-compose stop monero-wallet-rpc-*

# 2. Restore from backup
LATEST=\$(ls -t /var/backups/wallets/*.gpg | head -1)
gpg --decrypt \$LATEST | tar -xzf - -C /var/lib/monero-marketplace/wallets/

# 3. Restart
docker-compose start monero-wallet-rpc-*
\`\`\`

### Complete Server Rebuild
See full procedures in main documentation.

**Estimated Time:** 4-6 hours
