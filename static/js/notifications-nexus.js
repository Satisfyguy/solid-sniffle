// WebSocket Notifications System - Nexus Design
// Handles real-time notifications with Nexus toast UI

class NexusNotificationManager {
    constructor() {
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 3000;
        this.toastContainer = null;
        this.pendingOrdersCount = 0;
        this.notificationSound = true;
        this.init();
    }

    init() {
        this.createToastContainer();
        this.connect();
        this.fetchPendingCount();
    }

    createToastContainer() {
        if (document.getElementById('toast-container')) return;

        const container = document.createElement('div');
        container.id = 'toast-container';
        container.style.cssText = `
            position: fixed;
            top: var(--nexus-space-6);
            right: var(--nexus-space-6);
            z-index: var(--nexus-z-toast);
            display: flex;
            flex-direction: column;
            gap: var(--nexus-space-3);
            max-width: 400px;
            pointer-events: none;
        `;
        document.body.appendChild(container);
        this.toastContainer = container;
    }

    connect() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/`;

        console.log('[NEXUS WS] Connecting to:', wsUrl);

        try {
            this.ws = new WebSocket(wsUrl);

            this.ws.onopen = () => {
                console.log('[NEXUS WS] ✅ Connected');
                this.reconnectAttempts = 0;
                this.showToast(
                    'Connected',
                    'Real-time notifications enabled',
                    'success',
                    3000
                );
            };

            this.ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handleNotification(data);
                } catch (e) {
                    console.error('[NEXUS WS] Failed to parse message:', e);
                }
            };

            this.ws.onerror = (error) => {
                console.error('[NEXUS WS] Error:', error);
            };

            this.ws.onclose = () => {
                console.log('[NEXUS WS] Disconnected');
                this.attemptReconnect();
            };
        } catch (e) {
            console.error('[NEXUS WS] Failed to create WebSocket:', e);
            this.attemptReconnect();
        }
    }

    attemptReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('[NEXUS WS] Max reconnection attempts reached');
            this.showToast(
                'Connection Lost',
                'Unable to reconnect. Please refresh the page.',
                'destructive',
                0
            );
            return;
        }

        this.reconnectAttempts++;
        console.log(`[NEXUS WS] Reconnecting... (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);

