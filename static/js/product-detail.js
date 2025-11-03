// Product Detail Page - Add to Cart functionality

// Add to cart function
async function addToCart(listingId) {
    if (!window.CartManager) {
        alert('Cart system is loading...');
        return;
    }

    const result = await CartManager.addToCart(listingId, 1);

    if (result.success) {
        // Show success notification
        showNotification('Produit ajouté au panier!', 'success');
    } else {
        // Show error notification
        showNotification(result.message || 'Erreur lors de l\'ajout au panier', 'error');
    }
}

// Simple notification function
function showNotification(message, type) {
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 100px;
        right: 20px;
        padding: 1rem 1.5rem;
        background-color: ${type === 'success' ? '#10b981' : '#ef4444'};
        color: white;
        border-radius: 4px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        z-index: 10000;
        font-size: 0.875rem;
        font-weight: 500;
        animation: slideIn 0.3s ease-out;
    `;

    document.body.appendChild(notification);

    setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease-out';
        setTimeout(() => notification.remove(), 300);
    }, 3000);
}

// Add animations CSS
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from {
            transform: translateX(400px);
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
            transform: translateX(400px);
            opacity: 0;
        }
    }
`;
document.head.appendChild(style);

// Initialize event listeners when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    // Add to cart button
    const addToCartBtn = document.querySelector('.btn-product-cart');
    if (addToCartBtn) {
        addToCartBtn.addEventListener('click', function() {
            const listingId = this.getAttribute('data-listing-id');
            if (listingId) {
                addToCart(listingId);
            }
        });
    }
});
