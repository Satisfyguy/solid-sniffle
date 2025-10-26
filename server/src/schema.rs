// @generated automatically by Diesel CLI.

diesel::table! {
    escrows (id) {
        id -> Text,
        order_id -> Text,
        buyer_id -> Text,
        vendor_id -> Text,
        arbiter_id -> Text,
        amount -> BigInt,
        multisig_address -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        buyer_wallet_info -> Nullable<Binary>,
        vendor_wallet_info -> Nullable<Binary>,
        arbiter_wallet_info -> Nullable<Binary>,
        transaction_hash -> Nullable<Text>,
        expires_at -> Nullable<Timestamp>,
        last_activity_at -> Timestamp,
        multisig_phase -> Text,
        multisig_state_json -> Nullable<Text>,
        multisig_updated_at -> Integer,
    }
}

diesel::table! {
    listings (id) {
        id -> Text,
        vendor_id -> Text,
        title -> Text,
        description -> Text,
        price_xmr -> BigInt,
        stock -> Integer,
        status -> Text,
        images_ipfs_cids -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    orders (id) {
        id -> Text,
        buyer_id -> Text,
        vendor_id -> Text,
        listing_id -> Text,
        escrow_id -> Nullable<Text>,
        status -> Text,
        total_xmr -> BigInt,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    reviews (id) {
        id -> Text,
        txid -> Text,
        reviewer_id -> Text,
        vendor_id -> Text,
        rating -> Integer,
        comment -> Nullable<Text>,
        buyer_pubkey -> Text,
        signature -> Text,
        timestamp -> Timestamp,
        verified -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    transactions (id) {
        id -> Text,
        escrow_id -> Text,
        tx_hash -> Nullable<Text>,
        amount_xmr -> BigInt,
        confirmations -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password_hash -> Text,
        role -> Text,
        wallet_address -> Nullable<Text>,
        wallet_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(escrows -> orders (order_id));
diesel::joinable!(listings -> users (vendor_id));
diesel::joinable!(orders -> listings (listing_id));
diesel::joinable!(transactions -> escrows (escrow_id));

diesel::allow_tables_to_appear_in_same_query!(
    escrows,
    listings,
    orders,
    reviews,
    transactions,
    users,
);
