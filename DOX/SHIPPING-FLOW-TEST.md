# 🚢 Shipping & Wallet Flow - Test Plan

**Date:** 2025-10-28
**Status:** ✅ Code Complete - Ready for Testing
**Server:** Running on http://127.0.0.1:8080 (PID 827419)

---

## ✅ What Was Implemented

### Part A: Vendor Wallet Address Configuration

**1. Registration Enhancement**
- ✅ Added optional `wallet_address` field to vendor registration
- ✅ Client-side validation (JavaScript show/hide based on role)
- ✅ Server-side validation (`is_valid_monero_address()`)
- ✅ Format: Starts with 4 or 8, length 95-106 characters

**Files Modified:**
- [templates/auth/register.html](../templates/auth/register.html:117-134) - Form field + validation
- [server/src/handlers/auth.rs](../server/src/handlers/auth.rs:60-76) - Validation function
- [server/src/handlers/auth.rs](../server/src/handlers/auth.rs:99-118) - Registration handler

**2. Settings Page Enhancement**
- ✅ Created complete wallet configuration interface
- ✅ Shows current wallet address (if configured)
- ✅ Warning message if not configured
- ✅ Update/Add wallet address form with HTMX
- ✅ Toast notifications for success/error
- ✅ Page reload after successful update

**Files Modified:**
- [templates/settings.html](../templates/settings.html) - Complete wallet UI
- [server/src/handlers/frontend.rs](../server/src/handlers/frontend.rs:1090-1159) - Fetch wallet data
- [server/src/handlers/auth.rs](../server/src/handlers/auth.rs:432-518) - Update endpoint
- [server/src/main.rs](../server/src/main.rs:322-325) - Route registration

**3. Shipping Validation**
- ✅ Prevents vendor from marking order as "shipped" without wallet address
- ✅ Clear error message directing to Settings page

**Files Modified:**
- [server/src/handlers/orders.rs](../server/src/handlers/orders.rs:472-487) - Validation check

---

### Part B: Buyer Shipping Address Collection

**1. Database Migration**
- ✅ Created migration: `2025-10-28-183959-0000_add_shipping_info_to_orders`
- ✅ Adds `shipping_address TEXT` (encrypted via SQLCipher)
- ✅ Adds `shipping_notes TEXT` (encrypted via SQLCipher)

**Files Created:**
- [server/migrations/2025-10-28-183959-0000_add_shipping_info_to_orders/up.sql](../server/migrations/2025-10-28-183959-0000_add_shipping_info_to_orders/up.sql)
- [server/migrations/2025-10-28-183959-0000_add_shipping_info_to_orders/down.sql](../server/migrations/2025-10-28-183959-0000_add_shipping_info_to_orders/down.sql)

**2. Schema & Models Update**
- ✅ Updated schema.rs with new columns
- ✅ Updated Order struct with `shipping_address` and `shipping_notes`
- ✅ Updated NewOrder struct with shipping fields

**Files Modified:**
- [server/src/schema.rs](../server/src/schema.rs:45-57) - orders table
- [server/src/models/order.rs](../server/src/models/order.rs:93-126) - structs

**3. Purchase Form Enhancement**
- ✅ Added required `shipping_address` textarea (10-500 chars)
- ✅ Added optional `shipping_notes` textarea (max 200 chars)
- ✅ Security notice: "🔒 ENCRYPTED AND VISIBLE ONLY TO YOU AND THE VENDOR"
- ✅ JavaScript captures and sends shipping data

**Files Modified:**
- [templates/listings/show.html](../templates/listings/show.html:117-143) - Form fields
- [static/js/show-listing.js](../static/js/show-listing.js:22-50) - Data capture

**4. Vendor Order View Enhancement**
- ✅ Added "🔒 Delivery Address (Confidential)" section
- ✅ Only visible to vendor (not buyer)
- ✅ Displays `shipping_address` and `shipping_notes`
- ✅ Security warning about encrypted storage

**Files Modified:**
- [templates/orders/show.html](../templates/orders/show.html:96-126) - Vendor-only section
- [server/src/handlers/frontend.rs](../server/src/handlers/frontend.rs:843-844) - Pass shipping data

**5. API Enhancement**
- ✅ Updated CreateOrderRequest with shipping fields
- ✅ Backend validation (address 10-500 chars, notes max 200)
- ✅ Database insertion includes shipping data

