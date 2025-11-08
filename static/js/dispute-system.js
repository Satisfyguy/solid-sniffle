<<<<<<< HEAD
/**
 * Dispute System Component
 *
 * Modal-based dispute filing with evidence upload.
 * Production-grade with file validation and error handling.
 *
 * @module dispute-system
 */

class DisputeModal {
    /**
     * Create a dispute modal
     * @param {Object} options - Configuration options
     * @param {string} options.orderId - Order ID for dispute
     * @param {string} options.csrfToken - CSRF token for submission
     * @param {Function} [options.onSubmit] - Callback after successful submission
     */
    constructor(options = {}) {
        this.orderId = options.orderId;
        this.csrfToken = options.csrfToken;
        this.options = {
            onSubmit: options.onSubmit || null,
            maxFiles: 5,
            maxFileSize: 5 * 1024 * 1024, // 5MB
            allowedTypes: ['image/jpeg', 'image/png', 'image/gif', 'image/webp']
        };

        this.modal = null;
        this.files = [];
        this.isOpen = false;

        this.init();
    }

    /**
     * Initialize modal
     */
    init() {
        this.createModal();
        this.attachEventListeners();
        console.log('✅ Dispute Modal initialized');
    }

    /**
     * Create modal HTML structure
     */
    createModal() {
        const modal = document.createElement('div');
        modal.className = 'dispute-modal';
        modal.setAttribute('role', 'dialog');
        modal.setAttribute('aria-modal', 'true');
        modal.setAttribute('aria-labelledby', 'dispute-modal-title');

        modal.innerHTML = `
            <div class="dispute-modal-content">
                <div class="dispute-modal-header">
                    <h2 class="dispute-modal-title" id="dispute-modal-title">
                        <i data-lucide="alert-triangle"></i>
                        Open Dispute
                    </h2>
                    <button class="dispute-modal-close" aria-label="Close modal" type="button">
                        <i data-lucide="x"></i>
                    </button>
                </div>

                <div class="dispute-modal-body">
                    <form id="dispute-form" class="dispute-form">
                        <!-- Reason -->
                        <div class="dispute-form-group">
                            <label for="dispute-reason" class="dispute-form-label">
                                <i data-lucide="file-text"></i>
                                Reason <span class="dispute-form-label-required">*</span>
                            </label>
                            <select id="dispute-reason" name="reason" class="dispute-form-select" required>
                                <option value="">Select a reason...</option>
                                <option value="not_received">Product not received</option>
                                <option value="not_as_described">Product not as described</option>
                                <option value="defective">Defective or damaged item</option>
                                <option value="wrong_item">Wrong item sent</option>
                                <option value="fake">Counterfeit or fake product</option>
                                <option value="vendor_unresponsive">Vendor not responding</option>
                                <option value="other">Other issue</option>
                            </select>
                        </div>

                        <!-- Description -->
                        <div class="dispute-form-group">
                            <label for="dispute-description" class="dispute-form-label">
                                <i data-lucide="message-square"></i>
                                Detailed Description <span class="dispute-form-label-required">*</span>
                            </label>
                            <textarea
                                id="dispute-description"
                                name="description"
                                class="dispute-form-textarea"
                                placeholder="Describe the issue in detail. Include relevant dates, communications, and any attempts to resolve with the vendor..."
                                required
                                minlength="50"
                                maxlength="2000"
                                rows="6"
                            ></textarea>
                            <div class="dispute-form-counter">
                                <span id="char-count">0</span> / 2000 characters (minimum 50)
                            </div>
                        </div>

                        <!-- Evidence Upload -->
                        <div class="dispute-form-group">
                            <label class="dispute-form-label">
                                <i data-lucide="image"></i>
                                Evidence (optional)
                            </label>
                            <div class="dispute-file-upload" id="dispute-file-upload">
                                <i data-lucide="upload" class="dispute-file-upload-icon"></i>
                                <p class="dispute-file-upload-text">
                                    Drag & drop images here or <span class="dispute-file-upload-link">browse</span>
                                </p>
                                <p class="dispute-file-upload-hint">
                                    PNG, JPG, GIF up to 5MB (max 5 files)
                                </p>
                                <input
                                    type="file"
                                    id="dispute-file-input"
                                    class="dispute-file-input"
                                    accept="image/jpeg,image/png,image/gif,image/webp"
                                    multiple
                                >
                            </div>
                            <div id="dispute-file-previews" class="dispute-file-previews"></div>
                        </div>

                        <!-- Info Box -->
                        <div class="dispute-info-box">
                            <i data-lucide="info"></i>
                            <div class="dispute-info-box-text">
                                <strong>Important:</strong> Opening a dispute will notify the vendor and an arbiter will review your case. Provide as much evidence as possible to support your claim. False disputes may result in account suspension.
                            </div>
                        </div>
                    </form>
                </div>

                <div class="dispute-modal-footer">
                    <button type="button" class="dispute-btn dispute-btn-secondary" id="dispute-cancel-btn">
                        <i data-lucide="x"></i>
                        Cancel
                    </button>
                    <button type="submit" class="dispute-btn dispute-btn-primary" id="dispute-submit-btn" form="dispute-form">
                        <i data-lucide="alert-triangle"></i>
                        Submit Dispute
                    </button>
                </div>
            </div>
        `;

        document.body.appendChild(modal);
        this.modal = modal;

        // Initialize Lucide icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Attach event listeners
     */
    attachEventListeners() {
        // Close button
        const closeBtn = this.modal.querySelector('.dispute-modal-close');
        closeBtn.addEventListener('click', () => this.close());

        // Cancel button
        const cancelBtn = this.modal.querySelector('#dispute-cancel-btn');
        cancelBtn.addEventListener('click', () => this.close());

        // Close on background click
        this.modal.addEventListener('click', (e) => {
            if (e.target === this.modal) {
                this.close();
            }
        });

        // Keyboard navigation
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.isOpen) {
                this.close();
            }
        });

        // Form submission
        const form = this.modal.querySelector('#dispute-form');
        form.addEventListener('submit', (e) => {
            e.preventDefault();
            this.handleSubmit();
        });

        // Character counter
        const textarea = this.modal.querySelector('#dispute-description');
        textarea.addEventListener('input', () => this.updateCharCount());

        // File upload
        this.setupFileUpload();
    }

    /**
     * Setup file upload (drag & drop + click)
     */
    setupFileUpload() {
        const uploadZone = this.modal.querySelector('#dispute-file-upload');
        const fileInput = this.modal.querySelector('#dispute-file-input');

        // Click to browse
        uploadZone.addEventListener('click', () => fileInput.click());
        const browseLink = uploadZone.querySelector('.dispute-file-upload-link');
        if (browseLink) {
            browseLink.addEventListener('click', (e) => {
                e.stopPropagation();
                fileInput.click();
            });
        }

        // File input change
        fileInput.addEventListener('change', (e) => {
            this.handleFiles(Array.from(e.target.files));
        });

        // Drag & drop
        uploadZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadZone.classList.add('dragover');
        });

        uploadZone.addEventListener('dragleave', () => {
            uploadZone.classList.remove('dragover');
        });

        uploadZone.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadZone.classList.remove('dragover');

            const files = Array.from(e.dataTransfer.files);
            this.handleFiles(files);
        });
    }

    /**
     * Handle file selection
     * @param {File[]} newFiles - Selected files
     */
    handleFiles(newFiles) {
        const validFiles = [];

        for (const file of newFiles) {
            // Check file count
            if (this.files.length + validFiles.length >= this.options.maxFiles) {
                this.showError(`Maximum ${this.options.maxFiles} files allowed`);
                break;
            }

            // Check file type
            if (!this.options.allowedTypes.includes(file.type)) {
                this.showError(`${file.name}: Invalid file type. Only images allowed.`);
                continue;
            }

            // Check file size
            if (file.size > this.options.maxFileSize) {
                this.showError(`${file.name}: File too large (max 5MB)`);
                continue;
            }

            validFiles.push(file);
        }

        // Add valid files
        this.files.push(...validFiles);

        // Update preview
        this.renderFilePreviews();
    }

    /**
     * Render file previews
     */
    renderFilePreviews() {
        const container = this.modal.querySelector('#dispute-file-previews');
        container.innerHTML = '';

        this.files.forEach((file, index) => {
            const preview = document.createElement('div');
            preview.className = 'dispute-file-preview';

            // Create preview image
            const reader = new FileReader();
            reader.onload = (e) => {
                preview.innerHTML = `
                    <img src="${e.target.result}" alt="${file.name}" class="dispute-file-preview-image">
                    <button type="button" class="dispute-file-preview-remove" data-index="${index}" aria-label="Remove ${file.name}">
                        <i data-lucide="x"></i>
                    </button>
                `;

                // Initialize Lucide icons
                if (typeof lucide !== 'undefined') {
                    lucide.createIcons();
                }

                // Attach remove handler
                const removeBtn = preview.querySelector('.dispute-file-preview-remove');
                removeBtn.addEventListener('click', () => this.removeFile(index));
            };

            reader.readAsDataURL(file);
            container.appendChild(preview);
        });
    }

    /**
     * Remove file from list
     * @param {number} index - File index
     */
    removeFile(index) {
        this.files.splice(index, 1);
        this.renderFilePreviews();
    }

    /**
     * Update character counter
     */
    updateCharCount() {
        const textarea = this.modal.querySelector('#dispute-description');
        const counter = this.modal.querySelector('#char-count');
        const length = textarea.value.length;

        counter.textContent = length;

        // Update counter color
        const counterParent = counter.parentElement;
        counterParent.classList.remove('warning', 'error');

        if (length < 50) {
            counterParent.classList.add('error');
        } else if (length > 1900) {
            counterParent.classList.add('warning');
        }
    }

    /**
     * Handle form submission
     */
    async handleSubmit() {
        const form = this.modal.querySelector('#dispute-form');
        const submitBtn = this.modal.querySelector('#dispute-submit-btn');

        // Validate
        if (!form.checkValidity()) {
            form.reportValidity();
            return;
        }

        // Get form data
        const reason = form.elements['reason'].value;
        const description = form.elements['description'].value;

        // Check minimum length
        if (description.length < 50) {
            this.showError('Description must be at least 50 characters');
            return;
        }

        // Set loading state
        submitBtn.classList.add('loading');
        submitBtn.disabled = true;

        try {
            // Create FormData
            const formData = new FormData();
            formData.append('reason', reason);
            formData.append('description', description);

            // Add files
            this.files.forEach((file, index) => {
                formData.append(`evidence_${index}`, file);
            });

            // Submit
            const response = await fetch(`/api/orders/${this.orderId}/dispute`, {
                method: 'POST',
                headers: {
                    'X-CSRF-Token': this.csrfToken
                },
                body: formData
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || 'Failed to submit dispute');
            }

            const result = await response.json();

            // Success
            this.showSuccess('Dispute submitted successfully. An arbiter will review your case.');

            // Call callback
            if (this.options.onSubmit) {
                this.options.onSubmit(result);
            }

            // Close modal after delay
            setTimeout(() => {
                this.close();
                window.location.reload(); // Reload to show updated order status
            }, 2000);

        } catch (error) {
            console.error('Dispute submission error:', error);
            this.showError(error.message || 'Failed to submit dispute. Please try again.');

            // Clear loading state
            submitBtn.classList.remove('loading');
            submitBtn.disabled = false;
        }
    }

    /**
     * Show error message
     * @param {string} message - Error message
     */
    showError(message) {
        // Create toast notification (simple implementation)
        const toast = document.createElement('div');
        toast.style.cssText = `
            position: fixed;
            top: 2rem;
            right: 2rem;
            background: #ef4444;
            color: white;
            padding: 1rem 1.5rem;
            border-radius: 4px;
            box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
            z-index: 10001;
            animation: slideInRight 0.3s ease;
        `;
        toast.textContent = message;

        document.body.appendChild(toast);

        setTimeout(() => {
            toast.style.opacity = '0';
            toast.style.transition = 'opacity 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, 5000);
    }

    /**
     * Show success message
     * @param {string} message - Success message
     */
    showSuccess(message) {
        const toast = document.createElement('div');
        toast.style.cssText = `
            position: fixed;
            top: 2rem;
            right: 2rem;
            background: #22c55e;
            color: white;
            padding: 1rem 1.5rem;
            border-radius: 4px;
            box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
            z-index: 10001;
            animation: slideInRight 0.3s ease;
        `;
        toast.textContent = message;

        document.body.appendChild(toast);

        setTimeout(() => {
            toast.style.opacity = '0';
            toast.style.transition = 'opacity 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, 3000);
    }

    /**
     * Open modal
     */
    open() {
        this.modal.classList.add('active');
        this.isOpen = true;
        document.body.style.overflow = 'hidden';

        // Focus first input
        const firstInput = this.modal.querySelector('select, textarea');
        setTimeout(() => firstInput?.focus(), 100);
    }

    /**
     * Close modal
     */
    close() {
        this.modal.classList.remove('active');
        this.isOpen = false;
        document.body.style.overflow = '';

        // Reset form
        const form = this.modal.querySelector('#dispute-form');
        form.reset();
        this.files = [];
        this.renderFilePreviews();
        this.updateCharCount();
    }

    /**
     * Destroy modal (cleanup)
     */
    destroy() {
        if (this.modal && this.modal.parentNode) {
            this.modal.parentNode.removeChild(this.modal);
        }
    }
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { DisputeModal };
}
=======
// static/js/dispute-system.js

/**
 * Dispute System Modal
 *
 * Manages the display and functionality of a dispute submission modal.
 * Handles file uploads, previews, character counting, and form submission.
 */
class DisputeSystem {
    constructor(options = {}) {
        this.options = {
            modalId: 'dispute-modal',
            openButtonSelector: '.open-dispute-modal-btn',
            closeButtonSelector: '.dispute-modal-close',
            formId: 'dispute-form',
            reasonSelectId: 'dispute-reason',
            descriptionTextareaId: 'dispute-description',
            fileInputId: 'dispute-files',
            fileUploadAreaId: 'dispute-file-upload-area',
            filePreviewsId: 'dispute-file-previews',
            charCounterId: 'dispute-char-counter',
            submitButtonId: 'btn-dispute-submit',
            maxDescriptionLength: 500,
            maxFiles: 5,
            maxFileSize: 5 * 1024 * 1024, // 5MB
            allowedFileTypes: ['image/jpeg', 'image/png', 'image/gif'],
            apiEndpoint: '/api/orders/{orderId}/dispute',
            onDisputeSubmitted: () => {},
            ...options
        };

        this.modal = document.getElementById(this.options.modalId);
        if (!this.modal) {
            console.error(`DisputeSystem: Modal with ID "${this.options.modalId}" not found.`);
            return;
        }

        this.form = this.modal.querySelector(`#${this.options.formId}`);
        this.descriptionTextarea = this.modal.querySelector(`#${this.options.descriptionTextareaId}`);
        this.fileInput = this.modal.querySelector(`#${this.options.fileInputId}`);
        this.fileUploadArea = this.modal.querySelector(`#${this.options.fileUploadAreaId}`);
        this.filePreviewsContainer = this.modal.querySelector(`#${this.options.filePreviewsId}`);
        this.charCounter = this.modal.querySelector(`#${this.options.charCounterId}`);
        this.submitButton = this.modal.querySelector(`#${this.options.submitButtonId}`);

        this.selectedFiles = [];
        this.orderId = null; // To be set when modal is opened

        this._init();
    }

    _init() {
        // Event Listeners for opening and closing modal
        document.querySelectorAll(this.options.openButtonSelector).forEach(button => {
            button.addEventListener('click', (e) => {
                this.orderId = button.dataset.orderId; // Assuming orderId is stored in data-order-id
                this.openModal();
            });
        });
        this.modal.querySelector(this.options.closeButtonSelector).addEventListener('click', this.closeModal.bind(this));
        this.modal.addEventListener('click', (e) => {
            if (e.target === this.modal) {
                this.closeModal();
            }
        });
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.modal.classList.contains('active')) {
                this.closeModal();
            }
        });

        // Form element listeners
        if (this.descriptionTextarea) {
            this.descriptionTextarea.addEventListener('input', this._updateCharCounter.bind(this));
            this._updateCharCounter(); // Initial count
        }
        if (this.fileInput) {
            this.fileInput.addEventListener('change', this._handleFileSelect.bind(this));
        }
        if (this.fileUploadArea) {
            this.fileUploadArea.addEventListener('dragover', this._handleDragOver.bind(this));
            this.fileUploadArea.addEventListener('dragleave', this._handleDragLeave.bind(this));
            this.fileUploadArea.addEventListener('drop', this._handleDrop.bind(this));
            this.fileUploadArea.addEventListener('click', () => this.fileInput.click());
        }
        if (this.form) {
            this.form.addEventListener('submit', this._handleSubmit.bind(this));
        }
    }

    _updateCharCounter() {
        if (!this.descriptionTextarea || !this.charCounter) return;

        const currentLength = this.descriptionTextarea.value.length;
        this.charCounter.textContent = `${currentLength}/${this.options.maxDescriptionLength}`;

        this.charCounter.classList.remove('warning', 'error');
        if (currentLength > this.options.maxDescriptionLength * 0.8) {
            this.charCounter.classList.add('warning');
        }
        if (currentLength > this.options.maxDescriptionLength) {
            this.charCounter.classList.add('error');
        }
    }

    _handleDragOver(e) {
        e.preventDefault();
        this.fileUploadArea.classList.add('drag-over');
    }

    _handleDragLeave(e) {
        e.preventDefault();
        this.fileUploadArea.classList.remove('drag-over');
    }

    _handleDrop(e) {
        e.preventDefault();
        this.fileUploadArea.classList.remove('drag-over');
        const files = e.dataTransfer.files;
        this._addFiles(files);
    }

    _handleFileSelect(e) {
        const files = e.target.files;
        this._addFiles(files);
    }

    _addFiles(files) {
        let filesAdded = 0;
        for (let i = 0; i < files.length; i++) {
            const file = files[i];

            if (this.selectedFiles.length >= this.options.maxFiles) {
                alert(`You can only upload a maximum of ${this.options.maxFiles} files.`);
                break;
            }
            if (!this.options.allowedFileTypes.includes(file.type)) {
                alert(`File "${file.name}" is not an allowed type. Only JPEG, PNG, GIF are allowed.`);
                continue;
            }
            if (file.size > this.options.maxFileSize) {
                alert(`File "${file.name}" is too large. Maximum size is ${this.options.maxFileSize / (1024 * 1024)}MB.`);
                continue;
            }

            this.selectedFiles.push(file);
            filesAdded++;
        }
        if (filesAdded > 0) {
            this._renderFilePreviews();
        }
        this.fileInput.value = ''; // Clear input to allow re-uploading same file if needed
    }

    _renderFilePreviews() {
        if (!this.filePreviewsContainer) return;

        this.filePreviewsContainer.innerHTML = '';
        this.selectedFiles.forEach((file, index) => {
            const reader = new FileReader();
            reader.onload = (e) => {
                const previewItem = document.createElement('div');
                previewItem.classList.add('dispute-file-preview-item');
                previewItem.innerHTML = `
                    <img src="${e.target.result}" alt="${file.name}">
                    <button type="button" class="dispute-file-preview-remove" data-index="${index}">
                        <i data-lucide="x"></i>
                    </button>
                `;
                this.filePreviewsContainer.appendChild(previewItem);

                previewItem.querySelector('.dispute-file-preview-remove').addEventListener('click', (e) => {
                    const idxToRemove = parseInt(e.currentTarget.dataset.index, 10);
                    this._removeFile(idxToRemove);
                });
                if (typeof lucide !== 'undefined') {
                    lucide.createIcons();
                }
            };
            reader.readAsDataURL(file);
        });
    }

    _removeFile(index) {
        this.selectedFiles.splice(index, 1);
        this._renderFilePreviews();
    }

    async _handleSubmit(e) {
        e.preventDefault();
        if (!this.form || !this.orderId) return;

        this.submitButton.classList.add('loading');
        this.submitButton.disabled = true;
        const loaderIcon = this.submitButton.querySelector('.lucide-loader');
        if (loaderIcon) loaderIcon.style.display = 'inline-block';

        const formData = new FormData();
        formData.append('csrf_token', document.getElementById('csrf-token')?.value || '');
        formData.append('reason', this.form.querySelector(`#${this.options.reasonSelectId}`).value);
        formData.append('description', this.descriptionTextarea.value);
        this.selectedFiles.forEach((file, index) => {
            formData.append(`files[${index}]`, file);
        });

        try {
            const response = await fetch(this.options.apiEndpoint.replace('{orderId}', this.orderId), {
                method: 'POST',
                body: formData,
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to submit dispute');
            }

            const result = await response.json();
            alert('Dispute submitted successfully!');
            this.options.onDisputeSubmitted(result);
            this.closeModal();

        } catch (error) {
            console.error('Dispute submission error:', error);
            alert(`Error submitting dispute: ${error.message}`);
        } finally {
            this.submitButton.classList.remove('loading');
            this.submitButton.disabled = false;
            if (loaderIcon) loaderIcon.style.display = 'none';
        }
    }

    openModal() {
        this.modal.classList.add('active');
        document.body.style.overflow = 'hidden'; // Prevent scrolling background
        // Reset form fields
        if (this.form) this.form.reset();
        this.selectedFiles = [];
        this._renderFilePreviews();
        this._updateCharCounter();
    }

    closeModal() {
        this.modal.classList.remove('active');
        document.body.style.overflow = ''; // Restore scrolling
    }
}
>>>>>>> cd3680e (feat: Add new UI/UX features and ignore testnet data)
