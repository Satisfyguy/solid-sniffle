/**
 * Listing Preview Component
 *
 * Real-time preview of product listing during creation.
 * Production-grade with debouncing, image preview, and XSS protection.
 *
 * @module listing-preview
 */

class ListingPreview {
    /**
     * Create a listing preview instance
     * @param {Object} options - Configuration options
     * @param {string} options.formId - Form ID to watch
     * @param {string} options.previewContainerId - Preview container ID
     * @param {string} [options.vendorName='You'] - Vendor display name
     */
    constructor(options = {}) {
        this.formId = options.formId || 'create-listing-form';
        this.previewContainerId = options.previewContainerId || 'listing-preview';
        this.vendorName = options.vendorName || 'You';

        this.form = null;
        this.previewContainer = null;
        this.debounceTimers = {};
        this.selectedImages = [];

        this.init();
    }

    /**
     * Initialize preview component
     */
    init() {
        this.form = document.getElementById(this.formId);
        this.previewContainer = document.getElementById(this.previewContainerId);

        if (!this.form || !this.previewContainer) {
            console.error('Listing preview: form or container not found');
            return;
        }

        this.attachEventListeners();
        this.renderPreview();

        console.log('✅ Listing preview initialized');
    }

    /**
     * Attach event listeners to form fields
     */
    attachEventListeners() {
        // Title input
        const titleInput = this.form.querySelector('#title');
        if (titleInput) {
            titleInput.addEventListener('input', () => this.debouncedUpdate('title'));
        }

        // Description textarea
        const descriptionInput = this.form.querySelector('#description');
        if (descriptionInput) {
            descriptionInput.addEventListener('input', () => this.debouncedUpdate('description'));
        }

        // Category select
        const categoryInput = this.form.querySelector('#category');
        if (categoryInput) {
            categoryInput.addEventListener('change', () => this.updatePreview());
        }

        // Price input (XMR human-readable)
        const priceInput = this.form.querySelector('#price-xmr-human');
        if (priceInput) {
            priceInput.addEventListener('input', () => this.debouncedUpdate('price'));
        }

        // Stock input
        const stockInput = this.form.querySelector('#stock');
        if (stockInput) {
            stockInput.addEventListener('input', () => this.debouncedUpdate('stock'));
        }

        // Image file input
        const imagesInput = this.form.querySelector('#images');
        if (imagesInput) {
            imagesInput.addEventListener('change', (e) => this.handleImageSelection(e));
        }
    }

    /**
     * Debounced update to prevent excessive re-renders
     * @param {string} field - Field identifier for debouncing
     */
    debouncedUpdate(field) {
        clearTimeout(this.debounceTimers[field]);
        this.debounceTimers[field] = setTimeout(() => {
            this.updatePreview();
        }, 300);
    }

    /**
     * Handle image file selection
     * @param {Event} event - Change event from file input
     */
    handleImageSelection(event) {
        const files = Array.from(event.target.files);

        // Clear previous images
        this.selectedImages = [];

        // Validate and preview each file
        files.slice(0, 10).forEach(file => {
            if (!file.type.startsWith('image/')) {
                return;
            }

            if (file.size > 5 * 1024 * 1024) {
                console.warn(`Image ${file.name} is too large (max 5MB)`);
                return;
            }

            // Create preview URL
            const reader = new FileReader();
            reader.onload = (e) => {
                this.selectedImages.push({
                    name: file.name,
                    url: e.target.result
                });

                // Update preview after first image is loaded
                if (this.selectedImages.length === 1) {
                    this.updatePreview();
                }
            };
            reader.readAsDataURL(file);
        });

        // Update immediately to show placeholder if no images
        if (files.length === 0) {
            this.updatePreview();
        }
    }

    /**
     * Update the preview card
     */
    updatePreview() {
        const data = this.getFormData();
        this.renderPreview(data);
    }

    /**
     * Get current form data
     * @returns {Object} Form data
     */
    getFormData() {
        return {
            title: this.form.querySelector('#title')?.value || '',
            description: this.form.querySelector('#description')?.value || '',
            category: this.form.querySelector('#category')?.value || '',
            priceXmr: this.form.querySelector('#price-xmr-human')?.value || '',
            stock: this.form.querySelector('#stock')?.value || '',
            images: this.selectedImages
        };
    }

