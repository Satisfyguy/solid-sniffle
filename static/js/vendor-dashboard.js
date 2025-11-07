/**
 * Vendor Dashboard Component
 *
 * Manages vendor dashboard interactions: delete listings, refresh stats.
 * Production-grade with CSRF protection, error handling, and confirmation dialogs.
 *
 * @module vendor-dashboard
 */

class VendorDashboard {
    /**
     * Create a vendor dashboard instance
     * @param {Object} options - Configuration options
     * @param {string} options.csrfToken - CSRF token for API requests
     */
    constructor(options = {}) {
        this.csrfToken = options.csrfToken;
        this.container = null;
        this.modal = null;
        this.pendingDeleteId = null;

        this.init();
    }

    /**
     * Initialize dashboard component
     */
    init() {
        this.container = document.querySelector('.vendor-dashboard-container');

        if (!this.container) {
            console.error('Vendor dashboard container not found');
            return;
        }

        this.createDeleteModal();
        this.attachEventListeners();

        console.log('✅ Vendor dashboard initialized');
    }

    /**
     * Create delete confirmation modal
     */
    createDeleteModal() {
        // Create modal HTML
        const modalHTML = `
            <div class="modal-overlay" id="delete-listing-modal" style="display: none;">
                <div class="modal-container">
                    <div class="modal-header">
                        <h3 class="modal-title">
                            <i data-lucide="alert-triangle" style="width: 20px; height: 20px; color: #ef4444;"></i>
                            Delete Listing
                        </h3>
                    </div>
                    <div class="modal-content">
                        <p style="color: rgba(255, 255, 255, 0.7); line-height: 1.6;">
                            Are you sure you want to delete this listing? This action cannot be undone.
                        </p>
                        <p style="margin-top: 1rem; padding: 0.75rem; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 4px; font-size: 0.875rem; color: rgba(255, 255, 255, 0.6);">
                            <strong style="color: #ef4444;">Warning:</strong> Any pending orders for this listing will not be affected, but buyers will no longer see this product.
                        </p>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="modal-btn modal-btn-cancel" id="delete-modal-cancel">
                            Cancel
                        </button>
                        <button type="button" class="modal-btn modal-btn-danger" id="delete-modal-confirm">
                            <i data-lucide="trash-2" style="width: 16px; height: 16px;"></i>
                            Delete Listing
                        </button>
                    </div>
                </div>
            </div>
        `;

        // Insert modal into DOM
        document.body.insertAdjacentHTML('beforeend', modalHTML);

        this.modal = document.getElementById('delete-listing-modal');

        // Attach modal event listeners
        document.getElementById('delete-modal-cancel').addEventListener('click', () => {
            this.hideDeleteModal();
        });

        document.getElementById('delete-modal-confirm').addEventListener('click', () => {
            this.confirmDelete();
        });

        // Close modal on overlay click
        this.modal.addEventListener('click', (e) => {
            if (e.target === this.modal) {
                this.hideDeleteModal();
            }
        });

        // Close modal on Escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.modal.style.display !== 'none') {
                this.hideDeleteModal();
            }
        });

        // Initialize Lucide icons in modal
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Attach event listeners
     */
    attachEventListeners() {
        // Delete buttons (event delegation for dynamically loaded content)
        this.container.addEventListener('click', (e) => {
            const deleteBtn = e.target.closest('.action-btn-delete');

            if (deleteBtn) {
                e.preventDefault();
                const listingId = deleteBtn.dataset.listingId;
                if (listingId) {
                    this.showDeleteModal(listingId);
                }
            }
        });

        // Refresh stats button (if exists)
        const refreshBtn = document.getElementById('refresh-dashboard-stats');
        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => this.refreshStats());
        }
    }

    /**
     * Show delete confirmation modal
     * @param {string} listingId - Listing ID to delete
     */
    showDeleteModal(listingId) {
        this.pendingDeleteId = listingId;
        this.modal.style.display = 'flex';
        document.body.style.overflow = 'hidden'; // Prevent background scrolling
    }

    /**
     * Hide delete confirmation modal
     */
    hideDeleteModal() {
        this.modal.style.display = 'none';
        this.pendingDeleteId = null;
        document.body.style.overflow = ''; // Restore scrolling
    }

    /**
     * Confirm delete action
     */
    async confirmDelete() {
        if (!this.pendingDeleteId) {
            return;
        }

        const listingId = this.pendingDeleteId;
        const confirmBtn = document.getElementById('delete-modal-confirm');

        // Set loading state
        confirmBtn.disabled = true;
        confirmBtn.innerHTML = `
            <span class="spinner" style="
                display: inline-block;
                width: 14px;
                height: 14px;
                border: 2px solid rgba(255,255,255,0.3);
                border-top-color: white;
                border-radius: 50%;
                animation: spin 0.8s linear infinite;
            "></span>
            Deleting...
        `;

        try {
            const response = await fetch(`/api/listings/${listingId}`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': this.csrfToken
                }
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || 'Failed to delete listing');
            }

            // Success - show notification
            this.showNotification('Listing deleted successfully', 'success');

            // Remove listing row from table
            const listingRow = document.querySelector(`tr[data-listing-id="${listingId}"]`);
            if (listingRow) {
                listingRow.style.opacity = '0';
                listingRow.style.transition = 'opacity 0.3s ease';
                setTimeout(() => listingRow.remove(), 300);
            }

            // Check if table is now empty
            setTimeout(() => {
                const tableBody = document.querySelector('.listings-table tbody');
                if (tableBody && tableBody.children.length === 0) {
                    this.showEmptyListingsState();
                }
            }, 400);

            // Hide modal
            this.hideDeleteModal();

            // Refresh stats
            this.refreshStats();

        } catch (error) {
            console.error('Failed to delete listing:', error);
            this.showNotification(error.message || 'Failed to delete listing. Please try again.', 'error');

            // Reset button
            confirmBtn.disabled = false;
            confirmBtn.innerHTML = `
                <i data-lucide="trash-2" style="width: 16px; height: 16px;"></i>
                Delete Listing
            `;

            // Reinitialize icons
            if (typeof lucide !== 'undefined') {
                lucide.createIcons();
            }
        }
    }

    /**
     * Show empty listings state
     */
    showEmptyListingsState() {
        const tableContainer = document.querySelector('.dashboard-section:has(.listings-table)');
        if (!tableContainer) {
            return;
        }

        tableContainer.innerHTML = `
            <div class="section-header">
                <h2 class="section-title">
                    <i data-lucide="package"></i>
                    Active Listings
                </h2>
            </div>
            <div class="empty-state">
                <i data-lucide="package-x"></i>
                <h3 class="empty-state-title">No Active Listings</h3>
                <p class="empty-state-description">
                    You haven't created any listings yet. Start selling by creating your first product listing.
                </p>
                <a href="/listings/create" hx-boost="true" class="btn btn-primary">
                    <i data-lucide="plus"></i>
                    Create Your First Listing
                </a>
            </div>
        `;

        // Reinitialize icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Refresh dashboard stats
     */
    async refreshStats() {
        console.log('Refreshing dashboard stats...');

        try {
            const response = await fetch('/api/vendor/dashboard/stats');

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }

            const data = await response.json();

            // Update stats cards
            this.updateStatsCards(data);

            console.log('✅ Stats refreshed');

        } catch (error) {
            console.error('Failed to refresh stats:', error);
            // Silent failure - don't disrupt user experience
        }
    }

    /**
     * Update stats cards with new data
     * @param {Object} data - Stats data from API
     */
    updateStatsCards(data) {
        // Active Listings
        const activeListingsValue = document.querySelector('.stat-card.active-listings .stat-card-value');
        if (activeListingsValue && data.active_listings !== undefined) {
            activeListingsValue.textContent = data.active_listings;
        }

        // Pending Orders
        const pendingOrdersValue = document.querySelector('.stat-card.pending-orders .stat-card-value');
        if (pendingOrdersValue && data.pending_orders !== undefined) {
            pendingOrdersValue.textContent = data.pending_orders;
        }

        // Total Revenue
        const revenueValue = document.querySelector('.stat-card.revenue .stat-card-value');
        if (revenueValue && data.total_revenue_xmr !== undefined) {
            revenueValue.textContent = `${data.total_revenue_xmr} XMR`;
        }

        // Revenue This Month
        const revenueChange = document.querySelector('.stat-card.revenue .stat-card-change span');
        if (revenueChange && data.revenue_this_month_xmr !== undefined) {
            revenueChange.textContent = `+${data.revenue_this_month_xmr} XMR this month`;
        }

        // Total Sales
        const salesValue = document.querySelector('.stat-card.sales .stat-card-value');
        if (salesValue && data.total_sales !== undefined) {
            salesValue.textContent = data.total_sales;
        }

        // Sales This Month
        const salesChange = document.querySelector('.stat-card.sales .stat-card-change span');
        if (salesChange && data.sales_this_month !== undefined) {
            salesChange.textContent = `+${data.sales_this_month} this month`;
        }
    }

    /**
     * Show notification toast
     * @param {string} message - Notification message
     * @param {string} type - Notification type ('success' | 'error' | 'info')
     */
    showNotification(message, type = 'info') {
        const toast = document.createElement('div');

        const colors = {
            success: '#22c55e',
            error: '#ef4444',
            info: '#0ea5e9'
        };

        const icons = {
            success: 'check-circle',
            error: 'alert-circle',
            info: 'info'
        };

        toast.style.cssText = `
            position: fixed;
            top: 2rem;
            right: 2rem;
            background: ${colors[type] || colors.info};
            color: white;
            padding: 1rem 1.5rem;
            border-radius: 4px;
            box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
            z-index: 10001;
            display: flex;
            align-items: center;
            gap: 0.75rem;
            animation: slideInRight 0.3s ease;
            max-width: 400px;
        `;

        toast.innerHTML = `
            <i data-lucide="${icons[type] || icons.info}" style="width: 20px; height: 20px; flex-shrink: 0;"></i>
            <span style="font-weight: 500;">${this.escapeHtml(message)}</span>
        `;

        document.body.appendChild(toast);

        // Initialize icon
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }

        // Auto-remove after 5 seconds
        setTimeout(() => {
            toast.style.opacity = '0';
            toast.style.transition = 'opacity 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, 5000);
    }

    /**
     * Escape HTML to prevent XSS
     * @param {string} text - Text to escape
     * @returns {string}
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Destroy dashboard instance (cleanup)
     */
    destroy() {
        // Remove modal
        if (this.modal) {
            this.modal.remove();
        }

        // Event listeners would be removed here if we stored references
    }
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { VendorDashboard };
}