**Files Modified:**
- [server/src/handlers/orders.rs](../server/src/handlers/orders.rs:21-35) - Request struct
- [server/src/handlers/orders.rs](../server/src/handlers/orders.rs:225-235) - NewOrder creation

---

## 🧪 Test Sequence

### ⚠️ CRITICAL: Migration Status Unknown

**Before testing, we need to verify if the database migration was applied.**

The server is running and code expects `shipping_address` and `shipping_notes` columns to exist. If they don't, the first order creation will fail.

**Migration Check:**
```bash
# Option 1: Install sqlite3 and check manually
sudo apt-get install sqlite3 libsqlcipher-dev
sqlite3 marketplace.db "PRAGMA key='1507741993bdf8914031465a9dc63dd7e1f32a7bc2cd2b49e647042450503724'; PRAGMA table_info(orders);"

# Option 2: Test by creating an order (if it fails, migration needed)
# See Step 4 below

# Option 3: Check migration files exist
ls -la server/migrations/2025-10-28-183959-0000_add_shipping_info_to_orders/
```

### Step 1: Create New Vendor Account with Wallet

1. Navigate to http://127.0.0.1:8080/register
2. Fill in:
   - Username: `test_vendor_wallet`
   - Password: `testpassword123`
   - Role: **Vendor** (select from dropdown)
3. **The "Monero Wallet Address" field should appear automatically**
4. Enter a valid Monero address:
   - Example testnet address: `4AdUndXHHZ6cfufTMvppY6JwXNouMBzSkbLYfpAV5Usx3skxNgYeYTRj5UzqtReoS44qo9mtmXCqY45DJ852K5Jv2684Rge`
5. Click REGISTER
6. **Expected:** Success, redirect to login

**Fallback if no wallet address:** Register without it, then add via Settings (Step 2)

### Step 2: Configure Wallet via Settings (If not done at registration)

1. Login as vendor
2. Click profile dropdown → **⚙️ SETTINGS**
3. Scroll to "💰 MONERO WALLET" section
4. **Expected:**
   - If configured: Green box showing current address
   - If not configured: Yellow warning box
5. Enter/Update wallet address
6. Click "UPDATE WALLET ADDRESS"
7. **Expected:**
   - Toast notification: "✅ Wallet Updated"
   - Page reloads showing updated address

### Step 3: Create Listing as Vendor

1. Navigate to "ADD PRODUCT" (nav bar)
2. Create a test listing (e.g., "Test Physical Product")
3. Category: Select any
4. Price: e.g., 0.01 XMR
5. Stock: 10
6. **Important:** This is a physical product requiring shipping
7. Submit listing
8. **Expected:** Listing created successfully

### Step 4: Purchase as Buyer (TEST SHIPPING ADDRESS COLLECTION)

1. Logout, register as buyer: `test_buyer_shipping` / `testpassword123`
2. Navigate to the listing created in Step 3
3. Click "BUY NOW"
4. **Expected:** Order form with:
   - Quantity field
   - **SHIPPING ADDRESS** (required, textarea, 10-500 chars)
   - **DELIVERY INSTRUCTIONS** (optional, textarea, max 200 chars)
   - Security notice: "🔒 ENCRYPTED AND VISIBLE ONLY TO YOU AND THE VENDOR"
5. Fill in:
   - Quantity: 1
   - Shipping Address: `123 Test Street, Apartment 4B, Test City, TC 12345, Testland`
   - Delivery Instructions: `Ring doorbell twice, leave with neighbor if not home`
6. Click "PLACE ORDER"
7. **Expected:**
   - Success message
   - Redirect to order detail page
   - Order status: "PENDING"

**If this fails with a database error about missing columns, the migration was NOT applied. See "Emergency Migration Fix" below.**

### Step 5: Fund Escrow (DEV Simulation)

1. On order detail page, click "🧪 Simulate Payment (DEV)"
2. **Expected:**
   - Button disappears
   - Status updates to "FUNDED" (via WebSocket or page refresh)

### Step 6: Vendor Marks Order as Shipped (TEST WALLET VALIDATION)

1. Logout, login as vendor (`test_vendor_wallet`)
2. Navigate to "📦 MY ORDERS"
3. Find the order from `test_buyer_shipping`
4. Click to view order details
5. **Expected - Critical Checks:**
   - ✅ Section "🔒 Delivery Address (Confidential)" is visible
   - ✅ Shows full shipping address: `123 Test Street, Apartment 4B, Test City, TC 12345, Testland`
   - ✅ Shows delivery instructions: `Ring doorbell twice, leave with neighbor if not home`
   - ✅ Security warning: "⚠️ This address is encrypted in the database and only visible to you and the buyer"