    /**
     * Render the preview card
     * @param {Object} [data] - Optional form data to render
     */
    renderPreview(data) {
        if (!data) {
            data = this.getFormData();
        }

        const hasImage = data.images && data.images.length > 0;
        const hasTitle = data.title.trim().length > 0;
        const hasDescription = data.description.trim().length > 0;
        const hasCategory = data.category.length > 0;
        const hasPrice = data.priceXmr.length > 0;
        const hasStock = data.stock.length > 0;

        // Truncate description for card
        const truncatedDescription = data.description.length > 150
            ? data.description.substring(0, 150) + '...'
            : data.description;

        // Format price
        const formattedPrice = hasPrice
            ? `${parseFloat(data.priceXmr).toFixed(12)} XMR`
            : '0.000000000000 XMR';

        // Format category display
        const categoryDisplay = this.formatCategory(data.category);

        // Stock indicator
        let stockClass = '';
        let stockText = 'In Stock';
        if (hasStock) {
            const stockNum = parseInt(data.stock);
            if (stockNum === 0) {
                stockClass = 'out-of-stock';
                stockText = 'Out of Stock';
            } else if (stockNum < 5) {
                stockClass = 'low-stock';
                stockText = `Only ${stockNum} left`;
            } else {
                stockText = `${stockNum} available`;
            }
        }

        this.previewContainer.innerHTML = `
            <div class="listing-preview-card">
                <!-- Image -->
                <div class="listing-preview-image">
                    ${hasImage ? `
                        <img src="${data.images[0].url}" alt="${this.escapeHtml(data.title || 'Product')}">
                    ` : `
                        <div class="listing-preview-image-placeholder">
                            <i data-lucide="image"></i>
                            <span class="listing-preview-image-placeholder-text">No image selected</span>
                        </div>
                    `}
                    <div class="listing-preview-image-overlay">
                        <div>
                            <span class="listing-preview-category-badge ${!hasCategory ? 'placeholder' : ''}">
                                ${hasCategory ? categoryDisplay : 'No Category'}
                            </span>
                        </div>
                        <div>
                            <div class="listing-preview-rating">
                                <i data-lucide="star"></i>
                                <span>New</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Content -->
                <div class="listing-preview-content">
                    <h3 class="listing-preview-card-title ${!hasTitle ? 'placeholder' : ''}">
                        ${hasTitle ? this.escapeHtml(data.title) : 'Product Title'}
                    </h3>
                    <p class="listing-preview-card-description ${!hasDescription ? 'placeholder' : ''}">
                        ${hasDescription ? this.escapeHtml(truncatedDescription) : 'Product description will appear here...'}
                    </p>

                    <div class="listing-preview-card-info">
                        <div class="listing-preview-info-item">
                            <span class="listing-preview-info-label">Vendor</span>
                            <span class="listing-preview-info-value">${this.escapeHtml(this.vendorName)}</span>
                        </div>
                        <div class="listing-preview-info-item">
                            <span class="listing-preview-info-label">Category</span>
                            <span class="listing-preview-info-value ${!hasCategory ? 'placeholder' : ''}">
                                ${hasCategory ? categoryDisplay : 'Uncategorized'}
                            </span>
                        </div>
                    </div>

                    ${hasStock ? `
                        <div class="listing-preview-stock ${stockClass}">
                            <i data-lucide="package"></i>
                            <span>${stockText}</span>
                        </div>
                    ` : ''}
                </div>

                <!-- Footer -->
                <div class="listing-preview-footer">
                    <span class="listing-preview-price ${!hasPrice ? 'placeholder' : ''}">
                        ${hasPrice ? formattedPrice : '0.00 XMR'}
                    </span>
                    <button class="listing-preview-btn" disabled>
                        <i data-lucide="shopping-cart"></i>
                        <span>Add to Cart</span>
                    </button>
                </div>
            </div>
        `;

        // Reinitialize Lucide icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Format category for display
     * @param {string} category - Category value
     * @returns {string} Formatted category
     */
    formatCategory(category) {
        const categoryMap = {
            'digital': '💻 Digital Goods',
            'physical': '📦 Physical Goods',
            'services': '🛠️ Services',
            'vpn': '🔒 VPN & Privacy',
            'hosting': '🌐 Web Hosting',
            'software': '⚙️ Software',
            'tutorials': '📚 Tutorials',
            'accounts': '👤 Accounts',
            'other': '🔹 Other'
        };

        return categoryMap[category] || category;
    }

    /**
     * Escape HTML to prevent XSS
     * @param {string} text - Text to escape
     * @returns {string} Escaped HTML
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Destroy preview instance (cleanup)
     */
    destroy() {
        // Clear all debounce timers
        Object.values(this.debounceTimers).forEach(timer => clearTimeout(timer));
        this.debounceTimers = {};
        this.selectedImages = [];
    }
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ListingPreview };
}
