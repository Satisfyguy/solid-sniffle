<<<<<<< HEAD
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
=======
// static/js/xmr-converter.js

/**
 * XMR Converter Widget
 *
 * Provides real-time, bidirectional conversion between human-readable XMR and Monero atomic units (piconeros).
 * Uses BigInt to prevent floating-point errors.
 * Includes validation for min/max values and clear error messages.
 *
 * 1 XMR = 1,000,000,000,000 piconeros (10^12)
 */

const XMR_TO_ATOMIC_UNITS = BigInt(1_000_000_000_000); // 10^12
const MAX_XMR_SUPPLY = BigInt(18_400_000) * XMR_TO_ATOMIC_UNITS; // Approx 18.4 million XMR in atomic units

/**
 * Formats a BigInt atomic value to a human-readable XMR string.
 * @param {BigInt} atomicValue - The value in atomic units.
 * @returns {string} - The formatted XMR string.
 */
function formatAtomicToXmr(atomicValue) {
    if (atomicValue === BigInt(0)) {
        return "0.000000000000";
    }
    const sign = atomicValue < BigInt(0) ? "-" : "";
    const absAtomic = atomicValue < BigInt(0) ? -atomicValue : atomicValue;

    const integerPart = absAtomic / XMR_TO_ATOMIC_UNITS;
    let fractionalPart = absAtomic % XMR_TO_ATOMIC_UNITS;

    // Pad fractional part with leading zeros
    let fractionalString = fractionalPart.toString().padStart(12, '0');

    // Remove trailing zeros
    fractionalString = fractionalString.replace(/0+$/, '');

    return `${sign}${integerPart}.${fractionalString || '0'}`;
}

/**
 * Parses a human-readable XMR string to a BigInt atomic value.
 * @param {string} xmrString - The XMR string.
 * @returns {BigInt} - The value in atomic units.
 */
function parseXmrToAtomic(xmrString) {
    xmrString = xmrString.trim();
    if (!xmrString) {
        return BigInt(0);
    }

    const sign = xmrString.startsWith('-') ? -1 : 1;
    if (sign === -1) {
        xmrString = xmrString.substring(1);
    }

    const parts = xmrString.split('.');
    let integerPart = BigInt(parts[0] || '0');
    let fractionalPart = BigInt(0);

    if (parts.length > 1) {
        let fractionalString = parts[1].padEnd(12, '0').substring(0, 12);
        fractionalPart = BigInt(fractionalString);
    }

    return (integerPart * XMR_TO_ATOMIC_UNITS + fractionalPart) * BigInt(sign);
}

/**
 * Validates an atomic value.
 * @param {BigInt} atomicValue - The value in atomic units.
 * @returns {string|null} - Error message or null if valid.
 */
function validateAtomicValue(atomicValue) {
    if (atomicValue < BigInt(0)) {
        return "Le montant ne peut pas être négatif.";
    }
    if (atomicValue === BigInt(0)) {
        return "Le montant doit être supérieur à 0.";
    }
    if (atomicValue > MAX_XMR_SUPPLY) {
        return `Le montant dépasse l'offre maximale de Monero (${formatAtomicToXmr(MAX_XMR_SUPPLY)} XMR).`;
    }
    return null;
}

/**
 * Initializes an XMR converter widget.
 * @param {HTMLElement} xmrInput - The input element for human-readable XMR.
 * @param {HTMLElement} atomicInput - The input element for atomic units.
 * @param {HTMLElement} errorElement - The element to display error messages.
 */
function initXmrConverter(xmrInput, atomicInput, errorElement) {
    let lastValidAtomic = BigInt(0);

    const updateXmrFromAtomic = () => {
        const atomicValue = parseXmrToAtomic(atomicInput.value);
        const error = validateAtomicValue(atomicValue);
        if (error) {
            errorElement.textContent = error;
            xmrInput.value = ''; // Clear XMR input on error
            atomicInput.classList.add('input-error');
            xmrInput.classList.add('input-error');
        } else {
            errorElement.textContent = '';
            xmrInput.value = formatAtomicToXmr(atomicValue);
            lastValidAtomic = atomicValue;
            atomicInput.classList.remove('input-error');
            xmrInput.classList.remove('input-error');
        }
    };

    const updateAtomicFromXmr = () => {
        const xmrValue = xmrInput.value;
        const atomicValue = parseXmrToAtomic(xmrValue);
        const error = validateAtomicValue(atomicValue);
        if (error) {
            errorElement.textContent = error;
            atomicInput.value = ''; // Clear atomic input on error
            atomicInput.classList.add('input-error');
            xmrInput.classList.add('input-error');
        } else {
            errorElement.textContent = '';
            atomicInput.value = atomicValue.toString();
            lastValidAtomic = atomicValue;
            atomicInput.classList.remove('input-error');
            xmrInput.classList.remove('input-error');
        }
    };

    atomicInput.addEventListener('input', updateXmrFromAtomic);
    xmrInput.addEventListener('input', updateAtomicFromXmr);

    // Initial update
    updateAtomicFromXmr();
}

/**
 * Creates and initializes an XMR converter widget within a given container.
 * @param {HTMLElement} container - The container element for the widget.
 * @param {string} xmrInputName - The name attribute for the XMR input.
 * @param {string} atomicInputName - The name attribute for the atomic input.
 * @param {string} initialAtomicValue - The initial value for the atomic input (as string).
 */
function createXmrConverterWidget(container, xmrInputName, atomicInputName, initialAtomicValue = '0') {
    container.innerHTML = `
        <div class="xmr-converter-widget">
            <div class="xmr-converter-inputs">
                <div class="form-group">
                    <label for="${xmrInputName}" class="label">Price (XMR)</label>
                    <input
                        type="text"
                        id="${xmrInputName}"
                        name="${xmrInputName}"
                        placeholder="0.000000000000"
                        class="input xmr-input"
                        style="font-family: monospace;"
                    >
                </div>
                <div class="xmr-converter-icon">
                    <i data-lucide="arrow-left-right"></i>
                </div>
                <div class="form-group">
                    <label for="${atomicInputName}" class="label">Price (Atomic Units)</label>
                    <input
                        type="number"
                        id="${atomicInputName}"
                        name="${atomicInputName}"
                        placeholder="0"
                        min="1"
                        required
                        class="input atomic-input"
                        style="font-family: monospace;"
                        value="${initialAtomicValue}"
                        readonly
                    >
                </div>
            </div>
            <small class="xmr-converter-info">
                1 XMR = 1,000,000,000,000 piconeros
            </small>
            <div class="xmr-converter-error" style="color: var(--color-destructive); margin-top: 0.5rem;"></div>
        </div>
    `;

    const xmrInput = container.querySelector(`#${xmrInputName}`);
    const atomicInput = container.querySelector(`#${atomicInputName}`);
    const errorElement = container.querySelector('.xmr-converter-error');

    if (xmrInput && atomicInput && errorElement) {
        initXmrConverter(xmrInput, atomicInput, errorElement);
        // Ensure Lucide icons are created if they are used in the widget
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    } else {
        console.error("XMR Converter widget elements not found after creation.");
    }
}
>>>>>>> cd3680e (feat: Add new UI/UX features and ignore testnet data)
