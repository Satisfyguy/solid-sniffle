/**
 * Order Chat Component
 *
 * Real-time vendor-buyer chat using HTMX polling.
 * Production-grade with auto-scroll, error handling, and accessibility.
 *
 * @module order-chat
 */

class OrderChat {
    /**
     * Create an order chat instance
     * @param {Object} options - Configuration options
     * @param {string} options.orderId - Order ID
     * @param {string} options.csrfToken - CSRF token for submissions
     * @param {string} options.currentUserId - Current user ID
     * @param {string} options.currentUsername - Current user username
     * @param {number} [options.pollInterval=3000] - Polling interval in ms
     */
    constructor(options = {}) {
        this.orderId = options.orderId;
        this.csrfToken = options.csrfToken;
        this.currentUserId = options.currentUserId;
        this.currentUsername = options.currentUsername;
        this.pollInterval = options.pollInterval || 3000;

        this.container = null;
        this.messagesContainer = null;
        this.inputElement = null;
        this.sendButton = null;
        this.counterElement = null;
        this.lastMessageId = null;
        this.isLoadingMessages = false;
        this.isSending = false;
        this.pollTimer = null;

        this.init();
    }

    /**
     * Initialize chat component
     */
    init() {
        this.container = document.getElementById('order-chat');

        if (!this.container) {
            console.error('Order chat container not found');
            return;
        }

        this.messagesContainer = this.container.querySelector('.order-chat-messages');
        this.inputElement = this.container.querySelector('#chat-message-input');
        this.sendButton = this.container.querySelector('.order-chat-send-btn');
        this.counterElement = this.container.querySelector('.order-chat-input-counter');

        if (!this.messagesContainer || !this.inputElement || !this.sendButton) {
            console.error('Chat elements not found');
            return;
        }

        this.attachEventListeners();
        this.loadMessages();
        this.startPolling();

        console.log('✅ Order chat initialized', {
            orderId: this.orderId,
            pollInterval: this.pollInterval
        });
    }

