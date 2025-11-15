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
        recovery_mode -> Text,
        buyer_temp_wallet_id -> Nullable<Text>,
        vendor_temp_wallet_id -> Nullable<Text>,
        arbiter_temp_wallet_id -> Nullable<Text>,
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
        category -> Text,
    }
}

diesel::table! {
    order_messages (id) {
        id -> Text,
        order_id -> Text,
        sender_id -> Text,
        message -> Text,
        created_at -> Integer,
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
        shipping_address -> Nullable<Text>,
        shipping_notes -> Nullable<Text>,
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

diesel::table! {
    wallet_address_history (id) {
        id -> Text,
        user_id -> Text,
        old_address -> Nullable<Text>,
        new_address -> Text,
        changed_at -> Integer,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    wallet_rpc_configs (wallet_id) {
        wallet_id -> Nullable<Text>,
        escrow_id -> Text,
        role -> Text,
        rpc_url_encrypted -> Binary,
        rpc_user_encrypted -> Nullable<Binary>,
        rpc_password_encrypted -> Nullable<Binary>,
        created_at -> Integer,
        last_connected_at -> Nullable<Integer>,
        connection_attempts -> Integer,
        last_error -> Nullable<Text>,
    }
}

diesel::table! {
    multisig_round_state (id) {
        id -> Integer,
        escrow_id -> Text,
        round_number -> Integer,
        status -> Text,
        rpc_url -> Text,
        wallet_filename -> Text,
        role -> Text,
        multisig_info -> Nullable<Text>,
        started_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        last_error -> Nullable<Text>,
    }
}

diesel::joinable!(escrows -> orders (order_id));
diesel::joinable!(multisig_round_state -> escrows (escrow_id));
diesel::joinable!(listings -> users (vendor_id));
diesel::joinable!(order_messages -> orders (order_id));
diesel::joinable!(order_messages -> users (sender_id));
diesel::joinable!(orders -> listings (listing_id));
diesel::joinable!(transactions -> escrows (escrow_id));
diesel::joinable!(wallet_address_history -> users (user_id));
diesel::joinable!(wallet_rpc_configs -> escrows (escrow_id));

diesel::allow_tables_to_appear_in_same_query!(
    escrows,
    listings,
    multisig_round_state,
    order_messages,
    orders,
    reviews,
    transactions,
    users,
    wallet_address_history,
    wallet_rpc_configs,
);
