/**
 * Image Upload Handler for Listings
 * 
 * Handles image uploads to IPFS via the server API.
 * Provides drag-and-drop functionality and progress indicators.
 */

class ImageUploader {
    constructor(listingId, containerId) {
        this.listingId = listingId;
        this.container = document.getElementById(containerId);
        this.uploadEndpoint = `/api/listings/${listingId}/images`;
        this.imageEndpoint = `/api/listings/${listingId}/images`;
        this.maxFiles = 10;
        this.maxFileSize = 5 * 1024 * 1024; // 5MB
        this.allowedTypes = ['image/jpeg', 'image/png', 'image/gif'];
        
        this.init();
    }

    init() {
        this.createUploadArea();
        this.bindEvents();
    }

    createUploadArea() {
        const uploadArea = document.createElement('div');
        uploadArea.className = 'upload-area';
        uploadArea.innerHTML = `
            <div class="upload-zone" id="upload-zone">
                <div class="upload-icon">📁</div>
                <div class="upload-text">
                    <strong>Drop images here</strong><br>
                    or click to select files
                </div>
                <div class="upload-info">
                    Max ${this.maxFiles} images, ${this.maxFileSize / 1024 / 1024}MB each
                </div>
                <input type="file" id="file-input" multiple accept="image/*" style="display: none;">
            </div>
            <div class="upload-progress" id="upload-progress" style="display: none;">
                <div class="progress-bar">
                    <div class="progress-fill" id="progress-fill"></div>
                </div>
                <div class="progress-text" id="progress-text">Uploading...</div>
            </div>
            <div class="uploaded-images" id="uploaded-images"></div>
        `;

        // Add CSS styles
        const style = document.createElement('style');
        style.textContent = `
            .upload-area {
                margin: 20px 0;
                border: 2px dashed #ccc;
                border-radius: 8px;
                padding: 20px;
                text-align: center;
                background: #f9f9f9;
            }
            
            .upload-zone {
                padding: 40px 20px;
                cursor: pointer;
                transition: all 0.3s ease;
            }
            
            .upload-zone:hover {
                background: #f0f0f0;
                border-color: #999;
            }
            
            .upload-zone.dragover {
                background: #e8f4fd;
                border-color: #007bff;
            }
            
            .upload-icon {
                font-size: 48px;
                margin-bottom: 16px;
            }
            
            .upload-text {
                font-size: 16px;
                margin-bottom: 8px;
            }
            
            .upload-info {
                font-size: 12px;
                color: #666;
            }
            
            .upload-progress {
                margin: 20px 0;
            }
            
            .progress-bar {
                width: 100%;
                height: 20px;
                background: #e0e0e0;
                border-radius: 10px;
                overflow: hidden;
                margin-bottom: 10px;
            }
            
            .progress-fill {
                height: 100%;
                background: #007bff;
                width: 0%;
                transition: width 0.3s ease;
            }
            
            .progress-text {
                text-align: center;
                font-size: 14px;
                color: #666;
            }
            
            .uploaded-images {
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
                gap: 10px;
                margin-top: 20px;
            }
            
            .image-preview {
                position: relative;
                border: 1px solid #ddd;
                border-radius: 8px;
                overflow: hidden;
                background: #fff;
            }
            
            .image-preview img {
                width: 100%;
                height: 150px;
                object-fit: cover;
            }
            
            .image-remove {
                position: absolute;
                top: 5px;
                right: 5px;
                background: rgba(255, 0, 0, 0.8);
                color: white;
                border: none;
                border-radius: 50%;
                width: 24px;
                height: 24px;
                cursor: pointer;
                font-size: 14px;
            }
            
            .image-remove:hover {
                background: rgba(255, 0, 0, 1);
            }
        `;
        
        document.head.appendChild(style);
        this.container.appendChild(uploadArea);
    }

