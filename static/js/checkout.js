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

        // DEV: Simulate payment button
        const devSimulateBtn = document.getElementById('dev-simulate-payment-btn');
        if (devSimulateBtn) {
            devSimulateBtn.addEventListener('click', () => this.devSimulatePayment());
        }

        // Fund Escrow button
        const fundEscrowBtn = document.getElementById('fund-escrow-btn');
        if (fundEscrowBtn) {
            fundEscrowBtn.addEventListener('click', () => this.checkPaymentManually());
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
     * Check payment manually - forces blockchain verification
     */
    async checkPaymentManually() {
        console.log('[Checkout] Manual payment check requested');

        if (!this.escrowId) {
            this.showNotification('Erreur: Aucun escrow trouvé', 'error');
            return;
        }

        try {
            // Disable button
            const btn = document.getElementById('fund-escrow-btn');
            if (btn) {
                btn.disabled = true;
                btn.innerHTML = '<i data-lucide="loader" class="animate-spin"></i><span>Vérification en cours...</span>';
                if (typeof lucide !== 'undefined') lucide.createIcons();
            }

            // Force check payment status
            await this.checkPaymentStatus();

            // Show notification
            this.showNotification('Vérification lancée - la confirmation peut prendre quelques minutes', 'info');

            // Re-enable button after 5 seconds
            setTimeout(() => {
                if (btn) {
                    btn.disabled = false;
                    btn.innerHTML = '<i data-lucide="wallet"></i><span>J\'ai envoyé les fonds - Vérifier le paiement</span>';
                    if (typeof lucide !== 'undefined') lucide.createIcons();
                }
            }, 5000);

        } catch (error) {
            console.error('[Checkout] Manual check error:', error);
            this.showNotification('Erreur lors de la vérification', 'error');

            // Re-enable button
            const btn = document.getElementById('fund-escrow-btn');
            if (btn) {
                btn.disabled = false;
                btn.innerHTML = '<i data-lucide="wallet"></i><span>J\'ai envoyé les fonds - Vérifier le paiement</span>';
                if (typeof lucide !== 'undefined') lucide.createIcons();
            }
        }
    }

    /**
     * DEV: Simulate payment (for testing without blockchain)
     */
    async devSimulatePayment() {
        console.log('[DEV] Simulating payment...');

        if (!this.orderId) {
            this.showNotification('Erreur: Aucune commande en cours', 'error');
            return;
        }

        try {
            // Disable button
            const btn = document.getElementById('dev-simulate-payment-btn');
            if (btn) {
                btn.disabled = true;
                btn.innerHTML = '<i data-lucide="loader" class="animate-spin"></i><span>Simulation en cours...</span>';
                if (typeof lucide !== 'undefined') lucide.createIcons();
            }

            // Call dev-simulate-payment endpoint
            const response = await fetch(`/api/orders/${this.orderId}/dev-simulate-payment`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': this.csrfToken
                }
            });

            const data = await response.json();

            if (response.ok && data.success) {
                console.log('[DEV] Payment simulated successfully');

                // Hide dev section
                const devSection = document.getElementById('dev-simulate-section');
                if (devSection) devSection.style.display = 'none';

                // Show payment confirmed
                this.showNotification('✅ Paiement simulé avec succès!', 'success');

                // Update to show payment confirmed state
                setTimeout(() => {
                    this.showPaymentConfirmed();
                }, 1000);

            } else {
                console.error('[DEV] Payment simulation failed:', data);
                this.showNotification(data.error || 'Échec de la simulation', 'error');

                // Re-enable button
                if (btn) {
                    btn.disabled = false;
                    btn.innerHTML = '<i data-lucide="zap"></i><span>DEV: Simuler le Paiement</span>';
                    if (typeof lucide !== 'undefined') lucide.createIcons();
                }
            }
        } catch (error) {
            console.error('[DEV] Simulation error:', error);
            this.showNotification('Erreur lors de la simulation', 'error');

            // Re-enable button
            const btn = document.getElementById('dev-simulate-payment-btn');
            if (btn) {
                btn.disabled = false;
                btn.innerHTML = '<i data-lucide="zap"></i><span>DEV: Simuler le Paiement</span>';
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

        // Start typewriter effect
        if (window.startMultisigTypewriter) {
            // console.log('[Checkout] Starting typewriter effect');
            window.startMultisigTypewriter();
        }

        // Step 1: Create order if needed (already created with shipping address)
        if (!this.orderId) {
            console.error('[Checkout] No order ID - should have been created with shipping address');
            return;
        }

        // Step 2: Initialize escrow
        try {
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

                // Show escrow-init section before simulating progress
                const escrowInitSection = document.getElementById('escrow-init');
                if (escrowInitSection) {
                    escrowInitSection.style.display = 'block';
                }

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
     * Now tracks REAL backend progress via polling
     */
    async simulateMultisigProgress() {
        // console.log('[Checkout] Starting realistic multisig progress tracking');

        // Step timing based on real observations (in seconds)
        // Total: ~115 seconds (1m 55s)
        const stepTimings = {
            prepare: { duration: 30, label: 'Generating multisig information...', percentage: 20 },
            make: { duration: 25, label: 'Building 2-of-3 wallet...', percentage: 40 },
            'sync-r1': { duration: 25, label: 'Exchanging sync information (round 1)...', percentage: 60 },
            'sync-r2': { duration: 25, label: 'Finalizing multisig wallet (round 2)...', percentage: 80 },
            verify: { duration: 10, label: 'Checking wallet consistency...', percentage: 100 }
        };

        const steps = ['prepare', 'make', 'sync-r1', 'sync-r2', 'verify'];
        const totalDuration = 115; // seconds
        const startTime = Date.now();

        let currentStepIndex = 0;

        // Start first step
        this.updateGlobalProgress(0, stepTimings[steps[0]].label, 0, totalDuration);

        // Poll backend every 2 seconds to check real progress
        const pollInterval = setInterval(async () => {
            const elapsed = Math.floor((Date.now() - startTime) / 1000);

            // Calculate which step we should be on based on elapsed time
            let cumulativeTime = 0;
            let expectedStep = 0;
            for (let i = 0; i < steps.length; i++) {
                cumulativeTime += stepTimings[steps[i]].duration;
                if (elapsed < cumulativeTime) {
                    expectedStep = i;
                    break;
                }
            }

            // Update UI if we've advanced to a new step
            if (expectedStep > currentStepIndex) {
                currentStepIndex = expectedStep;
            }

            // Update global progress bar
            const currentStep = steps[currentStepIndex];
            const stepInfo = stepTimings[currentStep];
            const percentage = stepInfo.percentage;
            const eta = Math.max(0, totalDuration - elapsed);

            this.updateGlobalProgress(percentage, stepInfo.label, elapsed, eta);

            // Check if escrow is actually ready
            if (elapsed >= totalDuration || currentStepIndex >= steps.length - 1) {
                try {
                    const response = await fetch(`/api/escrow/${this.escrowId}/status`);
                    const data = await response.json();

                    if (response.ok && data.multisig_address && data.multisig_address !== 'Pending') {
                        console.log('[Checkout] Multisig address ready!', data.multisig_address);

                        this.updateGlobalProgress(100, 'Multisig address generated!', elapsed, 0);

                        // Stop polling
                        clearInterval(pollInterval);

                        // Show payment instructions
                        await this.sleep(1000);
                        await this.checkEscrowStatus();
                    }
                } catch (error) {
                    console.error('[Checkout] Error checking escrow status:', error);
                }
            }

            // Safety timeout: stop after 3 minutes
            if (elapsed > 180) {
                console.warn('[Checkout] Multisig setup timeout - checking final status');
                clearInterval(pollInterval);
                await this.checkEscrowStatus();
            }
        }, 2000); // Poll every 2 seconds
    }

    /**
     * Update global progress bar
     */
    updateGlobalProgress(percentage, statusText, elapsed, eta) {
        const progressBarFill = document.getElementById('progress-bar-fill');
        const progressPercentage = document.getElementById('progress-percentage');
        const progressStatusText = document.getElementById('progress-status-text');
        const progressElapsed = document.getElementById('progress-elapsed');
        const progressEta = document.getElementById('progress-eta');

        if (progressBarFill) {
            progressBarFill.style.width = `${percentage}%`;
        }

        if (progressPercentage) {
            progressPercentage.textContent = `${percentage}%`;
        }

        if (progressStatusText) {
            progressStatusText.textContent = statusText;
        }

        if (progressElapsed) {
            const minutes = Math.floor(elapsed / 60);
            const seconds = elapsed % 60;
            progressElapsed.textContent = `Elapsed: ${minutes}m ${seconds}s`;
        }

        if (progressEta) {
            if (eta <= 0) {
                progressEta.textContent = 'Almost done...';
            } else {
                const minutes = Math.floor(eta / 60);
                const seconds = eta % 60;
                progressEta.textContent = `ETA: ~${minutes}m ${seconds}s`;
            }
        }
    }


    /**
     * Check escrow status
     */
    async checkEscrowStatus() {
        if (!this.escrowId) return;

        try {
            const response = await fetch(`/api/escrow/${this.escrowId}/status`, {
                credentials: 'include'
            });
            const data = await response.json();

            if (response.ok) {
                // console.log('[Checkout] Escrow status:', data);
                this.escrowStatus = data.status;

                // Show payment instructions even if address is "Pending" (for DEV mode)
                if (data.multisig_address || data.status === 'created') {
                    // Hide escrow init progress
                    const escrowInitSection = document.getElementById('escrow-init');
                    if (escrowInitSection) {
                        escrowInitSection.style.display = 'none';
                    }

                    // Show payment instructions
                    this.showPaymentInstructions();
                    this.startPaymentMonitoring();

                    // Update multisig address in UI (even if "Pending")
                    const addressInput = document.getElementById('multisig-address');
                    if (addressInput && data.multisig_address) {
                        addressInput.value = data.multisig_address;
                    }

                    // Update amount
                    const amountEl = document.getElementById('amount-xmr');
                    if (amountEl && data.amount) {
                        amountEl.textContent = (data.amount / 1000000000000).toFixed(12);
                    }

                    // Enable copy button only if address is real (not "Pending")
                    const copyBtn = document.getElementById('copy-address-btn');
                    if (copyBtn && data.multisig_address && data.multisig_address !== 'Pending') {
                        copyBtn.disabled = false;
                    }

                    // Generate QR code for multisig address (NON-CUSTODIAL Phase 6)
                    if (data.multisig_address && data.multisig_address !== 'Pending' && data.multisig_address.length === 95) {
                        this.generateQRCode(data.multisig_address);
                    }
                }
            }
        } catch (error) {
            console.error('[Checkout] Error checking escrow status:', error);
        }
    }

    /**
     * Generate QR code for multisig address (Phase 6: Non-custodial frontend)
     */
    generateQRCode(multisigAddress) {
        console.log('[Checkout] Generating QR code for multisig address');

        const qrcodeContainer = document.getElementById('qrcode');
        if (!qrcodeContainer) {
            console.warn('[Checkout] QR code container not found');
            return;
        }

        // Clear any existing QR code
        qrcodeContainer.innerHTML = '';

        // Generate QR code using QRCode.js library
        if (typeof QRCode !== 'undefined') {
            try {
                new QRCode(qrcodeContainer, {
                    text: multisigAddress,
                    width: 200,
                    height: 200,
                    colorDark: '#000000',
                    colorLight: '#ffffff',
                    correctLevel: QRCode.CorrectLevel.M
                });
                console.log('[Checkout] QR code generated successfully');
            } catch (error) {
                console.error('[Checkout] Error generating QR code:', error);
            }
        } else {
            console.error('[Checkout] QRCode library not loaded');
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
            const response = await fetch(`/api/escrow/${this.escrowId}/status`, {
                credentials: 'include'
            });
            const data = await response.json();

            if (response.ok) {
                // console.log('[Checkout] Payment status:', data.status);

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
                    // console.log('[Checkout] WebSocket message:', message);

                    this.handleWebSocketMessage(message);
                } catch (error) {
                    console.error('[Checkout] WebSocket message parse error:', error);
                }
            };

            this.ws.onerror = (error) => {
                console.error('[Checkout] WebSocket error:', error);
            };

            this.ws.onclose = () => {
                // console.log('[Checkout] WebSocket disconnected');

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

// =============================================================================
// TYPEWRITER EFFECT FOR MULTISIG PROGRESS
// =============================================================================

const funnyMessages = [
    "Multisig your potatoes",
    "Funds somewhere between here and there",
    "Escrow for your thoughts",
    "Mining your own business",
    "Proof of steak",
    "Consensus among vegetables",
    "Hashing it out with reality",
    "Your keys are probably fine",
    "Wallet.exe has stopped caring",
    "Nodes nodding off",
    "Transaction pending since birth",
    "Signatures collecting dust",
    "The blockchain suggests therapy",
    "Zero knowledge, zero problems",
    "Fork in the road ahead",
    "Mining for compliments",
    "Your balance is conceptually yours",
    "Escrow is just friendship with paperwork",
    "Three of three shrugging",
    "Cryptographically secure feelings",
    "Funds temporarily eternal",
    "The mempool of consciousness",
    "Validating your parking",
    "Keys under the digital doormat",
    "Mining your childhood trauma",
    "Consensus is a social construct",
    "Your coins exist theoretically",
    "Escrow in the middle of nowhere",
    "Multisig your vegetables first",
    "Proof of work-life balance",
    "Hashing out the details later",
    "Nodes having a moment",
    "Transaction lost in translation",
    "Signatures but no autographs",
    "The blockchain of command",
    "Zero confirmations, maximum doubt",
    "Fork it, we'll do it live",
    "Mining for meaning",
    "Your balance is having doubts",
    "Escrow with extra steps",
    "Two of three ain't bad",
    "Cryptographically probable",
    "Funds in a quantum state",
    "The mempool of broken dreams",
    "Validating your existence",
    "Keys to the kingdom of nothing",
    "Mining for approval",
    "Consensus pending approval",
    "Your coins in witness protection",
    "Escrow as a lifestyle choice",
    "Multisig your expectations",
    "Proof of trying",
    "Hashing browns for breakfast",
    "Nodes knowing nothing",
    "Transaction in a relationship",
    "Signatures without commitment",
    "The blockchain of memories",
    "Zero balance, full vibes",
    "Fork around and find out",
    "Mining your own grave",
    "Your balance on vacation",
    "Escrow is just spicy holding",
    "Three signatures, no witnesses",
    "Cryptographically confused",
    "Funds having an identity crisis",
    "The mempool of regret",
    "Validating parking tickets",
    "Keys to someone else's car",
    "Mining for bitcoin in a goldmine",
    "Consensus among chaos",
    "Your coins taking a break",
    "Escrow without the crow",
    "Multisig your feelings",
    "Proof of stake in society",
    "Hashing out childhood issues",
    "Nodes pretending to care",
    "Transaction having second thoughts",
    "Signatures with commitment issues",
    "The blockchain of evidence",
    "Zero effort, maximum security",
    "Fork this particular situation",
    "Mining compliments from strangers",
    "Your balance needs balance",
    "Escrow eating crow",
    "Three blind signatures",
    "Cryptographically challenged",
    "Funds fundamentally misunderstood",
    "The mempool of destiny",
    "Validating invalid validation",
    "Keys to the highway",
    "Mining for comedy gold",
    "Consensus is just peer pressure",
    "Your coins coining phrases",
    "Escrow and tell",
    "Multisig your homework",
    "Proof of proof",
    "Hashing hashtags",
    "Nodes being nodular",
    "Transaction pending since yesterday",
    "Signatures signing off",
    "The blockchain of custody",
    "Zero sum, negative game",
    "Fork in the disposal",
    "Mining for likes",
    "Your balance balanced badly",
    "Escrow growing slow",
    "Three signatures short of a quartet",
    "Cryptographically cryptic",
    "Funds funded by fun",
    "The mempool of mediocrity",
    "Validating the validators",
    "Keys to the city of nowhere",
    "Mining minor minerals",
    "Consensus consensually",
    "Your coins flipping themselves",
    "Escrow is tomorrow",
    "Multisig your sandwich",
    "Proof of pudding",
    "Hashing out the hash",
    "Nodes anonymously famous",
    "Transaction attracting inaction",
    "Signatures significantly insignificant",
    "The blockchain of jokes",
    "Zero to hero to zero",
    "Fork the police",
    "Mining minding its business",
    "Your balance imbalanced",
    "Escrow? More like es-later",
    "Three amigos, no signatures",
    "Cryptographically graphic",
    "Funds fun while they lasted",
    "The mempool of maybe",
    "Validating various velociraptors",
    "Keys to unlock nothing",
    "Mining underwater",
    "Consensus among NPCs",
    "Your coins in coin heaven",
    "Escrow escaping tomorrow",
    "Multisig your multisig",
    "Proof of poof",
    "Hashing rehashing old hash",
    "Nodes known to be unknown",
    "Transaction transacting badly",
    "Signatures signing in cursive"
];

let currentMessageIndex = 0;
let currentCharIndex = 0;
let typewriterInterval = null;

function typeNextChar() {
    const element = document.getElementById('multisig-typewriter');
    if (!element) {
        console.error('[Typewriter] Element #multisig-typewriter not found!');
        return;
    }

    const message = funnyMessages[currentMessageIndex];
    // console.log('[Typewriter] Message:', message, 'CharIndex:', currentCharIndex, '/', message.length);

    if (currentCharIndex < message.length) {
        element.textContent = message.substring(0, currentCharIndex + 1);
        // console.log('[Typewriter] Updated text to:', element.textContent);
        currentCharIndex++;
    } else {
        // Message complete, wait 3 seconds then start next message
        // console.log('[Typewriter] Message complete, waiting 3s for next...');
        clearInterval(typewriterInterval);
        setTimeout(() => {
            currentMessageIndex = (currentMessageIndex + 1) % funnyMessages.length;
            currentCharIndex = 0;
            element.textContent = '';
            typewriterInterval = setInterval(typeNextChar, 50);
        }, 3000);
    }
}

// Global function to start typewriter (called from CheckoutFlow)
window.startMultisigTypewriter = function() {
    // console.log('[Typewriter] Starting typewriter effect!');

    // Shuffle messages for variety
    funnyMessages.sort(() => Math.random() - 0.5);

    // Reset state
    currentMessageIndex = 0;
    currentCharIndex = 0;

    // Clear any existing interval
    if (typewriterInterval) {
        clearInterval(typewriterInterval);
    }

    // Start typing
    typewriterInterval = setInterval(typeNextChar, 50);
};
