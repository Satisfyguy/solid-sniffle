// WebSocket Notifications System
// Handles real-time notifications with toast UI

class NotificationManager {
    constructor() {
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 3000;
        this.toastContainer = null;
        this.pendingOrdersCount = 0;
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
            top: 80px;
            right: 20px;
            z-index: 10000;
            display: flex;
            flex-direction: column;
            gap: 10px;
            max-width: 400px;
        `;
        document.body.appendChild(container);
        this.toastContainer = container;
    }

    connect() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/`;
        
        console.log('Connecting to WebSocket:', wsUrl);
        
        try {
            this.ws = new WebSocket(wsUrl);
            
            this.ws.onopen = () => {
                console.log('✅ WebSocket connected');
                this.reconnectAttempts = 0;
                this.showToast('Connected', 'Real-time notifications enabled', 'success', 3000);
            };
            
            this.ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handleNotification(data);
                } catch (e) {
                    console.error('Failed to parse WebSocket message:', e);
                }
            };
            
            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
            };
            
            this.ws.onclose = () => {
                console.log('WebSocket disconnected');
                this.attemptReconnect();
            };
        } catch (e) {
            console.error('Failed to create WebSocket:', e);
            this.attemptReconnect();
        }
    }

    attemptReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            this.showToast(
                'Connection Lost',
                'Unable to reconnect. Please refresh the page.',
                'error',
                0
            );
            return;
        }
        
        this.reconnectAttempts++;
        console.log(`Reconnecting... (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
        
        setTimeout(() => {
            this.connect();
        }, this.reconnectDelay);
    }

    handleNotification(data) {
        console.log('Received notification:', data);
        
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
        const statusEmojis = {
            'pending': '⏳',
            'funded': '💰',
            'shipped': '📦',
            'completed': '✅',
            'cancelled': '❌',
            'disputed': '⚠️',
            'refunded': '↩️'
        };
        
        const emoji = statusEmojis[data.new_status] || '📋';
        const title = `${emoji} Order Update`;
        const message = `Order status changed to: ${data.new_status.toUpperCase()}`;
        
        // Check if we're on the order page or orders list
        const currentPath = window.location.pathname;
        const isOnOrderPage = currentPath.includes('/orders/') || currentPath === '/orders';
        
        if (isOnOrderPage) {
            // If on orders page, reload to show updated status
            this.showToast(title, message + ' - Refreshing...', 'success', 2000);
            setTimeout(() => {
                window.location.reload();
            }, 2000);
        } else {
            // If on another page, show clickable notification
            this.showToast(title, message, 'info', 8000, () => {
                window.location.href = `/orders/${data.order_id}`;
            });
        }
        
        // Update badge count for new pending orders
        if (data.new_status === 'pending') {
            this.incrementBadge();
        }
        
        // Play notification sound
        this.playNotificationSound();
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
                badge.style.display = 'inline-block';
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
            console.error('Failed to fetch pending count:', err);
        });
    }

    handleEscrowStatusChanged(data) {
        const title = '🔒 Escrow Update';
        const message = `Escrow status: ${data.new_status.toUpperCase()}`;
        
        this.showToast(title, message, 'info', 8000, () => {
            window.location.href = `/escrow/${data.escrow_id}`;
        });
        
        this.playNotificationSound();
    }

    handleTransactionConfirmed(data) {
        const title = '⛓️ Transaction Confirmed';
        const message = `${data.confirmations} confirmation(s)`;
        
        this.showToast(title, message, 'success', 8000);
        this.playNotificationSound();
    }

    handleNewMessage(data) {
        const title = '💬 New Message';
        const message = data.content.substring(0, 50) + (data.content.length > 50 ? '...' : '');
        
        this.showToast(title, message, 'info', 8000);
        this.playNotificationSound();
    }

    handleReviewInvitation(data) {
        const title = '⭐ Review Invitation';
        const message = 'Your order is complete! Leave a review for the vendor.';
        
        this.showToast(title, message, 'info', 0, () => {
            window.location.href = `/escrow/${data.escrow_id}`;
        });
        
        this.playNotificationSound();
    }

    handleDisputeResolved(data) {
        const title = '⚖️ Dispute Resolved';
        const message = `Resolution: ${data.resolution}`;
        
        this.showToast(title, message, 'success', 10000, () => {
            window.location.href = `/escrow/${data.escrow_id}`;
        });
        
        this.playNotificationSound();
    }

    showToast(title, message, type = 'info', duration = 5000, onClick = null) {
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        
        const colors = {
            success: { bg: '#1a3a1a', border: '#22c55e', text: '#22c55e' },
            error: { bg: '#3a1a1a', border: '#ef4444', text: '#ef4444' },
            info: { bg: '#1a1a3a', border: '#3b82f6', text: '#3b82f6' },
            warning: { bg: '#3a2a1a', border: '#f59e0b', text: '#f59e0b' }
        };
        
        const color = colors[type] || colors.info;
        
        toast.style.cssText = `
            background: ${color.bg};
            border: 1px solid ${color.border};
            border-radius: 8px;
            padding: 16px;
            min-width: 300px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
            animation: slideIn 0.3s ease-out;
            cursor: ${onClick ? 'pointer' : 'default'};
            transition: transform 0.2s;
        `;
        
        toast.innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: start; gap: 12px;">
                <div style="flex: 1;">
                    <div style="font-size: 13px; font-weight: bold; color: ${color.text}; letter-spacing: 1px; text-transform: uppercase; margin-bottom: 6px;">
                        ${title}
                    </div>
                    <div style="font-size: 12px; color: #f5f5f5; line-height: 1.5;">
                        ${message}
                    </div>
                </div>
                <button class="toast-close" style="
                    background: none;
                    border: none;
                    color: #888;
                    font-size: 18px;
                    cursor: pointer;
                    padding: 0;
                    width: 20px;
                    height: 20px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    transition: color 0.2s;
                ">×</button>
            </div>
        `;
        
        // Add hover effect
        if (onClick) {
            toast.addEventListener('mouseenter', () => {
                toast.style.transform = 'translateX(-5px)';
            });
            toast.addEventListener('mouseleave', () => {
                toast.style.transform = 'translateX(0)';
            });
            toast.addEventListener('click', (e) => {
                if (!e.target.classList.contains('toast-close')) {
                    onClick();
                }
            });
        }
        
        // Close button
        const closeBtn = toast.querySelector('.toast-close');
        closeBtn.addEventListener('mouseenter', () => {
            closeBtn.style.color = '#f5f5f5';
        });
        closeBtn.addEventListener('mouseleave', () => {
            closeBtn.style.color = '#888';
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
        toast.style.animation = 'slideOut 0.3s ease-out';
        setTimeout(() => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
            }
        }, 300);
    }

    playNotificationSound() {
        // Simple notification beep using Web Audio API
        try {
            const audioContext = new (window.AudioContext || window.webkitAudioContext)();
            const oscillator = audioContext.createOscillator();
            const gainNode = audioContext.createGain();
            
            oscillator.connect(gainNode);
            gainNode.connect(audioContext.destination);
            
            oscillator.frequency.value = 800;
            oscillator.type = 'sine';
            
            gainNode.gain.setValueAtTime(0.1, audioContext.currentTime);
            gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.1);
            
            oscillator.start(audioContext.currentTime);
            oscillator.stop(audioContext.currentTime + 0.1);
        } catch (e) {
            // Silently fail if audio not supported
        }
    }
}

// Add CSS animations
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from {
            transform: translateX(400px);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }
    
    @keyframes slideOut {
        from {
            transform: translateX(0);
            opacity: 1;
        }
        to {
            transform: translateX(400px);
            opacity: 0;
        }
    }
`;
document.head.appendChild(style);

// Initialize notification manager when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.notificationManager = new NotificationManager();
    });
} else {
    window.notificationManager = new NotificationManager();
}
