/**
 * Enhanced Timeline Component
 *
 * Expandable timeline with transaction details.
 * Production-grade with copy-to-clipboard and smooth animations.
 *
 * @module enhanced-timeline
 */

class EnhancedTimeline {
    /**
     * Create an enhanced timeline
     * @param {string} containerSelector - CSS selector for timeline container
     */
    constructor(containerSelector) {
        this.container = document.querySelector(containerSelector);

        if (!this.container) {
            console.error(`Enhanced Timeline: Container not found (${containerSelector})`);
            return;
        }

        this.items = [];
        this.init();
    }

    /**
     * Initialize timeline
     */
    init() {
        this.items = Array.from(this.container.querySelectorAll('.timeline-item'));

        // Attach event listeners to all items
        this.items.forEach(item => {
            this.setupItem(item);
        });

        console.log('✅ Enhanced Timeline initialized:', { items: this.items.length });
    }

    /**
     * Setup a timeline item
     * @param {HTMLElement} item - Timeline item element
     */
    setupItem(item) {
        const header = item.querySelector('.timeline-header');
        const expandBtn = item.querySelector('.timeline-expand-btn');
        const details = item.querySelector('.timeline-details');

        if (!header || !expandBtn || !details) {
            return; // Skip items without expandable details
        }

        // Toggle on header click
        header.addEventListener('click', () => {
            this.toggleItem(item);
        });

        // Setup copy buttons within this item
        const copyButtons = item.querySelectorAll('.timeline-copy-btn');
        copyButtons.forEach(btn => {
            btn.addEventListener('click', (e) => {
                e.stopPropagation(); // Don't trigger expand
                this.copyToClipboard(btn);
            });
        });
    }

    /**
     * Toggle timeline item expansion
     * @param {HTMLElement} item - Timeline item to toggle
     */
    toggleItem(item) {
        const expandBtn = item.querySelector('.timeline-expand-btn');
        const details = item.querySelector('.timeline-details');

        if (!expandBtn || !details) return;

        const isExpanded = details.classList.contains('expanded');

        if (isExpanded) {
            // Collapse
            details.classList.remove('expanded');
            expandBtn.classList.remove('expanded');
            expandBtn.setAttribute('aria-expanded', 'false');
        } else {
            // Expand
            details.classList.add('expanded');
            expandBtn.classList.add('expanded');
            expandBtn.setAttribute('aria-expanded', 'true');
        }
    }

    /**
     * Expand specific timeline item
     * @param {HTMLElement} item - Timeline item to expand
     */
    expandItem(item) {
        const expandBtn = item.querySelector('.timeline-expand-btn');
        const details = item.querySelector('.timeline-details');

        if (!details) return;

        if (!details.classList.contains('expanded')) {
            details.classList.add('expanded');
            if (expandBtn) {
                expandBtn.classList.add('expanded');
                expandBtn.setAttribute('aria-expanded', 'true');
            }
        }
    }

    /**
     * Collapse specific timeline item
     * @param {HTMLElement} item - Timeline item to collapse
     */
    collapseItem(item) {
        const expandBtn = item.querySelector('.timeline-expand-btn');
        const details = item.querySelector('.timeline-details');

        if (!details) return;

        if (details.classList.contains('expanded')) {
            details.classList.remove('expanded');
            if (expandBtn) {
                expandBtn.classList.remove('expanded');
                expandBtn.setAttribute('aria-expanded', 'false');
            }
        }
    }

    /**
     * Expand all timeline items
     */
    expandAll() {
        this.items.forEach(item => this.expandItem(item));
    }

    /**
     * Collapse all timeline items
     */
    collapseAll() {
        this.items.forEach(item => this.collapseItem(item));
    }

    /**
     * Copy text to clipboard
     * @param {HTMLElement} button - Copy button element
     */
    async copyToClipboard(button) {
        const textToCopy = button.dataset.copy;

        if (!textToCopy) {
            console.error('No data-copy attribute found on button');
            return;
        }

        try {
            // Modern clipboard API
            if (navigator.clipboard && navigator.clipboard.writeText) {
                await navigator.clipboard.writeText(textToCopy);
            } else {
                // Fallback for older browsers
                const textarea = document.createElement('textarea');
                textarea.value = textToCopy;
                textarea.style.position = 'fixed';
                textarea.style.opacity = '0';
                document.body.appendChild(textarea);
                textarea.select();
                document.execCommand('copy');
                document.body.removeChild(textarea);
            }

            // Show success state
            this.showCopySuccess(button);

        } catch (error) {
            console.error('Failed to copy:', error);
            this.showCopyError(button);
        }
    }

