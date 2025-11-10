// Base template JavaScript
// Handles HTMX config, dark mode, and user menu

// HTMX configuration for Tor-safe navigation
document.body.addEventListener('htmx:configRequest', function(evt) {
    evt.detail.headers['X-Requested-With'] = 'XMLHttpRequest';
});

// Dark mode toggle (stored in localStorage)
const savedTheme = localStorage.getItem('theme') || 'light';
if (savedTheme === 'dark') {
    document.body.classList.add('dark-mode');
}

// User menu toggle
document.addEventListener('DOMContentLoaded', function() {
    const userMenuBtn = document.getElementById('user-menu-btn');
    const userMenuDropdown = document.getElementById('user-menu-dropdown');

    if (userMenuBtn && userMenuDropdown) {
        userMenuBtn.addEventListener('click', function(e) {
            e.stopPropagation();
            const isOpen = userMenuDropdown.style.display === 'block';
            userMenuDropdown.style.display = isOpen ? 'none' : 'block';
        });

        // Close menu when clicking outside
        document.addEventListener('click', function(e) {
            if (!e.target.closest('.user-menu-container')) {
                userMenuDropdown.style.display = 'none';
            }
        });

        // Close menu on escape key
        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape') {
                userMenuDropdown.style.display = 'none';
            }
        });
    }

    // Initialize Lucide icons (if available and not already initialized)
    if (typeof lucide !== 'undefined' && !window.lucideInitialized) {
        lucide.createIcons();
        window.lucideInitialized = true;
    }

    // Escrow Notifications Widget
    initEscrowNotifications();
});

/**
 * Initialize Escrow Notifications Widget
 */
function initEscrowNotifications() {
    const notificationsBtn = document.getElementById('escrow-notifications-btn');
    const notificationsDropdown = document.getElementById('escrow-notifications-dropdown');
    const notificationCount = document.getElementById('escrow-notification-count');

    if (!notificationsBtn || !notificationsDropdown) {
        return; // Not logged in or widget not present
    }

    // Toggle dropdown
    notificationsBtn.addEventListener('click', function(e) {
        e.stopPropagation();
        const isVisible = notificationsDropdown.style.display === 'block';

        if (isVisible) {
            notificationsDropdown.style.display = 'none';
        } else {
            notificationsDropdown.style.display = 'block';
            loadEscrowNotifications();
        }
    });

    // Close dropdown when clicking outside
    document.addEventListener('click', function(e) {
        if (!notificationsBtn.contains(e.target) && !notificationsDropdown.contains(e.target)) {
            notificationsDropdown.style.display = 'none';
        }
    });

    // Close on escape key
    document.addEventListener('keydown', function(e) {
        if (e.key === 'Escape') {
            notificationsDropdown.style.display = 'none';
        }
    });

    // Load notification count on page load
    loadNotificationsCount();
}

/**
 * Load notification count (escrows needing attention)
 */
async function loadNotificationsCount() {
    try {
        const response = await fetch('/api/user/escrows');
        if (!response.ok) return;

        const escrows = await response.json();

        // Count escrows needing attention
        const needsAttention = escrows.filter(e =>
            e.status === 'pending' ||
            e.multisig_phase === 'awaiting_signatures' ||
            e.status === 'disputed'
        ).length;

        const badge = document.getElementById('escrow-notification-count');
        if (badge && needsAttention > 0) {
            badge.textContent = needsAttention;
            badge.style.display = 'flex';
        }
    } catch (error) {
        // Silent fail - user might not be logged in
        console.debug('Escrow notifications not available');
    }
}

/**
 * Load escrow notifications for dropdown
 */
async function loadEscrowNotifications() {
    const listContainer = document.getElementById('escrow-notifications-list');
    const countSpan = document.getElementById('dropdown-escrow-count');

    if (!listContainer) return;

    try {
        const response = await fetch('/api/user/escrows');
        if (!response.ok) {
            listContainer.innerHTML = '<div class="empty-state">Failed to load escrows</div>';
            return;
        }

        const escrows = await response.json();

        if (escrows.length === 0) {
            listContainer.innerHTML = `
                <div class="empty-state">
                    <p style="margin: 0;">No active escrows</p>
                    <a href="/listings" style="color: var(--color-accent); font-size: 0.875rem; margin-top: 0.5rem; display: inline-block;">Browse Listings</a>
                </div>
            `;
            if (countSpan) countSpan.textContent = '0 active';
            return;
        }

        // Sort by most recent first
        escrows.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));

        // Take only first 5
        const recentEscrows = escrows.slice(0, 5);

        if (countSpan) countSpan.textContent = `${escrows.length} active`;

        listContainer.innerHTML = recentEscrows.map(escrow => {
            const icon = getEscrowIcon(escrow);
            const message = getEscrowMessage(escrow);
            const timeAgo = formatTimeAgo(escrow.created_at);

            return `
                <div class="escrow-notification-item" onclick="window.location.href='/escrow/${escrow.id}'">
                    <div class="escrow-notification-content">
                        <div class="escrow-notification-title">
                            ${icon}
                            <span>Order #${escrow.order_id.substring(0, 8)}</span>
                        </div>
                        <div class="escrow-notification-message">${message}</div>
                        <div class="escrow-notification-meta">
                            <span class="escrow-notification-time">${timeAgo}</span>
                            <span class="escrow-status-badge escrow-status-${escrow.status}">${escrow.status}</span>
                        </div>
                    </div>
                </div>
            `;
        }).join('');

    } catch (error) {
        console.error('Error loading escrow notifications:', error);
        listContainer.innerHTML = '<div class="empty-state">Error loading escrows</div>';
    }
}

/**
 * Get icon for escrow based on status
 */
function getEscrowIcon(escrow) {
    const icons = {
        'pending': '⏳',
        'funded': '💰',
        'disputed': '⚠️',
        'released': '✅',
        'refunded': '↩️'
    };
    return icons[escrow.status] || '🔒';
}

/**
 * Get message for escrow based on status and user role
 */
function getEscrowMessage(escrow) {
    const role = escrow.user_role;

    if (escrow.status === 'pending') {
        if (role === 'Buyer') return 'Awaiting your funding';
        if (role === 'Vendor') return 'Waiting for buyer to fund';
        return 'Awaiting funding';
    }

    if (escrow.status === 'funded') {
        if (role === 'Vendor') return 'Funded - ready to ship';
        if (role === 'Buyer') return 'Funded - awaiting shipment';
        return 'Escrow funded';
    }

    if (escrow.status === 'disputed') return 'Dispute in progress';
    if (escrow.status === 'released') return 'Funds released to vendor';
    if (escrow.status === 'refunded') return 'Funds refunded to buyer';

    return `${(escrow.amount / 1000000000000).toFixed(4)} XMR in escrow`;
}

/**
 * Format timestamp to relative time
 */
function formatTimeAgo(timestamp) {
    const now = new Date();
    const past = new Date(timestamp);
    const diffMs = now - past;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return past.toLocaleDateString();
}
