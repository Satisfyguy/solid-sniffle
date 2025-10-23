/* tslint:disable */
/* eslint-disable */
/**
 * Initialize WASM module with panic hook for better debugging
 */
export function init(): void;
/**
 * Verify a complete vendor reputation file
 *
 * This is the main entry point for client-side verification.
 * It verifies:
 * 1. All signatures are valid
 * 2. Statistics match calculated values
 * 3. No tampering has occurred
 *
 * # Arguments
 * * `reputation_json` - JSON string of VendorReputation file
 *
 * # Returns
 * * `VerificationResult` - Detailed verification results
 *
 * # Example (JavaScript)
 * ```javascript
 * import init, { verify_reputation_file } from './reputation_wasm.js';
 *
 * await init();
 *
 * const reputationJson = await fetch('/api/reputation/vendor_id').then(r => r.text());
 * const result = verify_reputation_file(reputationJson);
 *
 * if (result.is_valid) {
 *   console.log(`✅ All ${result.total_reviews} reviews verified!`);
 * } else {
 *   console.error(`❌ Verification failed: ${result.error_message}`);
 * }
 * ```
 */
export function verify_reputation_file(reputation_json: string): VerificationResult;
/**
 * Verify a single review signature
 *
 * # Arguments
 * * `review_json` - JSON string of SignedReview
 *
 * # Returns
 * * `true` if signature is valid, `false` otherwise
 *
 * # Example (JavaScript)
 * ```javascript
 * const reviewJson = JSON.stringify({
 *   txid: "abc123",
 *   rating: 5,
 *   comment: "Great!",
 *   timestamp: "2025-10-22T12:00:00Z",
 *   buyer_pubkey: "base64_pubkey",
 *   signature: "base64_signature"
 * });
 *
 * const isValid = verify_single_review(reviewJson);
 * ```
 */
export function verify_single_review(review_json: string): boolean;
/**
 * Get version information
 */
export function get_version(): string;
/**
 * Result of reputation file verification
 */
export class VerificationResult {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Convert to JavaScript object for detailed inspection
   */
  toJSON(): any;
  readonly is_valid: boolean;
  readonly total_reviews: number;
  readonly valid_signatures: number;
  readonly invalid_signatures: number;
  readonly stats_match: boolean;
  readonly error_message: string | undefined;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_verificationresult_free: (a: number, b: number) => void;
  readonly verificationresult_is_valid: (a: number) => number;
  readonly verificationresult_total_reviews: (a: number) => number;
  readonly verificationresult_valid_signatures: (a: number) => number;
  readonly verificationresult_invalid_signatures: (a: number) => number;
  readonly verificationresult_stats_match: (a: number) => number;
  readonly verificationresult_error_message: (a: number, b: number) => void;
  readonly verificationresult_toJSON: (a: number) => number;
  readonly verify_reputation_file: (a: number, b: number) => number;
  readonly verify_single_review: (a: number, b: number) => number;
  readonly get_version: (a: number) => void;
  readonly init: () => void;
  readonly __wbindgen_export_0: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
