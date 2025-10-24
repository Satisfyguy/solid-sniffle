document.addEventListener('DOMContentLoaded', function() {
    const orderForm = document.getElementById('order-form');
    if (!orderForm) return;

    const quantityInput = document.getElementById('quantity');
    const totalPriceSpan = document.getElementById('total-price');
    const unitPrice = parseFloat(orderForm.dataset.unitPrice) || 0;

    if (quantityInput && totalPriceSpan) {
        quantityInput.addEventListener('input', function() {
            const quantity = parseInt(this.value, 10) || 1;
            // Use a library for precise multiplication of currency in a real app
            const total = (unitPrice * quantity) / 1_000_000_000_000.0;
            totalPriceSpan.textContent = total.toFixed(12) + ' XMR';
        });
    }
});
