/**
 * XMR ↔ Atomic Units Converter
 *
 * Production-grade converter with real-time validation.
 * Respects design system: #C9A445 accent color.
 *
 * @module xmr-converter
 */

const XMR_TO_ATOMIC = 1_000_000_000_000; // 1 XMR = 10^12 piconeros
const MIN_XMR = 0.000000000001; // 1 piconero in XMR
const MAX_XMR = 18_400_000; // Total Monero supply

/**
 * Convert XMR to atomic units (piconeros)
 * @param {number|string} xmr - Amount in XMR
 * @returns {bigint} Amount in atomic units
 * @throws {Error} If invalid input
 */
function xmrToAtomic(xmr) {
    const amount = parseFloat(xmr);

    if (isNaN(amount)) {
        throw new Error('Invalid XMR amount: not a number');
    }

    if (amount < 0) {
        throw new Error('Invalid XMR amount: cannot be negative');
    }

    if (amount < MIN_XMR && amount !== 0) {
        throw new Error(`Amount too small: minimum is ${MIN_XMR} XMR (1 piconero)`);
    }

    if (amount > MAX_XMR) {
        throw new Error(`Amount too large: maximum is ${MAX_XMR} XMR`);
    }

    // Use BigInt for precision
    const atomicStr = (amount * XMR_TO_ATOMIC).toFixed(0);
    return BigInt(atomicStr);
}

/**
 * Convert atomic units to XMR
 * @param {number|string|bigint} atomic - Amount in atomic units
 * @returns {string} Amount in XMR (formatted with 12 decimals)
 * @throws {Error} If invalid input
 */
function atomicToXmr(atomic) {
    let atomicBigInt;

    try {
        atomicBigInt = BigInt(atomic);
    } catch (e) {
        throw new Error('Invalid atomic amount: not a valid integer');
    }

    if (atomicBigInt < 0n) {
        throw new Error('Invalid atomic amount: cannot be negative');
    }

    // Convert to number for division (safe for display purposes)
    const xmr = Number(atomicBigInt) / XMR_TO_ATOMIC;

    // Format with exactly 12 decimal places
    return xmr.toFixed(12);
}

/**
 * Format XMR amount for display (removes trailing zeros)
 * @param {string} xmr - XMR amount as string
 * @returns {string} Formatted XMR amount
 */
function formatXmr(xmr) {
    return parseFloat(xmr).toString();
}

/**
 * Format atomic amount for display (with thousands separators)
 * @param {bigint|string|number} atomic - Atomic amount
 * @returns {string} Formatted atomic amount
 */
