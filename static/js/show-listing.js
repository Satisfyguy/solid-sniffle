document.addEventListener('DOMContentLoaded', function() {
    const orderForm = document.getElementById('order-form');
    if (!orderForm) return;

    const quantityInput = document.getElementById('quantity');
    const totalPriceSpan = document.getElementById('total-price');
    const unitPrice = parseFloat(orderForm.dataset.unitPrice) || 0;

    // Update total price when quantity changes
    if (quantityInput && totalPriceSpan) {
        quantityInput.addEventListener('input', function() {
            const quantity = parseInt(this.value, 10) || 1;
            const total = (unitPrice * quantity) / 1_000_000_000_000.0;
            totalPriceSpan.textContent = total.toFixed(12) + ' XMR';
        });
    }

    // Handle form submission
    orderForm.addEventListener('submit', async function(e) {
        e.preventDefault();
        
        const submitButton = orderForm.querySelector('button[type="submit"]');
        const resultDiv = document.getElementById('order-result');
        const listingId = orderForm.querySelector('input[name="listing_id"]').value;
        const quantity = parseInt(quantityInput.value, 10);

        // Get CSRF token from meta tag or cookie
        const csrfToken = document.querySelector('meta[name="csrf-token"]')?.content || '';

        submitButton.disabled = true;
        submitButton.textContent = 'CREATING ORDER...';
        resultDiv.innerHTML = '<p style="color: #888;">Creating escrow order...</p>';

        try {
            const response = await fetch('/api/orders', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': csrfToken,
                },
                body: JSON.stringify({
                    listing_id: listingId,
                    quantity: quantity
                }),
                credentials: 'same-origin'
            });

            const data = await response.json();

            if (response.ok) {
                resultDiv.innerHTML = `
                    <div style="padding: 15px; background: #1a3a1a; border: 1px solid #22c55e; border-radius: 4px;">
                        <p style="color: #22c55e; margin-bottom: 10px;">✓ ORDER CREATED SUCCESSFULLY</p>
                        <p style="color: #888; font-size: 11px;">Order ID: ${data.id}</p>
                        <p style="color: #888; font-size: 11px;">Total: ${data.total_display}</p>
                        <p style="color: #f5f5f5; margin-top: 10px;">Redirecting to escrow page...</p>
                    </div>
                `;
                
                // Redirect to orders page after 2 seconds
                setTimeout(() => {
                    window.location.href = '/orders/' + data.id;
                }, 2000);
            } else {
                resultDiv.innerHTML = `
                    <div style="padding: 15px; background: #3a1a1a; border: 1px solid #ef4444; border-radius: 4px;">
                        <p style="color: #ef4444;">✗ ERROR</p>
                        <p style="color: #888; font-size: 11px;">${data.error || 'Failed to create order'}</p>
                    </div>
                `;
                submitButton.disabled = false;
                submitButton.textContent = 'CREATE ESCROW ORDER →';
            }
        } catch (error) {
            resultDiv.innerHTML = `
                <div style="padding: 15px; background: #3a1a1a; border: 1px solid #ef4444; border-radius: 4px;">
                    <p style="color: #ef4444;">✗ NETWORK ERROR</p>
                    <p style="color: #888; font-size: 11px;">${error.message}</p>
                </div>
            `;
            submitButton.disabled = false;
            submitButton.textContent = 'CREATE ESCROW ORDER →';
        }
    });
});
