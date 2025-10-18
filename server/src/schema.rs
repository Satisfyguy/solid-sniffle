// @generated automatically by Diesel CLI.

diesel::table! {
    escrows (id) {
        id -> Uuid,
        order_id -> Uuid,
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
    listings (id) {
        id -> Uuid,
        vendor_id -> Uuid,
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
        id -> Uuid,
        buyer_id -> Nullable<Uuid>,
        vendor_id -> Nullable<Uuid>,
        listing_id -> Nullable<Uuid>,
        escrow_id -> Nullable<Uuid>,
        status -> Text,
        total_xmr -> BigInt,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transactions (id) {
        id -> Uuid,
        escrow_id -> Uuid,
        tx_hash -> Nullable<Text>,
        amount_xmr -> BigInt,
        confirmations -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        password_hash -> Text,
        role -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(escrows -> orders (order_id));
// diesel::joinable!(escrows -> users (buyer_id)); // Commented out due to type mismatch (Text vs Uuid)
diesel::joinable!(listings -> users (vendor_id));
diesel::joinable!(orders -> escrows (escrow_id));
diesel::joinable!(orders -> listings (listing_id));
diesel::joinable!(orders -> users (buyer_id));
diesel::joinable!(orders -> users (vendor_id));
diesel::joinable!(transactions -> escrows (escrow_id));

diesel::allow_tables_to_appear_in_same_query!(
    escrows,
    listings,
    orders,
    transactions,
    users,
);