function formatAtomic(atomic) {
    const atomicStr = atomic.toString();
    return atomicStr.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

/**
 * Initialize XMR converter for an input pair
 * @param {string} xmrInputId - ID of XMR input element
 * @param {string} atomicInputId - ID of atomic input element
 * @param {Object} options - Configuration options
 * @param {string} [options.displayId] - ID of display element for formatted amount
 * @param {Function} [options.onChange] - Callback when value changes
 */
function initXmrConverter(xmrInputId, atomicInputId, options = {}) {
    const xmrInput = document.getElementById(xmrInputId);
    const atomicInput = document.getElementById(atomicInputId);

    if (!xmrInput || !atomicInput) {
        console.error(`XMR Converter: Input elements not found (${xmrInputId}, ${atomicInputId})`);
        return;
    }

    const displayElement = options.displayId ? document.getElementById(options.displayId) : null;

    // State to prevent circular updates
    let isUpdating = false;

    /**
     * Update atomic input from XMR input
     */
    function updateFromXmr() {
        if (isUpdating) return;
        isUpdating = true;

        try {
            const xmrValue = xmrInput.value.trim();

            if (xmrValue === '' || xmrValue === '0') {
                atomicInput.value = '0';
                if (displayElement) displayElement.textContent = '0 XMR';
                clearError(xmrInput);
                if (options.onChange) options.onChange({ xmr: '0', atomic: '0' });
                return;
            }

            const atomic = xmrToAtomic(xmrValue);
            atomicInput.value = atomic.toString();

            if (displayElement) {
                const formatted = formatXmr(atomicToXmr(atomic));
                displayElement.textContent = `${formatted} XMR`;
            }

            clearError(xmrInput);

            if (options.onChange) {
                options.onChange({
                    xmr: xmrValue,
                    atomic: atomic.toString()
                });
            }

        } catch (error) {
            showError(xmrInput, error.message);
            if (displayElement) displayElement.textContent = 'Invalid';
        } finally {
            isUpdating = false;
        }
    }

    /**
     * Update XMR input from atomic input
     */
    function updateFromAtomic() {
        if (isUpdating) return;
        isUpdating = true;

        try {
            const atomicValue = atomicInput.value.trim();

            if (atomicValue === '' || atomicValue === '0') {
                xmrInput.value = '0';
                if (displayElement) displayElement.textContent = '0 XMR';
                clearError(atomicInput);
                if (options.onChange) options.onChange({ xmr: '0', atomic: '0' });
                return;
            }

            const xmr = atomicToXmr(atomicValue);
            const formatted = formatXmr(xmr);
            xmrInput.value = formatted;

            if (displayElement) {
                displayElement.textContent = `${formatted} XMR`;
            }

            clearError(atomicInput);

            if (options.onChange) {
                options.onChange({
                    xmr: formatted,
                    atomic: atomicValue
                });
            }

        } catch (error) {
            showError(atomicInput, error.message);
            if (displayElement) displayElement.textContent = 'Invalid';
        } finally {
            isUpdating = false;
        }
    }

    /**
     * Show error message below input
     */
    function showError(inputElement, message) {
        // Remove existing error
        clearError(inputElement);

        // Add error styles
        inputElement.style.borderColor = '#ef4444';

        // Create error message
        const errorDiv = document.createElement('div');
        errorDiv.className = 'xmr-converter-error';
        errorDiv.textContent = message;
        errorDiv.style.cssText = `
            color: #ef4444;
            font-size: 0.75rem;
            margin-top: 0.25rem;
            font-weight: 300;
        `;

        inputElement.parentElement.appendChild(errorDiv);
    }

    /**
     * Clear error message
     */
    function clearError(inputElement) {
        inputElement.style.borderColor = '';
        const existingError = inputElement.parentElement.querySelector('.xmr-converter-error');
        if (existingError) {
            existingError.remove();
        }
    }

    // Attach event listeners
    xmrInput.addEventListener('input', updateFromXmr);
    xmrInput.addEventListener('change', updateFromXmr);

    atomicInput.addEventListener('input', updateFromAtomic);
    atomicInput.addEventListener('change', updateFromAtomic);

    // Initialize with existing value
    if (atomicInput.value && atomicInput.value !== '0') {
        updateFromAtomic();
    } else if (xmrInput.value && xmrInput.value !== '0') {
        updateFromXmr();
    }

    console.log('✅ XMR Converter initialized:', { xmrInputId, atomicInputId });
}

/**
 * Create a visual converter widget
 * @param {string} containerId - ID of container element
 * @param {Object} options - Configuration options
 * @returns {Object} Widget API
 */
function createXmrConverterWidget(containerId, options = {}) {
    const container = document.getElementById(containerId);

    if (!container) {
        console.error(`XMR Converter Widget: Container not found (${containerId})`);
        return null;
    }

    const widgetId = `xmr-converter-${Date.now()}`;

    container.innerHTML = `
        <div class="xmr-converter-widget" style="
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid var(--color-border);
            border-radius: 4px;
            padding: 1.5rem;
            margin-bottom: 1rem;
        ">
            <div style="display: grid; grid-template-columns: 1fr auto 1fr; gap: 1rem; align-items: center;">
                <!-- XMR Input -->
                <div>
                    <label for="${widgetId}-xmr" style="
                        display: block;
                        font-size: 0.75rem;
                        font-weight: 600;
                        color: var(--color-foreground);
                        text-transform: uppercase;
                        letter-spacing: 0.05em;
                        margin-bottom: 0.5rem;
                    ">XMR</label>
                    <input
                        type="number"
                        id="${widgetId}-xmr"
                        step="0.000000000001"
                        min="0"
                        placeholder="0.000000000000"
                        style="
                            width: 100%;
                            padding: 0.75rem;
                            background: #1A1A1A;
                            border: 1px solid var(--color-border);
                            border-radius: 4px;
                            color: var(--color-accent);
                            font-family: monospace;
                            font-size: 1rem;
                        "
                    >
                </div>

                <!-- Swap Icon -->
                <div style="
                    color: var(--color-accent);
                    margin-top: 1.5rem;
                ">
                    <i data-lucide="arrow-left-right" style="width: 20px; height: 20px;"></i>
                </div>

                <!-- Atomic Input -->
                <div>
                    <label for="${widgetId}-atomic" style="
                        display: block;
                        font-size: 0.75rem;
                        font-weight: 600;
                        color: var(--color-foreground);
                        text-transform: uppercase;
                        letter-spacing: 0.05em;
                        margin-bottom: 0.5rem;
                    ">PICONEROS (ATOMIC)</label>
                    <input
                        type="number"
                        id="${widgetId}-atomic"
                        step="1"
                        min="0"
                        placeholder="0"
                        style="
                            width: 100%;
                            padding: 0.75rem;
                            background: #1A1A1A;
                            border: 1px solid var(--color-border);
                            border-radius: 4px;
                            color: var(--color-foreground);
                            font-family: monospace;
                            font-size: 1rem;
                        "
                    >
                </div>
            </div>

            <!-- Info Banner -->
            <div style="
                margin-top: 1rem;
                padding: 0.75rem;
                background: rgba(201, 164, 69, 0.1);
                border: 1px solid rgba(201, 164, 69, 0.3);
                border-radius: 4px;
                display: flex;
                gap: 0.75rem;
                align-items: center;
            ">
                <i data-lucide="info" style="width: 16px; height: 16px; color: var(--color-accent); flex-shrink: 0;"></i>
                <span style="font-size: 0.75rem; color: rgba(255, 255, 255, 0.7);">
                    1 XMR = 1,000,000,000,000 piconeros (atomic units)
                </span>
            </div>
        </div>
    `;

    // Initialize Lucide icons
    if (typeof lucide !== 'undefined') {
        lucide.createIcons();
    }

    // Initialize converter
    initXmrConverter(`${widgetId}-xmr`, `${widgetId}-atomic`, options);

    return {
        getXmr: () => document.getElementById(`${widgetId}-xmr`).value,
        getAtomic: () => document.getElementById(`${widgetId}-atomic`).value,
        setXmr: (value) => {
            document.getElementById(`${widgetId}-xmr`).value = value;
            document.getElementById(`${widgetId}-xmr`).dispatchEvent(new Event('input'));
        },
        setAtomic: (value) => {
            document.getElementById(`${widgetId}-atomic`).value = value;
            document.getElementById(`${widgetId}-atomic`).dispatchEvent(new Event('input'));
        }
    };
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        xmrToAtomic,
        atomicToXmr,
        formatXmr,
        formatAtomic,
        initXmrConverter,
        createXmrConverterWidget
    };
}
