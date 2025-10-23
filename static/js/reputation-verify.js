/**
 * Reputation Verification Module
 *
 * Client-side WASM-powered verification of vendor reputation files.
 *
 * Production-grade features:
 * - Zero trust: All signatures verified client-side
 * - Error handling with fallback
 * - Performance optimized with caching
 * - Security: No data sent to server for verification
 *
 * @module reputation-verify
 */

let wasmModule = null;
let wasmInitialized = false;

/**
 * Initialize WASM module
 *
 * Must be called before any verification.
 * Safe to call multiple times (idempotent).
 *
 * @returns {Promise<void>}
 * @throws {Error} If WASM fails to load
 */
export async function initWasm() {
    if (wasmInitialized) {
        return;
    }

    try {
        const { default: init, verify_reputation_file, verify_single_review, get_version } =
            await import('/static/wasm/reputation_wasm.js');

        await init();

        wasmModule = {
            verify_reputation_file,
            verify_single_review,
            get_version,
        };

        wasmInitialized = true;

        const version = wasmModule.get_version();
        console.log(`✅ Reputation WASM v${version} initialized`);

    } catch (error) {
        console.error('❌ Failed to initialize WASM:', error);
        throw new Error(`WASM initialization failed: ${error.message}`);
    }
}

/**
 * Verify a complete vendor reputation file
 *
 * @param {Object|string} reputation - VendorReputation object or JSON string
 * @returns {Promise<VerificationResult>}
 *
 * @typedef {Object} VerificationResult
 * @property {boolean} is_valid - Overall verification status
 * @property {number} total_reviews - Total number of reviews
 * @property {number} valid_signatures - Number of valid signatures
 * @property {number} invalid_signatures - Number of invalid signatures
 * @property {boolean} stats_match - Whether stats match calculated values
 * @property {string|null} error_message - Error message if verification failed
 *
 * @example
 * const reputation = await fetch('/api/reputation/vendor_id').then(r => r.json());
 * const result = await verifyReputation(reputation);
 *
 * if (result.is_valid) {
 *   console.log(`✅ All ${result.total_reviews} reviews verified!`);
 * } else {
 *   console.error(`❌ Verification failed: ${result.error_message}`);
 * }
 */
export async function verifyReputation(reputation) {
    if (!wasmInitialized) {
        await initWasm();
    }

    try {
        const reputationJson = typeof reputation === 'string'
            ? reputation
            : JSON.stringify(reputation);

        const result = wasmModule.verify_reputation_file(reputationJson);

        return {
            is_valid: result.is_valid,
            total_reviews: result.total_reviews,
            valid_signatures: result.valid_signatures,
            invalid_signatures: result.invalid_signatures,
            stats_match: result.stats_match,
            error_message: result.error_message,
        };

    } catch (error) {
        console.error('Verification error:', error);
        return {
            is_valid: false,
            total_reviews: 0,
            valid_signatures: 0,
            invalid_signatures: 0,
            stats_match: false,
            error_message: error.message,
        };
    }
}

/**
 * Verify a single review signature
 *
 * @param {Object|string} review - SignedReview object or JSON string
 * @returns {Promise<boolean>} True if signature is valid
 *
 * @example
 * const review = {
 *   txid: "abc123",
 *   rating: 5,
 *   comment: "Great!",
 *   timestamp: "2025-10-22T12:00:00Z",
 *   buyer_pubkey: "base64_pubkey",
 *   signature: "base64_signature"
 * };
 *
 * const isValid = await verifySingleReview(review);
 */
export async function verifySingleReview(review) {
    if (!wasmInitialized) {
        await initWasm();
    }

    try {
        const reviewJson = typeof review === 'string'
            ? review
            : JSON.stringify(review);

        return wasmModule.verify_single_review(reviewJson);

    } catch (error) {
        console.error('Single review verification error:', error);
        return false;
    }
}

/**
 * Display verification badge on page
 *
 * Updates DOM element with verification status.
 *
 * @param {string} elementId - ID of element to update
 * @param {VerificationResult} result - Verification result
 */
export function displayVerificationBadge(elementId, result) {
    const element = document.getElementById(elementId);
    if (!element) {
        console.warn(`Element #${elementId} not found`);
        return;
    }

    if (result.is_valid) {
        element.innerHTML = `
            <div class="verification-badge verified">
                <span class="icon">✅</span>
                <span class="text">Verified</span>
                <span class="count">${result.valid_signatures} review${result.valid_signatures !== 1 ? 's' : ''}</span>
            </div>
        `;
        element.classList.add('verified');
        element.classList.remove('unverified');
    } else {
        element.innerHTML = `
            <div class="verification-badge unverified">
                <span class="icon">⚠️</span>
                <span class="text">Unverified</span>
                <span class="error">${result.error_message || 'Signature verification failed'}</span>
            </div>
        `;
        element.classList.add('unverified');
        element.classList.remove('verified');
    }
}

/**
 * Auto-verify reputation on page load
 *
 * Looks for elements with data-reputation-verify attribute
 * and automatically verifies them.
 *
 * @example HTML
 * <div
 *   id="vendor-badge"
 *   data-reputation-verify
 *   data-vendor-id="vendor_uuid">
 * </div>
 */
export async function autoVerifyOnPage() {
    const elements = document.querySelectorAll('[data-reputation-verify]');

    for (const element of elements) {
        const vendorId = element.dataset.vendorId;
        if (!vendorId) {
            console.warn('Element missing data-vendor-id:', element);
            continue;
        }

        try {
            // Fetch reputation from API
            const response = await fetch(`/api/reputation/${vendorId}`);

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const reputation = await response.json();

            // Verify client-side
            const result = await verifyReputation(reputation);

            // Display badge
            displayVerificationBadge(element.id, result);

        } catch (error) {
            console.error(`Failed to verify vendor ${vendorId}:`, error);
            element.innerHTML = `
                <div class="verification-badge error">
                    <span class="icon">❌</span>
                    <span class="text">Verification Error</span>
                </div>
            `;
        }
    }
}

// Auto-initialize on DOM ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        initWasm().catch(console.error);
    });
} else {
    initWasm().catch(console.error);
}
