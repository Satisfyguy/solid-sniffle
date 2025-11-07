/**
 * Product Gallery Component
 *
 * Image gallery with thumbnails and lightbox functionality.
 * Production-grade with keyboard navigation and accessibility.
 *
 * @module product-gallery
 */

class ProductGallery {
    /**
     * Create a product gallery
     * @param {string} containerSelector - CSS selector for gallery container
     * @param {Array<string>} images - Array of image URLs
     * @param {Object} options - Configuration options
     */
    constructor(containerSelector, images, options = {}) {
        this.container = document.querySelector(containerSelector);

        if (!this.container) {
            console.error(`Product Gallery: Container not found (${containerSelector})`);
            return;
        }

        this.images = images || [];
        this.currentIndex = 0;
        this.options = {
            showThumbnails: options.showThumbnails !== false,
            showLightbox: options.showLightbox !== false,
            showZoomHint: options.showZoomHint !== false,
            categoryBadge: options.categoryBadge || null,
        };

        this.lightbox = null;
        this.mainImage = null;
        this.thumbnails = [];

        if (this.images.length > 0) {
            this.init();
        } else {
            this.showEmptyState();
        }
    }

    /**
     * Initialize gallery
     */
    init() {
        this.render();
        this.attachEventListeners();
        this.createLightbox();

        console.log('✅ Product Gallery initialized:', {
            images: this.images.length,
            thumbnails: this.options.showThumbnails,
            lightbox: this.options.showLightbox
        });
    }

