// Fund Escrow functionality
document.addEventListener('DOMContentLoaded', function() {
    const fundBtn = document.getElementById('fund-escrow-btn');
    const instructions = document.getElementById('escrow-instructions');
    const escrowAddressSpan = document.getElementById('escrow-address');
    const paymentStatus = document.getElementById('payment-status');
    const copyBtn = document.getElementById('copy-address-btn');
    
    if (!fundBtn) return;
    
    // Copy button handler
    if (copyBtn) {
        copyBtn.addEventListener('click', function() {
            copyToClipboard('escrow-address');
        });
    }
    
    const orderId = window.location.pathname.split('/').pop();
    const csrfToken = document.querySelector('meta[name="csrf-token"]')?.content || '';
    
    fundBtn.addEventListener('click', async function() {
        fundBtn.disabled = true;
        fundBtn.textContent = 'Initializing Escrow...';
        
        try {
            // Initialize escrow
            const response = await fetch(`/api/orders/${orderId}/init-escrow`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': csrfToken
                },
                credentials: 'same-origin'
            });
            
            // Check if response is JSON
            const contentType = response.headers.get('content-type');
            if (!contentType || !contentType.includes('application/json')) {
                const text = await response.text();
                console.error('Non-JSON response:', text);
                throw new Error('Server returned non-JSON response. Check console for details.');
            }
            
            const data = await response.json();
            
            if (response.ok) {
                // Show instructions with escrow address
                escrowAddressSpan.textContent = data.escrow_address;
                instructions.style.display = 'block';
                fundBtn.style.display = 'none';
                
                // Start monitoring for payment
                startPaymentMonitoring(orderId, data.escrow_id);
                
            } else {
                alert('Error: ' + (data.error || 'Failed to initialize escrow'));
                fundBtn.disabled = false;
                fundBtn.innerHTML = `
                    <svg class="icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                    </svg>
                    💰 Fund Escrow
                `;
            }
        } catch (error) {
            console.error('Error:', error);
            alert('Network error: ' + error.message);
            fundBtn.disabled = false;
        }
    });
});

function startPaymentMonitoring(orderId, escrowId) {
    const paymentStatus = document.getElementById('payment-status');
    
    paymentStatus.innerHTML = `
        <div style="padding: 10px; background: #1a1a3a; border: 1px solid #3b82f6; border-radius: 4px; text-align: center;">
            <p style="font-size: 11px; color: #3b82f6;">
                ⏳ Waiting for payment...<br>
                <span style="font-size: 10px; color: #888;">This may take a few minutes</span>
            </p>
        </div>
    `;
    
    // Poll every 10 seconds to check if payment received
    const checkInterval = setInterval(async () => {
        try {
            const response = await fetch(`/api/escrow/${escrowId}/status`, {
                credentials: 'same-origin'
            });
            
            const data = await response.json();
            
            if (data.status === 'funded') {
                clearInterval(checkInterval);
                paymentStatus.innerHTML = `
                    <div style="padding: 10px; background: #1a3a1a; border: 1px solid #22c55e; border-radius: 4px; text-align: center;">
                        <p style="font-size: 11px; color: #22c55e;">
                            ✅ Payment Received!<br>
                            <span style="font-size: 10px;">Refreshing page...</span>
                        </p>
                    </div>
                `;
                
                // Reload page after 2 seconds
                setTimeout(() => {
                    window.location.reload();
                }, 2000);
            }
        } catch (error) {
            console.error('Error checking payment status:', error);
        }
    }, 10000); // Check every 10 seconds
    
    // Stop checking after 30 minutes
    setTimeout(() => {
        clearInterval(checkInterval);
        paymentStatus.innerHTML = `
            <div style="padding: 10px; background: #3a2a1a; border: 1px solid #f59e0b; border-radius: 4px; text-align: center;">
                <p style="font-size: 11px; color: #f59e0b;">
                    ⚠️ Payment monitoring stopped<br>
                    <span style="font-size: 10px;">Refresh page to check status</span>
                </p>
            </div>
        `;
    }, 1800000); // 30 minutes
}

function copyToClipboard(elementId) {
    const element = document.getElementById(elementId);
    const text = element.textContent;
    
    navigator.clipboard.writeText(text).then(() => {
        // Show feedback
        const originalText = element.textContent;
        element.textContent = 'Copied!';
        element.style.color = '#22c55e';
        
        setTimeout(() => {
            element.textContent = originalText;
            element.style.color = '#f5f5f5';
        }, 2000);
    }).catch(err => {
        console.error('Failed to copy:', err);
        alert('Failed to copy to clipboard');
    });
}
