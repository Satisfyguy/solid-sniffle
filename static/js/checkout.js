// Checkout Page - 2/3 Multisig Monero Flow

class CheckoutFlow {
    constructor() {
        this.csrfToken = null;
        this.orderId = null;
        this.escrowId = null;
        this.escrowStatus = null;
        this.checkoutMode = 'cart';
        this.ws = null;
        this.paymentPoll = null;
    }

    /**
     * Initialize checkout flow
     */
    async init() {
        console.log('[Checkout] Initializing simplified checkout flow...');

        // Get data from hidden inputs
        this.csrfToken = document.getElementById('csrf-token')?.value;
        this.orderId = document.getElementById('order-id')?.value;
        this.escrowId = document.getElementById('escrow-id')?.value;
        this.escrowStatus = document.getElementById('escrow-status')?.value;
        this.checkoutMode = document.getElementById('checkout-mode')?.value || 'cart';

        console.log('[Checkout] Config:', {
            orderId: this.orderId,
            escrowId: this.escrowId,
            escrowStatus: this.escrowStatus,
            checkoutMode: this.checkoutMode
        });

        // Setup event listeners
        this.setupEventListeners();

        // Initialize Lucide icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }

        // Start appropriate flow based on state
        if (!this.escrowId || !this.orderId) {
            console.log('[Checkout] New checkout initiated');
            // Show shipping form (it's already visible by default)
            document.getElementById('shipping-address-form')?.style.removeProperty('display');
        } else if (this.escrowStatus === 'created' || this.escrowStatus === 'funded') {
            console.log('[Checkout] Escrow exists, showing payment instructions');
            // Hide shipping form, show payment
            document.getElementById('shipping-address-form').style.display = 'none';
            this.showPaymentInstructions();
            this.startPaymentMonitoring();
        } else if (this.escrowStatus === 'active') {
            console.log('[Checkout] Payment confirmed');
            // Hide shipping form, show confirmation
            document.getElementById('shipping-address-form').style.display = 'none';
            this.showPaymentConfirmed();
        }

        // Connect WebSocket for real-time updates
        this.connectWebSocket();
    }

    /**
     * Setup event listeners
     */
    setupEventListeners() {
        // Shipping form submission
        const shippingForm = document.getElementById('shipping-form');
        if (shippingForm) {
            shippingForm.addEventListener('submit', (e) => {
                e.preventDefault();
                this.submitShippingAddress();
            });
        }

        // Copy address button
        const copyBtn = document.getElementById('copy-address-btn');
        if (copyBtn) {
            copyBtn.addEventListener('click', () => this.copyMultisigAddress());
        }
    }

    /**
     * Submit shipping address
     */
    async submitShippingAddress() {
        console.log('[Checkout] Processing delivery information');

        // Gather form data
        const streetAddress = document.getElementById('street-address')?.value;
        const city = document.getElementById('city')?.value;
        const postalCode = document.getElementById('postal-code')?.value;
        const country = document.getElementById('country')?.value;
        const shippingNotes = document.getElementById('shipping-notes')?.value;

        if (!streetAddress || !city || !postalCode || !country) {
            this.showNotification('Veuillez remplir tous les champs obligatoires', 'error');
            return;
        }

        // Format address as JSON
        const shippingAddress = {
            street: streetAddress,
            city: city,
            postal_code: postalCode,
            country: country
        };

        try {
            // Disable submit button
            const submitBtn = document.getElementById('submit-shipping-btn');
            if (submitBtn) {
                submitBtn.disabled = true;
                submitBtn.innerHTML = '<i data-lucide="loader" class="animate-spin"></i><span>Traitement...</span>';
                if (typeof lucide !== 'undefined') lucide.createIcons();
            }

            // Create order with shipping address
            const response = await fetch('/api/orders/create', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': this.csrfToken
                },
                body: JSON.stringify({
                    checkout_mode: this.checkoutMode,
                    shipping_address: JSON.stringify(shippingAddress),
                    shipping_notes: shippingNotes || null
                })
            });

            const data = await response.json();

