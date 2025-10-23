/**
 * Reputation WASM Verification Wrapper
 * @module reputation-verify
 * @version 0.1.0
 */

import init, {
    verify_reputation_file,
    verify_single_review,
    get_version
} from '/static/wasm/reputation_wasm.js';

let wasmInitialized = false;

export async function initWasm() {
    if (wasmInitialized) {
        console.log('WASM already initialized');
        return;
    }

    try {
        await init();
        wasmInitialized = true;
        const version = get_version();
        console.log(`Reputation WASM v${version} initialized`);
    } catch (error) {
        console.error('WASM initialization failed:', error);
        throw new Error(`Failed to initialize WASM: ${error.message}`);
    }
}

export async function verifyReputation(reputationObj) {
    await initWasm();

    try {
        const reputationJson = JSON.stringify(reputationObj);
        const result = verify_reputation_file(reputationJson);

        return {
            is_valid: result.is_valid,
            total_reviews: result.total_reviews,
            valid_signatures: result.valid_signatures,
            invalid_signatures: result.invalid_signatures,
            stats_match: result.stats_match,
            error_message: result.error_message
        };
    } catch (error) {
        console.error('Verification error:', error);
        return {
            is_valid: false,
            total_reviews: 0,
            valid_signatures: 0,
            invalid_signatures: 0,
            stats_match: false,
            error_message: `Verification exception: ${error.message}`
        };
    }
}

export async function verifySingleReview(reviewObj) {
    await initWasm();

    try {
        const reviewJson = JSON.stringify(reviewObj);
        return verify_single_review(reviewJson);
    } catch (error) {
        console.error('Review verification error:', error);
        return false;
    }
}

export function displayVerificationBadge(target, result) {
    const element = typeof target === 'string'
        ? document.getElementById(target)
        : target;

    if (element === null) {
        console.error('Badge target element not found');
        return;
    }

    const statsWarning = result.stats_match ? '' : ' Stats mismatch';

    if (result.is_valid) {
        element.innerHTML = '<div class="verification-badge verified"><span class="badge-icon">✅</span><span class="badge-text">Verified: ' + result.valid_signatures + ' reviews</span></div>';
        element.classList.add('verified');
        element.classList.remove('unverified');
    } else {
        element.innerHTML = '<div class="verification-badge unverified"><span class="badge-icon">⚠️</span><span class="badge-text">' + result.invalid_signatures + ' invalid signature(s)' + statsWarning + '</span></div>';
        element.classList.add('unverified');
        element.classList.remove('verified');
    }
}

export function isWasmSupported() {
    return typeof WebAssembly !== 'undefined';
}

export async function getWasmVersion() {
    await initWasm();
    return get_version();
}