    /**
     * Render gallery HTML
     */
    render() {
        let html = '<div class="product-gallery">';

        // Main Image
        html += `
            <div class="gallery-main" data-gallery-main>
                <img
                    src="${this.images[0]}"
                    alt="Product image 1"
                    class="gallery-main-image"
                    data-gallery-main-image
                >
                ${this.options.categoryBadge ? `
                    <div class="gallery-badge">${this.options.categoryBadge}</div>
                ` : ''}
                ${this.options.showZoomHint ? `
                    <div class="gallery-zoom-hint">
                        <i data-lucide="zoom-in"></i>
                        <span>Click to enlarge</span>
                    </div>
                ` : ''}
            </div>
        `;

        // Thumbnails
        if (this.options.showThumbnails && this.images.length > 1) {
            html += '<div class="gallery-thumbnails" data-gallery-thumbnails>';

            this.images.forEach((image, index) => {
                html += `
                    <div
                        class="gallery-thumbnail ${index === 0 ? 'active' : ''}"
                        data-gallery-thumbnail
                        data-index="${index}"
                        tabindex="0"
                        role="button"
                        aria-label="View image ${index + 1}"
                    >
                        <img
                            src="${image}"
                            alt="Thumbnail ${index + 1}"
                            class="gallery-thumbnail-image"
                        >
                    </div>
                `;
            });

            html += '</div>';
        }

        html += '</div>';

        this.container.innerHTML = html;

        // Store references
        this.mainImage = this.container.querySelector('[data-gallery-main-image]');
        this.thumbnails = Array.from(this.container.querySelectorAll('[data-gallery-thumbnail]'));

        // Initialize Lucide icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Show empty state when no images
     */
    showEmptyState() {
        this.container.innerHTML = `
            <div class="gallery-empty">
                <i data-lucide="image"></i>
                <span class="gallery-empty-text">No images available</span>
            </div>
        `;

        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Attach event listeners
     */
    attachEventListeners() {
        // Main image click - open lightbox
        const mainContainer = this.container.querySelector('[data-gallery-main]');
        if (mainContainer && this.options.showLightbox) {
            mainContainer.addEventListener('click', () => this.openLightbox());
        }

        // Thumbnail clicks
        this.thumbnails.forEach((thumbnail, index) => {
            thumbnail.addEventListener('click', () => this.goToImage(index));

            // Keyboard navigation for thumbnails
            thumbnail.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    this.goToImage(index);
                }
            });
        });
    }

    /**
     * Go to specific image
     * @param {number} index - Image index
     */
    goToImage(index) {
        if (index < 0 || index >= this.images.length || index === this.currentIndex) {
            return;
        }

        // Update current index
        this.currentIndex = index;

        // Update main image with fade effect
        if (this.mainImage) {
            this.mainImage.classList.add('loading');

            setTimeout(() => {
                this.mainImage.src = this.images[index];
                this.mainImage.alt = `Product image ${index + 1}`;
                this.mainImage.classList.remove('loading');
            }, 150);
        }

        // Update active thumbnail
        this.thumbnails.forEach((thumb, i) => {
            if (i === index) {
                thumb.classList.add('active');
                thumb.setAttribute('aria-current', 'true');
            } else {
                thumb.classList.remove('active');
                thumb.removeAttribute('aria-current');
            }
        });
    }

    /**
     * Navigate to next image
     */
    next() {
        const nextIndex = (this.currentIndex + 1) % this.images.length;
        this.goToImage(nextIndex);
    }

    /**
     * Navigate to previous image
     */
    previous() {
        const prevIndex = (this.currentIndex - 1 + this.images.length) % this.images.length;
        this.goToImage(prevIndex);
    }

    /**
     * Create lightbox overlay
     */
    createLightbox() {
        if (!this.options.showLightbox) return;

        // Create lightbox HTML
        const lightbox = document.createElement('div');
        lightbox.className = 'gallery-lightbox';
        lightbox.setAttribute('role', 'dialog');
        lightbox.setAttribute('aria-modal', 'true');
        lightbox.setAttribute('aria-label', 'Image lightbox');

        lightbox.innerHTML = `
            <div class="gallery-lightbox-content">
                <img class="gallery-lightbox-image" src="${this.images[0]}" alt="Full size image">

                <button class="gallery-lightbox-close" aria-label="Close lightbox" tabindex="0">
                    <i data-lucide="x"></i>
                </button>

                <button class="gallery-lightbox-prev" aria-label="Previous image" tabindex="0">
                    <i data-lucide="chevron-left"></i>
                </button>

                <button class="gallery-lightbox-next" aria-label="Next image" tabindex="0">
                    <i data-lucide="chevron-right"></i>
                </button>

                <div class="gallery-lightbox-counter">
                    <span class="lightbox-current">1</span> / <span class="lightbox-total">${this.images.length}</span>
                </div>
            </div>
        `;

        document.body.appendChild(lightbox);
        this.lightbox = lightbox;

        // Initialize Lucide icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }

        // Event listeners
        const closeBtn = lightbox.querySelector('.gallery-lightbox-close');
        const prevBtn = lightbox.querySelector('.gallery-lightbox-prev');
        const nextBtn = lightbox.querySelector('.gallery-lightbox-next');

        closeBtn.addEventListener('click', () => this.closeLightbox());
        prevBtn.addEventListener('click', () => this.lightboxPrevious());
        nextBtn.addEventListener('click', () => this.lightboxNext());

        // Close on background click
        lightbox.addEventListener('click', (e) => {
            if (e.target === lightbox) {
                this.closeLightbox();
            }
        });

        // Keyboard navigation
        document.addEventListener('keydown', (e) => {
            if (!lightbox.classList.contains('active')) return;

            switch (e.key) {
                case 'Escape':
                    this.closeLightbox();
                    break;
                case 'ArrowLeft':
                    this.lightboxPrevious();
                    break;
                case 'ArrowRight':
                    this.lightboxNext();
                    break;
            }
        });
    }

    /**
     * Open lightbox
     */
    openLightbox() {
        if (!this.lightbox) return;

        const image = this.lightbox.querySelector('.gallery-lightbox-image');
        image.src = this.images[this.currentIndex];

        this.updateLightboxCounter();

        this.lightbox.classList.add('active');
        document.body.style.overflow = 'hidden'; // Prevent scrolling

        // Focus close button for accessibility
        const closeBtn = this.lightbox.querySelector('.gallery-lightbox-close');
        setTimeout(() => closeBtn.focus(), 100);
    }

    /**
     * Close lightbox
     */
    closeLightbox() {
        if (!this.lightbox) return;

        this.lightbox.classList.remove('active');
        document.body.style.overflow = ''; // Restore scrolling
    }

    /**
     * Navigate to previous image in lightbox
     */
    lightboxPrevious() {
        this.previous();
        this.updateLightboxImage();
    }

    /**
     * Navigate to next image in lightbox
     */
    lightboxNext() {
        this.next();
        this.updateLightboxImage();
    }

    /**
     * Update lightbox image
     */
    updateLightboxImage() {
        if (!this.lightbox) return;

        const image = this.lightbox.querySelector('.gallery-lightbox-image');
        image.src = this.images[this.currentIndex];

        this.updateLightboxCounter();
    }

    /**
     * Update lightbox counter
     */
    updateLightboxCounter() {
        if (!this.lightbox) return;

        const current = this.lightbox.querySelector('.lightbox-current');
        const total = this.lightbox.querySelector('.lightbox-total');

        if (current) current.textContent = this.currentIndex + 1;
        if (total) total.textContent = this.images.length;
    }

    /**
     * Destroy gallery (cleanup)
     */
    destroy() {
        if (this.lightbox && this.lightbox.parentNode) {
            this.lightbox.parentNode.removeChild(this.lightbox);
        }
        this.container.innerHTML = '';
    }
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ProductGallery };
}
