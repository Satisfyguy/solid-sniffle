/**
 * Shopping Cart JavaScript
 *
 * Handles:
 * - Quantity updates (increase/decrease/manual input)
 * - Item removal
 * - Cart clearing
 * - Dynamic total recalculation
 * - Cart badge counter in navigation
 */

(function() {
    'use strict';

    // Get CSRF token from meta tag or hidden input
    function getCsrfToken() {
        const meta = document.querySelector('meta[name="csrf-token"]');
        if (meta) return meta.getAttribute('content');

        const input = document.querySelector('input[name="csrf_token"]');
        if (input) return input.value;

        return '';
    }

    // Update cart badge counter in navigation
    function updateCartBadge() {
        fetch('/api/cart/count')
            .then(response => response.json())
            .then(data => {
                const badge = document.querySelector('.cart-badge');
                if (badge) {
                    badge.textContent = data.count;
                    badge.style.display = data.count > 0 ? 'flex' : 'none';
                }
            })
            .catch(err => console.error('Failed to update cart badge:', err));
    }

    // Update quantity via API
    async function updateQuantity(listingId, quantity) {
        try {
            const response = await fetch('/api/cart/update', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    listing_id: listingId,
                    quantity: parseInt(quantity),
                    csrf_token: getCsrfToken()
                })
            });

            const data = await response.json();

            if (data.success) {
                // Recalculate totals
                recalculateTotals(data.cart);
                updateCartBadge();
                return true;
            } else {
                alert(data.message || 'Failed to update quantity');
                return false;
            }
        } catch (err) {
            console.error('Update quantity error:', err);
            alert('Network error updating cart');
            return false;
        }
    }

    // Remove item from cart
    async function removeItem(listingId) {
        if (!confirm('Remove this item from your cart?')) {
            return;
        }

        try {
            const response = await fetch('/api/cart/remove', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    listing_id: listingId,
                    csrf_token: getCsrfToken()
                })
            });

            const data = await response.json();

            if (data.success) {
                // Remove item element from DOM
                const itemElement = document.querySelector(`.cart-item[data-listing-id="${listingId}"]`);
                if (itemElement) {
                    itemElement.remove();
                }

                // Recalculate totals
                if (data.cart.items.length === 0) {
                    // Reload page to show empty cart state
                    window.location.reload();
                } else {
                    recalculateTotals(data.cart);
                }

                updateCartBadge();
            } else {
                alert(data.message || 'Failed to remove item');
            }
        } catch (err) {
            console.error('Remove item error:', err);
            alert('Network error removing item');
        }
    }

    // Clear entire cart
    async function clearCart() {
        if (!confirm('Clear all items from your cart?')) {
            return;
        }

        try {
            const response = await fetch('/api/cart/clear', {
                method: 'POST',
                headers: {
                    'X-CSRF-Token': getCsrfToken()
                }
            });

            const data = await response.json();

            if (data.success) {
                window.location.reload();
            } else {
                alert('Failed to clear cart');
            }
        } catch (err) {
            console.error('Clear cart error:', err);
            alert('Network error clearing cart');
        }
    }

    // Recalculate totals from cart data
    function recalculateTotals(cart) {
        // Update item totals
        cart.items.forEach(item => {
            const itemElement = document.querySelector(`.cart-item[data-listing-id="${item.listing_id}"]`);
            if (itemElement) {
                const totalElement = itemElement.querySelector('.item-total');
                if (totalElement) {
                    const itemTotal = (item.unit_price_xmr * item.quantity) / 1000000000000;
                    totalElement.textContent = itemTotal.toFixed(12) + ' XMR';
                }

                // Update quantity input
                const quantityInput = itemElement.querySelector('.quantity-input');
                if (quantityInput && parseInt(quantityInput.value) !== item.quantity) {
                    quantityInput.value = item.quantity;
                }
            }
        });

        // Calculate and update cart totals
        const totalXmr = cart.items.reduce((sum, item) => {
            return sum + (item.unit_price_xmr * item.quantity);
        }, 0) / 1000000000000;

        const totalQuantity = cart.items.reduce((sum, item) => sum + item.quantity, 0);

        // Update total display
        const cartTotalElement = document.getElementById('cart-total');
        if (cartTotalElement) {
            cartTotalElement.textContent = totalXmr.toFixed(12) + ' XMR';
        }

        // Update summary (if exists in template variable scope)
        const summaryElements = document.querySelectorAll('[data-cart-summary]');
        summaryElements.forEach(el => {
            if (el.dataset.cartSummary === 'quantity') {
                el.textContent = totalQuantity;
            } else if (el.dataset.cartSummary === 'total') {
                el.textContent = totalXmr.toFixed(12) + ' XMR';
            }
        });
    }

    // Event handlers
    document.addEventListener('DOMContentLoaded', function() {
        // Quantity decrease buttons
        document.querySelectorAll('.quantity-decrease').forEach(button => {
            button.addEventListener('click', function() {
                const listingId = this.dataset.listingId;
                const input = document.querySelector(`.quantity-input[data-listing-id="${listingId}"]`);
                const currentQuantity = parseInt(input.value);

                if (currentQuantity > 1) {
                    updateQuantity(listingId, currentQuantity - 1);
                }
            });
        });

        // Quantity increase buttons
        document.querySelectorAll('.quantity-increase').forEach(button => {
            button.addEventListener('click', function() {
                const listingId = this.dataset.listingId;
                const input = document.querySelector(`.quantity-input[data-listing-id="${listingId}"]`);
                const currentQuantity = parseInt(input.value);

                updateQuantity(listingId, currentQuantity + 1);
            });
        });

        // Manual quantity input
        document.querySelectorAll('.quantity-input').forEach(input => {
            input.addEventListener('change', function() {
                const listingId = this.dataset.listingId;
                const quantity = parseInt(this.value);

                if (quantity >= 1) {
                    updateQuantity(listingId, quantity);
                } else {
                    // Reset to 1 if invalid
                    this.value = 1;
                    updateQuantity(listingId, 1);
                }
            });

            // Prevent non-numeric input
            input.addEventListener('keypress', function(e) {
                if (e.key && !/[0-9]/.test(e.key)) {
                    e.preventDefault();
                }
            });
        });

        // Remove item buttons
        document.querySelectorAll('.remove-item').forEach(button => {
            button.addEventListener('click', function() {
                const listingId = this.dataset.listingId;
                removeItem(listingId);
            });
        });

        // Clear cart button
        const clearCartButton = document.getElementById('clear-cart');
        if (clearCartButton) {
            clearCartButton.addEventListener('click', clearCart);
        }

        // Initialize cart badge
        updateCartBadge();
    });

    // Export for external use (e.g., adding to cart from listing pages)
    window.CartManager = {
        addToCart: async function(listingId, quantity) {
            if (!quantity) quantity = 1;

            try {
                const response = await fetch('/api/cart/add', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        listing_id: listingId,
                        quantity: quantity,
                        csrf_token: getCsrfToken()
                    })
                });

                const data = await response.json();

                if (data.success) {
                    updateCartBadge();
                    return { success: true, message: 'Added to cart!' };
                } else {
                    return { success: false, message: data.message };
                }
            } catch (err) {
                console.error('Add to cart error:', err);
                return { success: false, message: 'Network error' };
            }
        },

        updateBadge: updateCartBadge
    };

})();