    bindEvents() {
        const uploadZone = document.getElementById('upload-zone');
        const fileInput = document.getElementById('file-input');

        // Click to select files
        uploadZone.addEventListener('click', () => {
            fileInput.click();
        });

        // File input change
        fileInput.addEventListener('change', (e) => {
            this.handleFiles(e.target.files);
        });

        // Drag and drop
        uploadZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadZone.classList.add('dragover');
        });

        uploadZone.addEventListener('dragleave', (e) => {
            e.preventDefault();
            uploadZone.classList.remove('dragover');
        });

        uploadZone.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadZone.classList.remove('dragover');
            this.handleFiles(e.dataTransfer.files);
        });
    }

    handleFiles(files) {
        const fileArray = Array.from(files);
        
        // Validate files
        const validFiles = fileArray.filter(file => {
            if (!this.allowedTypes.includes(file.type)) {
                this.showError(`File ${file.name} is not a supported image format`);
                return false;
            }
            if (file.size > this.maxFileSize) {
                this.showError(`File ${file.name} is too large (max ${this.maxFileSize / 1024 / 1024}MB)`);
                return false;
            }
            return true;
        });

        if (validFiles.length === 0) {
            return;
        }

        if (validFiles.length > this.maxFiles) {
            this.showError(`Maximum ${this.maxFiles} files allowed`);
            return;
        }

        this.uploadFiles(validFiles);
    }

    async uploadFiles(files) {
        const formData = new FormData();
        
        // Add all files to form data
        files.forEach(file => {
            formData.append('images', file);
        });

        // Show progress
        this.showProgress();

        try {
            const response = await fetch(this.uploadEndpoint, {
                method: 'POST',
                body: formData,
                credentials: 'same-origin'
            });

            const result = await response.json();

            if (response.ok) {
                this.hideProgress();
                this.showSuccess(`Successfully uploaded ${result.image_count} images`);
                this.loadExistingImages();
            } else {
                this.hideProgress();
                this.showError(result.error || 'Upload failed');
            }
        } catch (error) {
            this.hideProgress();
            this.showError('Upload failed: ' + error.message);
        }
    }

    showProgress() {
        const progress = document.getElementById('upload-progress');
        const progressFill = document.getElementById('progress-fill');
        const progressText = document.getElementById('progress-text');
        
        progress.style.display = 'block';
        progressFill.style.width = '0%';
        progressText.textContent = 'Uploading...';

        // Simulate progress (in real implementation, use actual progress events)
        let width = 0;
        const interval = setInterval(() => {
            width += 10;
            progressFill.style.width = width + '%';
            if (width >= 100) {
                clearInterval(interval);
            }
        }, 100);
    }

    hideProgress() {
        const progress = document.getElementById('upload-progress');
        progress.style.display = 'none';
    }

    showSuccess(message) {
        this.showMessage(message, 'success');
    }

    showError(message) {
        this.showMessage(message, 'error');
    }

    showMessage(message, type) {
        // Remove existing messages
        const existing = document.querySelector('.upload-message');
        if (existing) {
            existing.remove();
        }

        const messageDiv = document.createElement('div');
        messageDiv.className = `upload-message ${type}`;
        messageDiv.textContent = message;
        messageDiv.style.cssText = `
            padding: 10px;
            margin: 10px 0;
            border-radius: 4px;
            color: white;
            background: ${type === 'success' ? '#28a745' : '#dc3545'};
        `;

        this.container.appendChild(messageDiv);

        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (messageDiv.parentNode) {
                messageDiv.remove();
            }
        }, 5000);
    }

    async loadExistingImages() {
        // Fetch the listing data to get existing images
        try {
            const response = await fetch(`/api/listings/${this.listingId}`);
            if (response.ok) {
                const listing = await response.json();
                this.displayImages(listing.images || []);
            }
        } catch (error) {
            console.error('Failed to load existing images:', error);
        }
    }

    displayImages(imageCids) {
        const uploadedImages = document.getElementById('uploaded-images');
        uploadedImages.innerHTML = '';

        imageCids.forEach(cid => {
            const imageDiv = document.createElement('div');
            imageDiv.className = 'image-preview';
            const img = document.createElement('img');
            img.src = `/api/listings/${this.listingId}/images/${cid}`;
            img.alt = 'Product image';

            const removeButton = document.createElement('button');
            removeButton.className = 'image-remove';
            removeButton.textContent = '×';
            removeButton.addEventListener('click', () => this.removeImage(cid));

            imageDiv.appendChild(img);
            imageDiv.appendChild(removeButton);
            uploadedImages.appendChild(imageDiv);
        });
    }

    async removeImage(cid) {
        if (!confirm('Remove this image?')) {
            return;
        }

        try {
            const response = await fetch(`/api/listings/${this.listingId}/images/${cid}`, {
                method: 'DELETE',
                credentials: 'same-origin'
            });

            if (response.ok) {
                this.showSuccess('Image removed successfully');
                this.loadExistingImages();
            } else {
                const result = await response.json();
                this.showError(result.error || 'Failed to remove image');
            }
        } catch (error) {
            this.showError('Failed to remove image: ' + error.message);
        }
    }
}

// Global function for removing images
window.removeImage = function(cid) {
    const uploadContainer = document.getElementById('image-upload-container');
    if (uploadContainer && uploadContainer.imageUploader) {
        uploadContainer.imageUploader.removeImage(cid);
    }
};

// Initialize image uploader when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    // Check if we're on a listing page with an upload container
    const uploadContainer = document.getElementById('image-upload-container');
    if (uploadContainer) {
        const listingId = uploadContainer.dataset.listingId;
        if (listingId) {
            const uploader = new ImageUploader(listingId, 'image-upload-container');
            uploadContainer.imageUploader = uploader;
            // Load existing images on page load
            uploader.loadExistingImages();
        }
    }
});
