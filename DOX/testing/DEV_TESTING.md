# Development Testing Guide

## Simulating Escrow Payment (DEV ONLY)

Since you don't have real XMR for testing, here's how to simulate the payment flow:

### Method 1: Manual Database Update

```bash
# Mark order as funded manually
sqlite3 marketplace.db "UPDATE orders SET status = 'funded' WHERE id = 'YOUR_ORDER_ID';"
```

### Method 2: Create Dev Endpoint (Recommended)

Add this to your server for testing:

```rust
// In server/src/handlers/orders.rs

#[post("/orders/{id}/dev-fund")]
pub async fn dev_fund_order(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> impl Responder {
    #[cfg(debug_assertions)]
    {
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }))
        };
        
        match Order::update_status(&mut conn, id.into_inner(), OrderStatus::Funded) {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Order marked as funded (DEV MODE)"
            })),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update: {}", e)
            }))
        }
    }
    
    #[cfg(not(debug_assertions))]
    {
        HttpResponse::Forbidden().json(serde_json::json!({
            "error": "This endpoint is only available in debug mode"
        }))
    }
}
```

Then call it:
```bash
curl -X POST http://localhost:8080/api/orders/YOUR_ORDER_ID/dev-fund
```

### Method 3: Use Monero Testnet

1. Download Monero CLI (testnet)
2. Get free testnet XMR from faucet
3. Send to escrow address
4. System detects payment automatically

## Real Production Flow

In production with real XMR:

1. Buyer clicks "Fund Escrow"
2. Escrow address is generated
3. Buyer sends XMR from their wallet:
   ```
   monero-wallet-cli
   > transfer <ESCROW_ADDRESS> <AMOUNT>
   ```
4. System monitors blockchain
5. After confirmations, order status → funded
6. Vendor can ship

## Security Notes

- Never commit real wallet seeds/keys
- Use testnet for development
- Real escrow requires Monero daemon sync
- Multisig requires all parties to sign