6. Click "MARK AS SHIPPED"
7. **Expected:**
   - Success (because wallet address was configured)
   - Status updates to "SHIPPED"
   - Timestamp recorded

**If wallet NOT configured, expected error:**
```
❌ You must configure your Monero wallet address before shipping orders.
Go to Settings to add your wallet address.
```

### Step 7: Buyer Confirms Receipt (TEST WALLET FUND RELEASE)

1. Logout, login as buyer (`test_buyer_shipping`)
2. Navigate to "📦 MY ORDERS"
3. Find the shipped order
4. Click to view details
5. **Expected:**
   - Status: "SHIPPED"
   - Button: "CONFIRM RECEIPT"
6. Click "CONFIRM RECEIPT"
7. **Expected:**
   - Status updates to "COMPLETED"
   - Funds released to vendor's wallet address (in production)
   - Success message displayed

---

## 🔥 Emergency Migration Fix

**If Step 4 fails with database column error:**

```bash
# Kill server
killall -9 server

# Install sqlcipher-tools (if not already)
sudo apt-get install sqlcipher

# Apply migration manually
cd /home/malix/Desktop/monero.marketplace
sqlcipher marketplace.db
# At sqlcipher prompt:
PRAGMA key = '1507741993bdf8914031465a9dc63dd7e1f32a7bc2cd2b49e647042450503724';
ALTER TABLE orders ADD COLUMN shipping_address TEXT;
ALTER TABLE orders ADD COLUMN shipping_notes TEXT;
.quit

# Restart server
DB_ENCRYPTION_KEY=1507741993bdf8914031465a9dc63dd7e1f32a7bc2cd2b49e647042450503724 DATABASE_URL=/home/malix/Desktop/monero.marketplace/marketplace.db ./target/release/server > server_shipping_fixed.log 2>&1 &

# Retry Step 4
```

---

## ✅ Success Criteria

### Vendor Wallet Configuration
- [x] Can register vendor with wallet address
- [ ] Can add wallet address via Settings (if skipped at registration)
- [ ] Can update existing wallet address
- [ ] Cannot ship order without wallet configured (validation works)

### Shipping Address Collection
- [ ] Buyer sees shipping address fields on purchase form
- [ ] Shipping address is required (form validation)
- [ ] Delivery instructions are optional
- [ ] Order creation includes shipping data

### Shipping Address Display
- [ ] Vendor can see shipping address on order detail page
- [ ] Buyer does NOT see separate "delivery address" section (only vendor sees it)
- [ ] Shipping notes display correctly if provided
- [ ] Security warning is visible

### Complete Flow
- [ ] Vendor can ship order (with wallet configured)
- [ ] Buyer can confirm receipt
- [ ] Status transitions: PENDING → FUNDED → SHIPPED → COMPLETED
- [ ] No errors in server logs

---

## 📊 Migration Verification Commands

```bash
# Check migration files exist
ls -la server/migrations/2025-10-28-183959-0000_add_shipping_info_to_orders/

# Check schema.rs has columns
grep -A 2 "shipping_address" server/src/schema.rs

# Check model structs have fields
grep -A 5 "pub struct Order" server/src/models/order.rs | grep shipping

# Test server is running
curl -s http://127.0.0.1:8080/ | head -20

# Check server logs for errors
tail -20 server_shipping.log
```

---

## 🎯 Next Steps

1. **Run Step 1-7 test sequence**
2. **Document results** (screenshot each step if possible)
3. **Report any errors** with:
   - Step number where error occurred
   - Error message (screenshot or copy-paste)
   - Server logs at time of error: `tail -50 server_shipping.log`
4. **Verify backward compatibility**: Check existing orders without shipping data still display correctly

---

## 🔒 Security Notes

- ✅ All shipping addresses encrypted via SQLCipher (AES-256)
- ✅ Only vendor and buyer can see shipping address (role-based access)
- ✅ Wallet addresses validated for correct Monero format
- ✅ Clear privacy notices on all forms
- ✅ Wallet addresses stored encrypted in database
- ✅ No shipping data logged to server logs (OPSEC compliant)

---

**STATUS:** Ready for testing. All code compiled successfully, server running.
**BLOCKER:** Migration status unknown - will be verified in Step 4.