        setTimeout(() => {
            this.connect();
        }, this.reconnectDelay);
    }

    handleNotification(data) {
        console.log('[NEXUS WS] Notification:', data);

        // Handle different notification types
        if (data.OrderStatusChanged) {
            this.handleOrderStatusChanged(data.OrderStatusChanged);
        } else if (data.EscrowStatusChanged) {
            this.handleEscrowStatusChanged(data.EscrowStatusChanged);
        } else if (data.TransactionConfirmed) {
            this.handleTransactionConfirmed(data.TransactionConfirmed);
        } else if (data.NewMessage) {
            this.handleNewMessage(data.NewMessage);
        } else if (data.ReviewInvitation) {
            this.handleReviewInvitation(data.ReviewInvitation);
        } else if (data.DisputeResolved) {
            this.handleDisputeResolved(data.DisputeResolved);
        }
    }

    handleOrderStatusChanged(data) {
        const statusConfig = {
            'pending': { emoji: '⏳', variant: 'default' },
            'funded': { emoji: '💰', variant: 'success' },
            'shipped': { emoji: '📦', variant: 'info' },
            'completed': { emoji: '✅', variant: 'success' },
            'cancelled': { emoji: '❌', variant: 'destructive' },
            'disputed': { emoji: '⚠️', variant: 'warning' },
            'refunded': { emoji: '↩️', variant: 'info' }
        };

        const config = statusConfig[data.new_status] || { emoji: '📋', variant: 'default' };
        const title = `${config.emoji} Order Update`;
        const message = `Order status changed to: ${data.new_status.toUpperCase()}`;

        // Check if we're on the order page or orders list
        const currentPath = window.location.pathname;
        const isOnOrderPage = currentPath.includes('/orders/') || currentPath === '/orders';

        if (isOnOrderPage) {
            // If on orders page, reload to show updated status
            this.showToast(title, message + ' - Refreshing...', config.variant, 2000);
            setTimeout(() => {
                window.location.reload();
            }, 2000);
        } else {
            // If on another page, show clickable notification
            this.showToast(
                title,
                message + ' - Click to view',
                config.variant,
                8000,
                () => window.location.href = `/orders/${data.order_id}`
            );
        }

        // Update badge count for new pending orders
        if (data.new_status === 'pending') {
            this.incrementBadge();
        }

        // Play notification sound
        if (this.notificationSound) {
            this.playNotificationSound();
        }
    }

    handleEscrowStatusChanged(data) {
        this.showToast(
            '🔒 Escrow Update',
            `Escrow status: ${data.new_status.toUpperCase()}`,
            'info',
            8000,
            () => window.location.href = `/escrow/${data.escrow_id}`
        );

        if (this.notificationSound) {
            this.playNotificationSound();
        }
    }

    handleTransactionConfirmed(data) {
        this.showToast(
            '⛓️ Transaction Confirmed',
            `${data.confirmations} confirmation(s)`,
            'success',
            8000
        );

        if (this.notificationSound) {
            this.playNotificationSound();
        }
    }

    handleNewMessage(data) {
        const preview = data.content.substring(0, 50) + (data.content.length > 50 ? '...' : '');
        this.showToast(
            '💬 New Message',
            preview,
            'info',
            8000
        );

        if (this.notificationSound) {
            this.playNotificationSound();
        }
    }

    handleReviewInvitation(data) {
        this.showToast(
            '⭐ Review Invitation',
            'Your order is complete! Leave a review for the vendor.',
            'info',
            0,
            () => window.location.href = `/escrow/${data.escrow_id}`
        );

        if (this.notificationSound) {
            this.playNotificationSound();
        }
    }

    handleDisputeResolved(data) {
        this.showToast(
            '⚖️ Dispute Resolved',
            `Resolution: ${data.resolution}`,
            'success',
            10000,
            () => window.location.href = `/escrow/${data.escrow_id}`
        );

        if (this.notificationSound) {
            this.playNotificationSound();
        }
    }

    showToast(title, message, variant = 'default', duration = 5000, onClick = null) {
        const toast = document.createElement('div');
        toast.className = `nexus-toast nexus-toast-${variant}`;

        // Nexus color variants
        const variants = {
            default: {
                bg: 'rgba(17, 17, 19, 0.95)',
                border: 'rgba(255, 255, 255, 0.1)',
                titleColor: 'var(--nexus-fg)',
                accent: 'var(--nexus-primary)'
            },
            success: {
                bg: 'rgba(16, 185, 129, 0.1)',
                border: 'rgba(16, 185, 129, 0.3)',
                titleColor: 'var(--nexus-success)',
                accent: 'var(--nexus-success)'
            },
            destructive: {
                bg: 'rgba(239, 68, 68, 0.1)',
                border: 'rgba(239, 68, 68, 0.3)',
                titleColor: 'var(--nexus-destructive)',
                accent: 'var(--nexus-destructive)'
            },
            warning: {
                bg: 'rgba(245, 158, 11, 0.1)',
                border: 'rgba(245, 158, 11, 0.3)',
                titleColor: 'var(--nexus-warning)',
                accent: 'var(--nexus-warning)'
            },
            info: {
                bg: 'rgba(59, 130, 246, 0.1)',
                border: 'rgba(59, 130, 246, 0.3)',
                titleColor: 'var(--nexus-info)',
                accent: 'var(--nexus-info)'
            }
        };

        const style = variants[variant] || variants.default;

        toast.style.cssText = `
            background: ${style.bg};
            border: 1px solid ${style.border};
            border-radius: var(--nexus-radius-lg);
            padding: var(--nexus-space-4);
            min-width: 320px;
            max-width: 400px;
            backdrop-filter: blur(12px);
            box-shadow: var(--nexus-shadow-lg);
            animation: nexusToastSlideIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
            cursor: ${onClick ? 'pointer' : 'default'};
            transition: transform 0.2s ease, box-shadow 0.2s ease;
            pointer-events: auto;
            font-family: var(--nexus-font-mono);
        `;

        toast.innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: start; gap: var(--nexus-space-3);">
                <div style="flex: 1;">
                    <div style="
                        font-size: var(--nexus-text-sm);
                        font-weight: var(--nexus-weight-bold);
                        color: ${style.titleColor};
                        letter-spacing: var(--nexus-tracking-wide);
                        text-transform: uppercase;
                        margin-bottom: var(--nexus-space-2);
                    ">
                        ${title}
                    </div>
                    <div style="
                        font-size: var(--nexus-text-sm);
                        color: var(--nexus-muted-fg);
                        line-height: var(--nexus-leading-relaxed);
                    ">
                        ${message}
                    </div>
                </div>
                <button class="nexus-toast-close" style="
                    background: none;
                    border: none;
                    color: var(--nexus-muted-fg);
                    font-size: var(--nexus-text-xl);
                    cursor: pointer;
                    padding: 0;
                    width: 24px;
                    height: 24px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    transition: color 0.2s ease;
                    border-radius: var(--nexus-radius-sm);
                " aria-label="Close notification">×</button>
            </div>
        `;

        // Add hover effects
        if (onClick) {
            toast.addEventListener('mouseenter', () => {
                toast.style.transform = 'translateX(-8px)';
                toast.style.boxShadow = 'var(--nexus-shadow-xl)';
            });
            toast.addEventListener('mouseleave', () => {
                toast.style.transform = 'translateX(0)';
                toast.style.boxShadow = 'var(--nexus-shadow-lg)';
            });
            toast.addEventListener('click', (e) => {
                if (!e.target.classList.contains('nexus-toast-close')) {
                    onClick();
                }
            });
        }

        // Close button functionality
        const closeBtn = toast.querySelector('.nexus-toast-close');
        closeBtn.addEventListener('mouseenter', () => {
            closeBtn.style.color = 'var(--nexus-fg)';
            closeBtn.style.background = 'rgba(255, 255, 255, 0.1)';
        });
        closeBtn.addEventListener('mouseleave', () => {
            closeBtn.style.color = 'var(--nexus-muted-fg)';
            closeBtn.style.background = 'none';
        });
        closeBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            this.removeToast(toast);
        });

        this.toastContainer.appendChild(toast);

        // Auto-remove after duration (0 = persistent)
        if (duration > 0) {
            setTimeout(() => {
                this.removeToast(toast);
            }, duration);
        }
    }

    removeToast(toast) {
        toast.style.animation = 'nexusToastSlideOut 0.3s cubic-bezier(0.5, 0, 0.75, 0)';
        setTimeout(() => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
            }
        }, 300);
    }

    incrementBadge() {
        this.pendingOrdersCount++;
        this.updateBadge();
    }

    updateBadge() {
        const badge = document.getElementById('orders-badge');
        if (badge) {
            if (this.pendingOrdersCount > 0) {
                badge.textContent = this.pendingOrdersCount;
                badge.style.display = 'inline-flex';
            } else {
                badge.style.display = 'none';
            }
        }
    }

    fetchPendingCount() {
        // Fetch initial pending count from server
        fetch('/api/orders/pending-count', {
            credentials: 'same-origin'
        })
        .then(res => res.json())
        .then(data => {
            this.pendingOrdersCount = data.count || 0;
            this.updateBadge();
        })
        .catch(err => {
            console.error('[NEXUS WS] Failed to fetch pending count:', err);
        });
    }

    playNotificationSound() {
        // Simple notification beep using Web Audio API
        try {
            const audioContext = new (window.AudioContext || window.webkitAudioContext)();
            const oscillator = audioContext.createOscillator();
            const gainNode = audioContext.createGain();

            oscillator.connect(gainNode);
            gainNode.connect(audioContext.destination);

            // Nexus sound: higher frequency for modern feel
            oscillator.frequency.value = 880; // A5 note
            oscillator.type = 'sine';

            gainNode.gain.setValueAtTime(0.08, audioContext.currentTime);
            gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.15);

            oscillator.start(audioContext.currentTime);
            oscillator.stop(audioContext.currentTime + 0.15);
        } catch (e) {
            // Silently fail if audio not supported
        }
    }

    toggleSound() {
        this.notificationSound = !this.notificationSound;
        console.log(`[NEXUS WS] Notification sound: ${this.notificationSound ? 'ON' : 'OFF'}`);
        return this.notificationSound;
    }
}

// Add Nexus animations
const style = document.createElement('style');
style.textContent = `
    @keyframes nexusToastSlideIn {
        from {
            transform: translateX(400px);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }

    @keyframes nexusToastSlideOut {
        from {
            transform: translateX(0);
            opacity: 1;
        }
        to {
            transform: translateX(400px);
            opacity: 0;
        }
    }

    /* Respect reduced motion preference */
    @media (prefers-reduced-motion: reduce) {
        .nexus-toast {
            animation: none !important;
            transition: none !important;
        }
    }
`;
document.head.appendChild(style);

// Initialize notification manager when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.nexusNotificationManager = new NexusNotificationManager();
        console.log('[NEXUS WS] ✅ Notification manager initialized');
    });
} else {
    window.nexusNotificationManager = new NexusNotificationManager();
    console.log('[NEXUS WS] ✅ Notification manager initialized');
}