    /**
     * Attach event listeners
     */
    attachEventListeners() {
        // Send button click
        this.sendButton.addEventListener('click', () => this.sendMessage());

        // Enter key to send (Shift+Enter for new line)
        this.inputElement.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                this.sendMessage();
            }
        });

        // Character counter
        this.inputElement.addEventListener('input', () => this.updateCharCounter());

        // Focus input on mount
        setTimeout(() => this.inputElement.focus(), 100);
    }

    /**
     * Load messages from API
     */
    async loadMessages() {
        if (this.isLoadingMessages) return;

        this.isLoadingMessages = true;

        try {
            const response = await fetch(`/api/orders/${this.orderId}/messages`);

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }

            const data = await response.json();
            this.renderMessages(data.messages);

        } catch (error) {
            console.error('Failed to load messages:', error);
            this.showError('Failed to load messages. Please refresh the page.');
        } finally {
            this.isLoadingMessages = false;
        }
    }

    /**
     * Render messages to the DOM
     * @param {Array} messages - Array of message objects
     */
    renderMessages(messages) {
        if (messages.length === 0) {
            this.renderEmptyState();
            return;
        }

        // Clear empty state if exists
        const emptyState = this.messagesContainer.querySelector('.order-chat-empty');
        if (emptyState) {
            emptyState.remove();
        }

        // Check if we need to scroll (user is at bottom)
        const shouldScroll = this.isScrolledToBottom();

        // Render each message
        messages.forEach(message => {
            // Skip if already rendered
            if (this.lastMessageId && message.id === this.lastMessageId) {
                return;
            }

            // Check if message already exists
            const existingMessage = this.messagesContainer.querySelector(`[data-message-id="${message.id}"]`);
            if (existingMessage) {
                return;
            }

            const messageEl = this.createMessageElement(message);
            this.messagesContainer.appendChild(messageEl);

            // Update last message ID
            this.lastMessageId = message.id;
        });

        // Auto-scroll if user was at bottom
        if (shouldScroll) {
            this.scrollToBottom();
        }

        // Initialize Lucide icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Create message HTML element
     * @param {Object} message - Message data
     * @returns {HTMLElement}
     */
    createMessageElement(message) {
        const messageEl = document.createElement('div');
        messageEl.className = `chat-message ${message.is_current_user ? 'self' : 'other'}`;
        messageEl.setAttribute('data-message-id', message.id);

        const avatar = message.sender_username.charAt(0).toUpperCase();
        const timestamp = this.formatTimestamp(message.created_at);

        messageEl.innerHTML = `
            <div class="chat-message-avatar">${avatar}</div>
            <div class="chat-message-content">
                <div class="chat-message-header">
                    <span class="chat-message-username">${this.escapeHtml(message.sender_username)}</span>
                    <span class="chat-message-time">${timestamp}</span>
                </div>
                <div class="chat-message-bubble">
                    ${this.escapeHtml(message.message)}
                </div>
            </div>
        `;

        return messageEl;
    }

    /**
     * Render empty state
     */
    renderEmptyState() {
        this.messagesContainer.innerHTML = `
            <div class="order-chat-empty">
                <i data-lucide="message-circle"></i>
                <p>No messages yet. Start the conversation!</p>
            </div>
        `;

        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    }

    /**
     * Send a new message
     */
    async sendMessage() {
        const message = this.inputElement.value.trim();

        if (!message || message.length === 0) {
            return;
        }

        if (message.length > 2000) {
            this.showError('Message is too long (max 2000 characters)');
            return;
        }

        if (this.isSending) {
            return;
        }

        // Set loading state
        this.isSending = true;
        this.sendButton.disabled = true;
        this.sendButton.classList.add('loading');

        try {
            const response = await fetch(`/api/orders/${this.orderId}/messages`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': this.csrfToken
                },
                body: JSON.stringify({ message })
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || 'Failed to send message');
            }

            const newMessage = await response.json();

            // Clear input
            this.inputElement.value = '';
            this.updateCharCounter();

            // Add message to DOM
            this.renderMessages([newMessage]);

            // Focus input
            this.inputElement.focus();

        } catch (error) {
            console.error('Failed to send message:', error);
            this.showError(error.message || 'Failed to send message. Please try again.');
        } finally {
            // Clear loading state
            this.isSending = false;
            this.sendButton.disabled = false;
            this.sendButton.classList.remove('loading');
        }
    }

    /**
     * Start polling for new messages
     */
    startPolling() {
        // Clear existing timer
        if (this.pollTimer) {
            clearInterval(this.pollTimer);
        }

        // Poll every X seconds
        this.pollTimer = setInterval(() => {
            this.loadMessages();
        }, this.pollInterval);

        // Stop polling when page is hidden
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                clearInterval(this.pollTimer);
            } else {
                this.startPolling();
                this.loadMessages(); // Load immediately on tab focus
            }
        });
    }

    /**
     * Stop polling
     */
    stopPolling() {
        if (this.pollTimer) {
            clearInterval(this.pollTimer);
            this.pollTimer = null;
        }
    }

    /**
     * Update character counter
     */
    updateCharCounter() {
        const length = this.inputElement.value.length;
        this.counterElement.textContent = `${length} / 2000`;

        // Update counter color
        this.counterElement.classList.remove('warning', 'error');

        if (length > 1900) {
            this.counterElement.classList.add('warning');
        }

        if (length > 2000) {
            this.counterElement.classList.add('error');
        }

        // Disable send button if too long
        this.sendButton.disabled = length > 2000 || length === 0;
    }

    /**
     * Check if user is scrolled to bottom
     * @returns {boolean}
     */
    isScrolledToBottom() {
        const threshold = 50; // pixels
        const position = this.messagesContainer.scrollTop + this.messagesContainer.clientHeight;
        const bottom = this.messagesContainer.scrollHeight;

        return bottom - position < threshold;
    }

    /**
     * Scroll to bottom of messages
     */
    scrollToBottom() {
        this.messagesContainer.scrollTo({
            top: this.messagesContainer.scrollHeight,
            behavior: 'smooth'
        });
    }

    /**
     * Format Unix timestamp to readable time
     * @param {number} timestamp - Unix timestamp
     * @returns {string}
     */
    formatTimestamp(timestamp) {
        const date = new Date(timestamp * 1000);
        const now = new Date();

        const isToday = date.toDateString() === now.toDateString();

        if (isToday) {
            return date.toLocaleTimeString('en-US', {
                hour: '2-digit',
                minute: '2-digit'
            });
        }

        const yesterday = new Date(now);
        yesterday.setDate(yesterday.getDate() - 1);
        const isYesterday = date.toDateString() === yesterday.toDateString();

        if (isYesterday) {
            return 'Yesterday ' + date.toLocaleTimeString('en-US', {
                hour: '2-digit',
                minute: '2-digit'
            });
        }

        return date.toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
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
     * Show error message (simple toast)
     * @param {string} message - Error message
     */
    showError(message) {
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
     * Destroy chat instance (cleanup)
     */
    destroy() {
        this.stopPolling();

        // Remove event listeners would go here if we stored references
    }
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { OrderChat };
}
