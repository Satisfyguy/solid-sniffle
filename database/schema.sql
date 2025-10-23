CREATE TABLE users (
    id VARCHAR(36) PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    wallet_address VARCHAR(95),
    wallet_id VARCHAR(36),
    role VARCHAR(20) NOT NULL CHECK (role IN ('buyer', 'vendor', 'arbiter', 'admin')),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE listings (
    id VARCHAR(36) PRIMARY KEY,
    vendor_id VARCHAR(36) REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    price_xmr BIGINT NOT NULL CHECK (price_xmr > 0),
    stock INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE orders (
    id VARCHAR(36) PRIMARY KEY,
    buyer_id VARCHAR(36) REFERENCES users(id) ON DELETE SET NULL,
    vendor_id VARCHAR(36) REFERENCES users(id) ON DELETE SET NULL,
    listing_id VARCHAR(36) REFERENCES listings(id) ON DELETE SET NULL,
    escrow_id VARCHAR(36) UNIQUE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    total_xmr BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE escrows (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) REFERENCES orders(id) ON DELETE CASCADE,
    buyer_wallet_info TEXT, -- ENCRYPTED
    vendor_wallet_info TEXT, -- ENCRYPTED
    arbiter_wallet_info TEXT, -- ENCRYPTED
    multisig_address VARCHAR(95),
    status VARCHAR(50) NOT NULL DEFAULT 'init',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE transactions (
    id VARCHAR(36) PRIMARY KEY,
    escrow_id VARCHAR(36) REFERENCES escrows(id) ON DELETE CASCADE,
    tx_hash VARCHAR(64) UNIQUE,
    amount_xmr BIGINT NOT NULL,
    confirmations INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_listings_vendor ON listings(vendor_id);
CREATE INDEX idx_orders_buyer ON orders(buyer_id);
CREATE INDEX idx_orders_vendor ON orders(vendor_id);
CREATE INDEX idx_escrows_order ON escrows(order_id);
CREATE INDEX idx_transactions_escrow ON transactions(escrow_id);