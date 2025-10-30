// Listings page JavaScript
// Handles image error fallback for IPFS images

document.addEventListener('DOMContentLoaded', function() {
    // Handle image loading errors for all product card images
    const productImages = document.querySelectorAll('.product-card-image img');

    productImages.forEach(function(img) {
        img.addEventListener('error', function() {
            // Hide the broken image
            this.style.display = 'none';

            // Show the fallback placeholder (next sibling)
            const placeholder = this.nextElementSibling;
            if (placeholder) {
                placeholder.style.display = 'flex';
            }
        });
    });
});
