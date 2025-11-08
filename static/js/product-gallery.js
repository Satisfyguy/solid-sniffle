// static/js/product-gallery.js

/**
 * Product Gallery with Lightbox
 *
 * Manages a product image gallery with a main image, thumbnails,
 * and a full-screen lightbox for detailed viewing.
 */
class ProductGallery {
    constructor(containerId, imagesData) {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            console.error(`ProductGallery: Container with ID "${containerId}" not found.`);
            return;
        }

        this.images = imagesData; // Array of image URLs
        this.mainImageWrapper = this.container.querySelector('.product-gallery-main-image-wrapper');
        this.mainImage = this.container.querySelector('.product-gallery-main-image');
        this.thumbnailsContainer = this.container.querySelector('.product-gallery-thumbnails');
        this.zoomHint = this.container.querySelector('.product-gallery-zoom-hint');

        this.lightboxOverlay = null;
        this.lightboxImage = null;
        this.lightboxCounter = null;
        this.currentImageIndex = 0;

        this._init();
    }

    _init() {
        if (this.images.length === 0) {
            this.mainImageWrapper.innerHTML = `
                <div class="product-gallery-image-placeholder">
                    <i data-lucide="image"></i>
                    <span>No image available</span>
                </div>
            `;
            if (typeof lucide !== 'undefined') {
                lucide.createIcons();
            }
            this.mainImageWrapper.style.cursor = 'default';
            return;
        }

        this._renderThumbnails();
        this._updateMainImage(this.currentImageIndex);

        this.mainImageWrapper.addEventListener('click', this._openLightbox.bind(this));
        this.thumbnailsContainer.addEventListener('click', this._handleThumbnailClick.bind(this));

        this._createLightbox();
    }

    _renderThumbnails() {
        this.thumbnailsContainer.innerHTML = '';
        this.images.forEach((imageSrc, index) => {
            const thumbWrapper = document.createElement('div');
            thumbWrapper.classList.add('product-gallery-thumbnail-wrapper');
            if (index === this.currentImageIndex) {
                thumbWrapper.classList.add('active');
            }
            thumbWrapper.dataset.index = index;

            const thumb = document.createElement('img');
            thumb.classList.add('product-gallery-thumbnail');
            thumb.src = imageSrc;
            thumb.alt = `Thumbnail ${index + 1}`;

            thumbWrapper.appendChild(thumb);
            this.thumbnailsContainer.appendChild(thumbWrapper);
        });
    }

    _updateMainImage(index) {
        if (index < 0 || index >= this.images.length) return;

        this.currentImageIndex = index;
        this.mainImage.src = this.images[index];
        this.mainImage.alt = `Product Image ${index + 1}`;

        // Update active thumbnail
        this.container.querySelectorAll('.product-gallery-thumbnail-wrapper').forEach((thumb, i) => {
            if (i === index) {
                thumb.classList.add('active');
            } else {
                thumb.classList.remove('active');
            }
        });
    }

    _handleThumbnailClick(event) {
        const thumbnailWrapper = event.target.closest('.product-gallery-thumbnail-wrapper');
        if (thumbnailWrapper) {
            const index = parseInt(thumbnailWrapper.dataset.index, 10);
            this._updateMainImage(index);
        }
    }

    _createLightbox() {
        this.lightboxOverlay = document.createElement('div');
        this.lightboxOverlay.classList.add('lightbox-overlay');
        this.lightboxOverlay.innerHTML = `
            <div class="lightbox-content">
                <img src="" alt="Lightbox Image" class="lightbox-image">
                <div class="lightbox-close">
                    <i data-lucide="x"></i>
                </div>
                <div class="lightbox-nav-button prev">
                    <i data-lucide="chevron-left"></i>
                </div>
                <div class="lightbox-nav-button next">
                    <i data-lucide="chevron-right"></i>
                </div>
                <div class="lightbox-counter"></div>
            </div>
        `;
        document.body.appendChild(this.lightboxOverlay);

        this.lightboxImage = this.lightboxOverlay.querySelector('.lightbox-image');
        this.lightboxCounter = this.lightboxOverlay.querySelector('.lightbox-counter');

        this.lightboxOverlay.querySelector('.lightbox-close').addEventListener('click', this._closeLightbox.bind(this));
        this.lightboxOverlay.querySelector('.lightbox-nav-button.prev').addEventListener('click', this._showPrevImage.bind(this));
        this.lightboxOverlay.querySelector('.lightbox-nav-button.next').addEventListener('click', this._showNextImage.bind(this));
        this.lightboxOverlay.addEventListener('click', (e) => {
            if (e.target === this.lightboxOverlay) {
                this._closeLightbox();
            }
        });

        document.addEventListener('keydown', this._handleKeyDown.bind(this));

        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    _openLightbox() {
        if (this.images.length === 0) return;

        this.lightboxImage.src = this.images[this.currentImageIndex];
        this.lightboxImage.alt = `Product Image ${this.currentImageIndex + 1}`;
        this.lightboxCounter.textContent = `${this.currentImageIndex + 1} / ${this.images.length}`;
        this.lightboxOverlay.classList.add('active');
        document.body.style.overflow = 'hidden'; // Prevent scrolling background
    }

    _closeLightbox() {
        this.lightboxOverlay.classList.remove('active');
        document.body.style.overflow = ''; // Restore scrolling
    }

    _showPrevImage() {
        this.currentImageIndex = (this.currentImageIndex - 1 + this.images.length) % this.images.length;
        this._openLightbox();
    }

    _showNextImage() {
        this.currentImageIndex = (this.currentImageIndex + 1) % this.images.length;
        this._openLightbox();
    }

    _handleKeyDown(event) {
        if (!this.lightboxOverlay.classList.contains('active')) return;

        if (event.key === 'Escape') {
            this._closeLightbox();
        } else if (event.key === 'ArrowLeft') {
            this._showPrevImage();
        } else if (event.key === 'ArrowRight') {
            this._showNextImage();
        }
    }
}

