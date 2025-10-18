// @generated manually to match migrations

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

diesel::joinable!(listings -> users (vendor_id));
diesel::joinable!(orders -> listings (listing_id));
diesel::joinable!(orders -> users (buyer_id));
diesel::joinable!(escrows -> orders (order_id));
diesel::joinable!(escrows -> users (buyer_id));
diesel::joinable!(transactions -> escrows (escrow_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    listings,
    orders,
    escrows,
    transactions,
);