// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password_hash -> Text,
        role -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    orders (id) {
        id -> Text,
        buyer_id -> Nullable<Text>,
        vendor_id -> Nullable<Text>,
        listing_id -> Nullable<Text>,
        escrow_id -> Nullable<Text>,
        status -> Text,
        total_xmr -> BigInt,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    escrows (id) {
        id -> Text,
        order_id -> Text,
        buyer_wallet_info -> Nullable<Text>,
        vendor_wallet_info -> Nullable<Text>,
        arbiter_wallet_info -> Nullable<Text>,
        multisig_address -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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

diesel::allow_tables_to_appear_in_same_query!(users, listings, orders, escrows, transactions,);