    /**
     * Show copy success state
     * @param {HTMLElement} button - Copy button
     */
    showCopySuccess(button) {
        const originalText = button.innerHTML;
        button.classList.add('copied');
        button.innerHTML = '<i data-lucide="check"></i> Copied';

        // Reinitialize Lucide for new icon
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }

        // Reset after 2 seconds
        setTimeout(() => {
            button.classList.remove('copied');
            button.innerHTML = originalText;

            // Reinitialize Lucide again
            if (typeof lucide !== 'undefined') {
                lucide.createIcons();
            }
        }, 2000);
    }

    /**
     * Show copy error state
     * @param {HTMLElement} button - Copy button
     */
    showCopyError(button) {
        const originalText = button.innerHTML;
        button.style.borderColor = '#ef4444';
        button.style.color = '#ef4444';
        button.innerHTML = '<i data-lucide="x"></i> Failed';

        // Reinitialize Lucide
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }

        // Reset after 2 seconds
        setTimeout(() => {
            button.style.borderColor = '';
            button.style.color = '';
            button.innerHTML = originalText;

            // Reinitialize Lucide
            if (typeof lucide !== 'undefined') {
                lucide.createIcons();
            }
        }, 2000);
    }

    /**
     * Add a new timeline item (dynamic)
     * @param {Object} data - Timeline item data
     * @returns {HTMLElement} Created timeline item
     */
    addItem(data) {
        const item = document.createElement('div');
        item.className = `timeline-item ${data.status || 'pending'}`;

        item.innerHTML = `
            <div class="timeline-marker"></div>
            <div class="timeline-content">
                <div class="timeline-header">
                    <div class="timeline-header-left">
                        <div class="timeline-icon">
                            <i data-lucide="${data.icon || 'circle'}"></i>
                        </div>
                        <div>
                            <h3 class="timeline-title">${data.title}</h3>
                            ${data.time ? `<p class="timeline-time">${data.time}</p>` : ''}
                        </div>
                    </div>
                    ${data.details ? `
                        <button class="timeline-expand-btn" aria-expanded="false" aria-label="Toggle details">
                            <i data-lucide="chevron-down"></i>
                        </button>
                    ` : ''}
                </div>
                ${data.description ? `
                    <p class="timeline-description">${data.description}</p>
                ` : ''}
                ${data.details ? `
                    <div class="timeline-details">
                        ${data.details}
                    </div>
                ` : ''}
            </div>
        `;

        this.container.appendChild(item);

        // Initialize Lucide icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }

        // Setup event listeners
        this.setupItem(item);

        // Add to items array
        this.items.push(item);

        return item;
    }

    /**
     * Remove a timeline item
     * @param {HTMLElement} item - Item to remove
     */
    removeItem(item) {
        const index = this.items.indexOf(item);
        if (index > -1) {
            this.items.splice(index, 1);
        }
        item.remove();
    }

    /**
     * Clear all timeline items
     */
    clear() {
        this.items.forEach(item => item.remove());
        this.items = [];
    }
}

// Helper function to create timeline detail row HTML
function createTimelineDetailRow(label, value, copyable = false) {
    const copyBtn = copyable ? `
        <button class="timeline-copy-btn" data-copy="${value}" aria-label="Copy ${label}">
            <i data-lucide="copy"></i> Copy
        </button>
    ` : '';

    return `
        <div class="timeline-detail-row">
            <span class="timeline-detail-label">${label}:</span>
            <span class="timeline-detail-value">
                ${value}
                ${copyBtn}
            </span>
        </div>
    `;
}

// Helper function to create status badge HTML
function createStatusBadge(label, type = 'info') {
    return `
        <span class="timeline-status-badge ${type}">
            ${label}
        </span>
    `;
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { EnhancedTimeline, createTimelineDetailRow, createStatusBadge };
}
