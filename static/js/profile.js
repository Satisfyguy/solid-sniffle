document.addEventListener('DOMContentLoaded', function() {
    // Initialize Lucide icons (only if not already done by base.js)
    if (typeof lucide !== 'undefined' && !window.lucideInitialized) {
        lucide.createIcons();
        window.lucideInitialized = true;
    }

    const tabsContainer = document.querySelector('.tabs-container');
    if (tabsContainer) {
        tabsContainer.addEventListener('change', function(event) {
            if (event.target.classList.contains('tab-radio')) {
                const tabId = event.target.id.replace('tab-', 'content-');
                document.querySelectorAll('.tab-content').forEach(content => {
                    content.style.display = 'none';
                });
                const activeContent = document.getElementById(tabId);
                if (activeContent) {
                    activeContent.style.display = 'block';
                }

                // Load escrows when tab is activated
                if (tabId === 'content-escrows') {
                    loadUserEscrows();
                }
            }
        });

        // Set initial active tab content
        const initialActiveTab = tabsContainer.querySelector('.tab-radio:checked');
        if (initialActiveTab) {
            const tabId = initialActiveTab.id.replace('tab-', 'content-');
            const activeContent = document.getElementById(tabId);
            if (activeContent) {
                activeContent.style.display = 'block';
            }
        }
    }
});

/**
 * Load user's escrows from API
 */
async function loadUserEscrows() {
    try {
        const response = await fetch('/api/user/escrows');
        if (!response.ok) {
            console.error('Failed to load escrows:', response.statusText);
            return;
        }

        const escrows = await response.json();

        const emptyState = document.querySelector('.empty-state');
        const escrowsTable = document.querySelector('.escrows-table');
        const escrowsTbody = document.getElementById('escrows-tbody');

        if (!escrows || escrows.length === 0) {
            // Show empty state
            if (emptyState) emptyState.style.display = 'block';
            if (escrowsTable) escrowsTable.style.display = 'none';
            return;
        }

        // Hide empty state, show table
        if (emptyState) emptyState.style.display = 'none';
        if (escrowsTable) escrowsTable.style.display = 'table';

        // Populate table
        escrowsTbody.innerHTML = escrows.map(escrow => {
            const statusBadge = getStatusBadge(escrow.status);
            const roleDisplay = getUserRole(escrow);
            const amountXmr = (escrow.amount / 1000000000000).toFixed(6);

            return `
                <tr style="border-bottom: 1px solid hsl(var(--border));">
                    <td style="padding: 0.75rem;">
                        <a href="/orders/${escrow.order_id}" style="color: hsl(var(--accent)); text-decoration: none; font-family: monospace; font-size: 0.875rem;">
                            #${escrow.order_id.substring(0, 8)}
                        </a>
                    </td>
                    <td style="padding: 0.75rem;">
                        <span style="font-size: 0.875rem; font-weight: 500; color: hsl(var(--foreground));">${roleDisplay}</span>
                    </td>
                    <td style="padding: 0.75rem;">
                        <span style="font-size: 0.875rem; font-weight: 600; color: hsl(var(--accent)); font-family: monospace;">${amountXmr} XMR</span>
                    </td>
                    <td style="padding: 0.75rem;">
                        ${statusBadge}
                    </td>
                    <td style="padding: 0.75rem;">
                        <a href="/escrow/${escrow.id}" class="btn-secondary" style="display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.375rem 0.75rem; font-size: 0.875rem; text-decoration: none;">
                            View Details
                        </a>
                    </td>
                </tr>
            `;
        }).join('');

        // Re-initialize Lucide icons for dynamically added content
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }

    } catch (error) {
        console.error('Error loading escrows:', error);
    }
}

/**
 * Get status badge HTML
 */
function getStatusBadge(status) {
    const statusConfig = {
        'pending': { color: '#eab308', text: '⏳ PENDING' },
        'funded': { color: '#22c55e', text: '💰 FUNDED' },
        'released': { color: '#00d9ff', text: '✅ RELEASED' },
        'refunded': { color: '#ef4444', text: '↩️ REFUNDED' },
        'disputed': { color: '#ff6b35', text: '⚠️ DISPUTED' }
    };

    const config = statusConfig[status] || { color: '#6b7280', text: status.toUpperCase() };

    return `<span style="display: inline-flex; align-items: center; padding: 0.25rem 0.75rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; background-color: ${config.color}; color: #1A1A1A;">${config.text}</span>`;
}

/**
 * Get user's role in the escrow
 */
function getUserRole(escrow) {
    // This will be populated from session/context
    // For now, we determine based on IDs matching current user
    // The backend should include a 'user_role' field in the response
    return escrow.user_role || 'Participant';
}