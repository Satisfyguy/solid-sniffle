/**
 * fund-escrow.js - Handles escrow funding flow for orders
 *
 * This script initializes escrow multisig and displays payment instructions
 * when the buyer clicks "Fund Escrow" button.
 */

document.addEventListener('DOMContentLoaded', function() {
    const fundBtn = document.getElementById('fund-escrow-btn');
    const instructionsDiv = document.getElementById('escrow-instructions');
    const escrowAddressSpan = document.getElementById('escrow-address');
    const copyAddressBtn = document.getElementById('copy-address-btn');
    const paymentStatusDiv = document.getElementById('payment-status');

    // Get order ID from current URL (/orders/{id})
    const orderId = window.location.pathname.split('/').pop();

    // Get CSRF token from hidden input, meta tag, or cookie
    function getCsrfToken() {
        // First, try to get from hidden input (checkout flow)
        const hiddenInput = document.getElementById('csrf-token');
        if (hiddenInput && hiddenInput.value) {
            return hiddenInput.value;
        }

        // Second, try meta tag
        const meta = document.querySelector('meta[name="csrf-token"]');
        if (meta) {
            return meta.getAttribute('content');
        }

        // Fallback: try to get from cookie
        const cookies = document.cookie.split(';');
        for (let cookie of cookies) {
            const [name, value] = cookie.trim().split('=');
            if (name === 'csrf_token') {
                return decodeURIComponent(value);
            }
        }

        console.error('CSRF token not found in hidden input, meta tag, or cookie');
        return null;
    }

    // If instructions div exists but no fund button, the escrow is already initialized
    // Show instructions automatically
    if (!fundBtn && instructionsDiv) {
        instructionsDiv.classList.add('show');
        console.log('Escrow already initialized, showing instructions');
        // Don't return, we need to set up the dev button handler below
    }

    // Initialize escrow when button clicked (only if button exists)
    if (fundBtn) {
        fundBtn.addEventListener('click', async function() {
        try {
            fundBtn.disabled = true;
            fundBtn.innerHTML = '<span style="opacity: 0.6;">⏳ Initializing escrow...</span>';

            // TEMPORARY: CSRF check disabled for testing database error
            // const csrfToken = getCsrfToken();
            // if (!csrfToken) {
            //     throw new Error('CSRF token not found. Please refresh the page.');
            // }

            const response = await fetch(`/api/orders/${orderId}/init-escrow`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                    // 'X-CSRF-Token': csrfToken
                }
            });

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.error || 'Failed to initialize escrow');
            }

            // Show payment instructions
            if (data.escrow_address && data.escrow_address !== 'Pending') {
                escrowAddressSpan.textContent = data.escrow_address;

                // Generate QR code for the escrow address
                const qrCanvas = document.getElementById('escrow-qr-code');
                if (qrCanvas && typeof QRCode !== 'undefined') {
                    QRCode.toCanvas(qrCanvas, data.escrow_address, {
                        width: 180,
                        margin: 1,
                        color: {
                            dark: '#000000',
                            light: '#FFFFFF'
                        }
                    }, function(error) {
                        if (error) {
                            console.error('QR Code generation error:', error);
                            qrCanvas.style.display = 'none';
                        } else {
                            console.log('QR Code generated successfully for escrow address');
                        }
                    });
                } else {
                    console.warn('QRCode library not loaded or canvas not found');
                }
            } else {
                escrowAddressSpan.textContent = 'Address generation in progress... Please wait.';
            }

            instructionsDiv.classList.add('show');
            fundBtn.style.display = 'none';

            // Enable copy button
            copyAddressBtn.addEventListener('click', function() {
                const address = escrowAddressSpan.textContent;

                if (address === 'Initializing...' || address.includes('in progress')) {
                    alert('⏳ Escrow address is still being generated. Please wait a moment.');
                    return;
                }

                navigator.clipboard.writeText(address).then(function() {
                    const originalText = copyAddressBtn.innerHTML;
                    copyAddressBtn.innerHTML = '✓ COPIED!';
                    copyAddressBtn.style.background = '#22c55e';
                    copyAddressBtn.style.borderColor = '#22c55e';

                    setTimeout(function() {
                        copyAddressBtn.innerHTML = originalText;
                        copyAddressBtn.style.background = '#2a2a2a';
                        copyAddressBtn.style.borderColor = '#3b82f6';
                    }, 2000);
                }).catch(function(err) {
                    console.error('Failed to copy address:', err);
                    alert('Failed to copy address. Please copy manually.');
                });
            });

            // Show payment status
            paymentStatusDiv.innerHTML = `
                <div style="padding: 10px; background: #1a3a1a; border: 1px solid #22c55e; border-radius: 4px;">
                    <p style="font-size: 11px; color: #22c55e; margin: 0;">
                        ✓ Escrow initialized (ID: ${data.escrow_id.substring(0, 8)}...)<br>
                        💡 Send payment to the address above to fund the escrow
                    </p>
                </div>
            `;

            // Start polling for payment detection (every 10 seconds)
            let pollCount = 0;
            const maxPolls = 60; // Poll for up to 10 minutes

            const pollInterval = setInterval(async function() {
                pollCount++;

                if (pollCount >= maxPolls) {
                    clearInterval(pollInterval);
                    paymentStatusDiv.innerHTML += `
                        <div style="padding: 10px; background: rgba(234, 179, 8, 0.1); border: 1px solid rgba(234, 179, 8, 0.3); border-radius: 4px; margin-top: 10px;">
                            <p style="font-size: 10px; color: rgba(234, 179, 8, 0.9); margin: 0;">
                                ⏱️ Polling timeout. Refresh the page to check payment status.
                            </p>
                        </div>
                    `;
                    return;
                }

                try {
                    const orderResponse = await fetch(`/api/orders/${orderId}`);
                    const orderData = await orderResponse.json();

                    if (orderData.status === 'funded') {
                        clearInterval(pollInterval);

                        paymentStatusDiv.innerHTML = `
                            <div style="padding: 10px; background: #1a3a1a; border: 1px solid #22c55e; border-radius: 4px;">
                                <p style="font-size: 11px; color: #22c55e; margin: 0; font-weight: bold;">
                                    ✓ PAYMENT RECEIVED!<br>
                                    🎉 Escrow is now funded. The vendor can ship your order.
                                </p>
                            </div>
                        `;

                        // Reload page after 2 seconds to show updated order status
                        setTimeout(() => location.reload(), 2000);
                    }
                } catch (err) {
                    console.error('Error polling order status:', err);
                    // Continue polling despite error
                }
            }, 10000); // Poll every 10 seconds

        } catch (error) {
            console.error('Error initializing escrow:', error);

            fundBtn.disabled = false;
            fundBtn.innerHTML = `
                <svg class="icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                💰 Fund Escrow
            `;

            // Show error message
            const errorDiv = document.createElement('div');
            errorDiv.style.cssText = 'padding: 10px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 4px; margin-top: 10px;';
            errorDiv.innerHTML = `
                <p style="font-size: 11px; color: rgba(239, 68, 68, 0.9); margin: 0;">
                    ❌ Error: ${error.message}<br>
                    Please try again or contact support.
                </p>
            `;

            if (instructionsDiv.style.display !== 'block') {
                fundBtn.parentElement.appendChild(errorDiv);

                // Remove error message after 5 seconds
                setTimeout(() => errorDiv.remove(), 5000);
            }
        }
        });
    } // End if (fundBtn)

    // DEV: Simulate Payment Button Handler
    // This allows testing the payment flow without real XMR transactions
    const devSimulateBtn = document.getElementById('dev-simulate-payment-btn');
    if (devSimulateBtn) {
        devSimulateBtn.addEventListener('click', async function() {
            try {
                // Confirm action
                if (!confirm('⚠️ DEV MODE: This will simulate a payment and mark the order as FUNDED.\n\nThis is for testing purposes only. Continue?')) {
                    return;
                }

                devSimulateBtn.disabled = true;
                devSimulateBtn.innerHTML = '⏳ Simulating payment...';

                const csrfToken = getCsrfToken();
                if (!csrfToken) {
                    throw new Error('CSRF token not found. Please refresh the page.');
                }

                const response = await fetch(`/api/orders/${orderId}/dev-simulate-payment`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-CSRF-Token': csrfToken
                    }
                });

                const data = await response.json();

                if (!response.ok) {
                    throw new Error(data.error || 'Failed to simulate payment');
                }

                // Show success message
                paymentStatusDiv.innerHTML = `
                    <div style="padding: 10px; background: #1a3a1a; border: 1px solid #22c55e; border-radius: 4px;">
                        <p style="font-size: 11px; color: #22c55e; margin: 0; font-weight: bold;">
                            ✓ PAYMENT SIMULATED SUCCESSFULLY!<br>
                            🎉 Order status: ${data.new_status.toUpperCase()}<br>
                            📦 The vendor can now ship your order.
                        </p>
                    </div>
                `;

                devSimulateBtn.style.display = 'none';

                // Reload page after 2 seconds to show updated order status
                setTimeout(() => {
                    location.reload();
                }, 2000);

            } catch (error) {
                console.error('Error simulating payment:', error);

                devSimulateBtn.disabled = false;
                devSimulateBtn.innerHTML = '🧪 DEV: SIMULATE PAYMENT (TESTNET ONLY)';

                // Show error message
                paymentStatusDiv.innerHTML = `
                    <div style="padding: 10px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 4px;">
                        <p style="font-size: 11px; color: rgba(239, 68, 68, 0.9); margin: 0;">
                            ❌ Error: ${error.message}<br>
                            Please try again or check the console for details.
                        </p>
                    </div>
                `;
            }
        });
    }
});