            if (response.ok && data.success) {
                console.log('[Checkout] Order created:', data.order_id);
                this.orderId = data.order_id;

                // Hide shipping form
                const shippingForm = document.getElementById('shipping-address-form');
                if (shippingForm) shippingForm.style.display = 'none';

                // Show notification
                this.showNotification('Informations enregistrées', 'success');

                // Proceed to escrow initialization
                await this.createOrderAndInitEscrow();
            } else {
                console.error('[Checkout] Order creation failed:', data);
                this.showNotification(data.message || 'Échec de la création de commande', 'error');

                // Re-enable button
                if (submitBtn) {
                    submitBtn.disabled = false;
                    submitBtn.innerHTML = '<i data-lucide="arrow-right"></i><span>Continuer vers le paiement</span>';
                    if (typeof lucide !== 'undefined') lucide.createIcons();
                }
            }
        } catch (error) {
            console.error('[Checkout] Shipping address submission error:', error);
            this.showNotification('Erreur lors de l\'enregistrement de l\'adresse', 'error');

            // Re-enable button
            const submitBtn = document.getElementById('submit-shipping-btn');
            if (submitBtn) {
                submitBtn.disabled = false;
                submitBtn.innerHTML = '<i data-lucide="arrow-right"></i><span>Continuer vers le paiement</span>';
                if (typeof lucide !== 'undefined') lucide.createIcons();
            }
        }
    }

    /**
     * Create order and initialize escrow
     */
    async createOrderAndInitEscrow() {
        console.log('[Checkout] Creating order and initializing escrow...');

        // Show escrow init UI
        document.getElementById('escrow-init')?.style.removeProperty('display');

        // Step 1: Create order if needed (already created with shipping address)
        if (!this.orderId) {
            console.error('[Checkout] No order ID - should have been created with shipping address');
            return;
        }

        // Step 2: Initialize escrow
        try {
            this.updateMultisigProgress('prepare', 'pending');

            const response = await fetch(`/api/orders/${this.orderId}/init-escrow`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': this.csrfToken
                }
            });

            const data = await response.json();

            if (response.ok && data.success) {
                console.log('[Checkout] Escrow initialized:', data);
                this.escrowId = data.escrow_id;
                this.escrowStatus = data.status;

                // Show progress through multisig setup
                this.simulateMultisigProgress();
            } else {
                console.error('[Checkout] Escrow init failed:', data);
                this.showNotification(data.message || 'Échec de l\'initialisation escrow', 'error');
            }
        } catch (error) {
            console.error('[Checkout] Escrow init error:', error);
            this.showNotification('Erreur lors de l\'initialisation escrow', 'error');
        }
    }

    /**
     * Create order from cart or listing
     */
    async createOrder() {
        console.log('[Checkout] Creating order (mode:', this.checkoutMode, ')');

        // For now, we need to handle cart mode
        // TODO: Implement actual order creation API call
        console.warn('[Checkout] Order creation not yet implemented - using placeholder');
        return null;
    }

    /**
     * Simulate multisig progress (UI updates)
     */
    async simulateMultisigProgress() {
        const steps = ['prepare', 'make', 'sync-r1', 'sync-r2', 'verify'];

        for (let i = 0; i < steps.length; i++) {
            await this.sleep(2000 + Math.random() * 1000);
            this.updateMultisigProgress(steps[i], 'complete');

            if (i < steps.length - 1) {
                this.updateMultisigProgress(steps[i + 1], 'pending');
            }
        }

        // Multisig complete - fetch escrow status
        await this.sleep(1000);
        await this.checkEscrowStatus();
    }

    /**
     * Update multisig progress UI
     */
    updateMultisigProgress(step, status) {
        const stepEl = document.getElementById(`step-${step}`);
        if (!stepEl) return;

        const icon = stepEl.querySelector('.progress-icon i');

        stepEl.classList.remove('pending', 'complete');
        stepEl.classList.add(status);

        if (status === 'pending') {
            icon.setAttribute('data-lucide', 'loader');
            icon.classList.add('animate-spin');
        } else if (status === 'complete') {
            icon.setAttribute('data-lucide', 'check-circle');
            icon.classList.remove('animate-spin');
        }

        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Check escrow status
     */
    async checkEscrowStatus() {
        if (!this.escrowId) return;

        try {
            const response = await fetch(`/api/escrow/${this.escrowId}/status`);
            const data = await response.json();

            if (response.ok) {
                console.log('[Checkout] Escrow status:', data);
                this.escrowStatus = data.status;

                if (data.multisig_address) {
                    // Escrow ready for payment
                    document.getElementById('escrow-init').style.display = 'none';
                    this.showPaymentInstructions();
                    this.startPaymentMonitoring();

                    // Update multisig address in UI
                    const addressInput = document.getElementById('multisig-address');
                    if (addressInput) {
                        addressInput.value = data.multisig_address;
                    }

                    // Update amount
                    const amountEl = document.getElementById('amount-xmr');
                    if (amountEl && data.amount) {
                        amountEl.textContent = (data.amount / 1000000000000).toFixed(12);
                    }

                    // Enable copy button
                    const copyBtn = document.getElementById('copy-address-btn');
                    if (copyBtn) {
                        copyBtn.disabled = false;
                    }
                }
            }
        } catch (error) {
            console.error('[Checkout] Error checking escrow status:', error);
        }
    }

    /**
     * Show payment instructions
     */
    showPaymentInstructions() {
        console.log('[Checkout] Showing payment instructions');

        // Hide escrow init
        const escrowInit = document.getElementById('escrow-init');
        if (escrowInit) escrowInit.style.display = 'none';

        // Show payment instructions
        const paymentInstructions = document.getElementById('payment-instructions');
        if (paymentInstructions) {
            paymentInstructions.style.display = 'block';
        }

        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Start payment monitoring
     */
    startPaymentMonitoring() {
        if (!this.escrowId) {
            console.warn('[Checkout] Cannot monitor payment: no escrow ID');
            return;
        }

        console.log('[Checkout] Starting payment monitoring for escrow:', this.escrowId);

        // Poll every 10 seconds
        this.paymentPoll = setInterval(async () => {
            await this.checkPaymentStatus();
        }, 10000);

        // Check immediately
        this.checkPaymentStatus();
    }

    /**
     * Check payment status
     */
    async checkPaymentStatus() {
        if (!this.escrowId) return;

        try {
            const response = await fetch(`/api/escrow/${this.escrowId}/status`);
            const data = await response.json();

            if (response.ok) {
                console.log('[Checkout] Payment status:', data.status);

                // Update confirmations if available
                if (data.confirmations !== undefined) {
                    this.updateConfirmations(data.confirmations);
                }

                // Check if payment confirmed
                if (data.status === 'active') {
                    console.log('[Checkout] Payment confirmed!');
                    clearInterval(this.paymentPoll);
                    this.showPaymentConfirmed();
                }
            }
        } catch (error) {
            console.error('[Checkout] Error checking payment status:', error);
        }
    }

    /**
     * Update confirmations UI
     */
    updateConfirmations(count) {
        const confirmationsCount = document.getElementById('confirmations-count');
        if (confirmationsCount) {
            confirmationsCount.textContent = count;
        }

        const confirmationsProgress = document.getElementById('confirmations-progress');
        if (confirmationsProgress) {
            const percentage = Math.min(100, (count / 10) * 100);
            confirmationsProgress.style.width = `${percentage}%`;
        }

        // Update status text
        const statusTitle = document.querySelector('#payment-status .status-title');
        if (statusTitle && count > 0) {
            statusTitle.textContent = `Paiement détecté (${count}/10 confirmations)`;
        }
    }

    /**
     * Show payment confirmed
     */
    showPaymentConfirmed() {
        console.log('[Checkout] Showing payment confirmed screen');

        // Hide payment instructions
        const paymentInstructions = document.getElementById('payment-instructions');
        if (paymentInstructions) paymentInstructions.style.display = 'none';

        // Show payment confirmed
        const paymentConfirmed = document.getElementById('payment-confirmed');
        if (paymentConfirmed) {
            paymentConfirmed.style.display = 'block';
        }

        // Show success notification
        this.showNotification('Paiement confirmé! Votre commande est en cours de traitement.', 'success');

        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Copy multisig address to clipboard
     */
    async copyMultisigAddress() {
        const addressInput = document.getElementById('multisig-address');
        if (!addressInput) return;

        const address = addressInput.value;

        try {
            await navigator.clipboard.writeText(address);
            console.log('[Checkout] Address copied to clipboard');
            this.showNotification('Adresse copiée!', 'success');

            // Visual feedback
            const copyBtn = document.getElementById('copy-address-btn');
            if (copyBtn) {
                const icon = copyBtn.querySelector('i');
                if (icon) {
                    icon.setAttribute('data-lucide', 'check');
                    if (typeof lucide !== 'undefined') {
                        lucide.createIcons();
                    }

                    setTimeout(() => {
                        icon.setAttribute('data-lucide', 'copy');
                        if (typeof lucide !== 'undefined') {
                            lucide.createIcons();
                        }
                    }, 2000);
                }
            }
        } catch (error) {
            console.error('[Checkout] Failed to copy address:', error);
            this.showNotification('Échec de la copie', 'error');
        }
    }

    /**
     * Connect WebSocket for real-time updates
     */
    connectWebSocket() {
        if (!this.escrowId) {
            console.log('[Checkout] Skipping WebSocket: no escrow ID yet');
            return;
        }

        try {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}/ws`;

            console.log('[Checkout] Connecting to WebSocket:', wsUrl);

            this.ws = new WebSocket(wsUrl);

            this.ws.onopen = () => {
                console.log('[Checkout] WebSocket connected');
            };

            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    console.log('[Checkout] WebSocket message:', message);

                    this.handleWebSocketMessage(message);
                } catch (error) {
                    console.error('[Checkout] WebSocket message parse error:', error);
                }
            };

            this.ws.onerror = (error) => {
                console.error('[Checkout] WebSocket error:', error);
            };

            this.ws.onclose = () => {
                console.log('[Checkout] WebSocket disconnected');

                // Reconnect after 5 seconds
                setTimeout(() => {
                    if (this.escrowId && this.escrowStatus !== 'active') {
                        console.log('[Checkout] Reconnecting WebSocket...');
                        this.connectWebSocket();
                    }
                }, 5000);
            };
        } catch (error) {
            console.error('[Checkout] WebSocket connection error:', error);
        }
    }

    /**
     * Handle WebSocket message
     */
    handleWebSocketMessage(message) {
        if (!message.event) return;

        switch (message.event) {
            case 'EscrowStatusChanged':
                if (message.escrow_id === this.escrowId) {
                    console.log('[Checkout] Escrow status changed:', message.new_status);
                    this.escrowStatus = message.new_status;

                    if (message.new_status === 'funded' && message.multisig_address) {
                        this.showPaymentInstructions();
                    } else if (message.new_status === 'active') {
                        this.showPaymentConfirmed();
                    }
                }
                break;

            case 'EscrowInit':
                if (message.escrow_id === this.escrowId) {
                    console.log('[Checkout] Escrow initialized via WebSocket');
                    this.checkEscrowStatus();
                }
                break;

            case 'PaymentDetected':
                if (message.escrow_id === this.escrowId) {
                    console.log('[Checkout] Payment detected:', message.confirmations);
                    this.updateConfirmations(message.confirmations);
                }
                break;

            default:
                console.log('[Checkout] Unhandled WebSocket event:', message.event);
        }
    }

    /**
     * Show notification
     */
    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `checkout-notification ${type}`;

        const icon = type === 'success' ? 'check-circle' : type === 'error' ? 'alert-circle' : 'info';

        notification.innerHTML = `
            <i data-lucide="${icon}"></i>
            <span>${message}</span>
        `;

        notification.style.cssText = `
            position: fixed;
            top: 100px;
            right: 20px;
            background-color: ${type === 'success' ? '#10b981' : type === 'error' ? '#ef4444' : '#3b82f6'};
            color: white;
            padding: 1rem 1.5rem;
            border-radius: 0.5rem;
            display: flex;
            align-items: center;
            gap: 0.75rem;
            box-shadow: 0 10px 25px rgba(0,0,0,0.2);
            z-index: 10000;
            animation: slideIn 0.3s ease-out;
            font-family: var(--font-body);
        `;

        document.body.appendChild(notification);

        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }

        setTimeout(() => {
            notification.style.animation = 'slideOut 0.3s ease-out';
            setTimeout(() => notification.remove(), 300);
        }, 4000);
    }

    /**
     * Sleep utility
     */
    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// Initialize checkout flow when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    console.log('[Checkout] DOM ready, initializing...');

    const checkout = new CheckoutFlow();
    checkout.init();

    // Store globally for debugging
    window.checkoutFlow = checkout;
});

// Add CSS animations
const style = document.createElement('style');
style.textContent = `
@keyframes slideIn {
    from {
        transform: translateX(100%);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}

@keyframes slideOut {
    from {
        transform: translateX(0);
        opacity: 1;
    }
    to {
        transform: translateX(100%);
        opacity: 0;
    }
}

.animate-spin {
    animation: spin 1s linear infinite;
}

@keyframes spin {
    from {
        transform: rotate(0deg);
    }
    to {
        transform: rotate(360deg);
    }
}
`;
document.head.appendChild(style);
