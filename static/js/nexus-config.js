/**
 * nexus-config.js - Global Nexus configuration and HTMX error handlers
 *
 * This file contains:
 * - Debug mode configuration
 * - Global HTMX error handlers
 * - Global HTMX timeout handlers
 */

// Global debug flag - enable via URL (?debug=1) or localStorage
window.NEXUS_DEBUG = (
    new URLSearchParams(window.location.search).get('debug') === '1' ||
    localStorage.getItem('nexus_debug') === '1'
);

// Save debug state if URL parameter is set
if (new URLSearchParams(window.location.search).get('debug') === '1') {
    localStorage.setItem('nexus_debug', '1');
}

// Helper to disable debug mode
window.disableDebug = function() {
    localStorage.removeItem('nexus_debug');
    window.NEXUS_DEBUG = false;
    console.log('[NEXUS] Debug mode disabled. Reload to apply.');
};

// Global error handler for all HTMX requests
document.body.addEventListener('htmx:responseError', function(event) {
    const status = event.detail.xhr.status;
    const url = event.detail.requestConfig.path;

    let message = 'AN ERROR OCCURRED. PLEASE TRY AGAIN.';
    let title = 'ERROR';

    // Handle specific HTTP status codes
    if (status === 401) {
        title = 'UNAUTHORIZED';
        message = 'SESSION EXPIRED. REDIRECTING TO LOGIN...';
        setTimeout(() => window.location.href = '/login', 2000);
    } else if (status === 403) {
        title = 'FORBIDDEN';
        message = 'YOU DO NOT HAVE PERMISSION TO PERFORM THIS ACTION.';
    } else if (status === 404) {
        title = 'NOT FOUND';
        message = 'THE REQUESTED RESOURCE COULD NOT BE FOUND.';
    } else if (status === 422) {
        title = 'VALIDATION ERROR';
        message = 'PLEASE CHECK YOUR INPUT AND TRY AGAIN.';
    } else if (status === 429) {
        title = 'RATE LIMIT';
        message = 'TOO MANY REQUESTS. PLEASE WAIT A MOMENT.';
    } else if (status >= 500) {
        title = 'SERVER ERROR';
        message = 'A SERVER ERROR OCCURRED. OUR TEAM HAS BEEN NOTIFIED.';
    } else if (status === 0) {
        title = 'CONNECTION ERROR';
        message = 'COULD NOT CONNECT TO SERVER. CHECK YOUR TOR CONNECTION.';
    }

    // Create error alert (NEXUS design system)
    const alertDiv = document.createElement('div');
    alertDiv.className = 'nexus-alert nexus-alert-error nexus-alert-dismissible';
    alertDiv.setAttribute('role', 'alert');
    alertDiv.style.cssText = 'position: fixed; top: 80px; right: 20px; z-index: 10000; max-width: 400px; animation: slideInRight 0.3s ease;';

    alertDiv.innerHTML = `
        <div class="nexus-alert-icon">❌</div>
        <div class="nexus-alert-content">
            <div class="nexus-alert-title">${escapeHtml(title)}</div>
            <div class="nexus-alert-message">${escapeHtml(message)}</div>
        </div>
        <button class="nexus-alert-close" aria-label="Close alert">✕</button>
    `;

    // Add close handler
    const closeBtn = alertDiv.querySelector('.nexus-alert-close');
    closeBtn.addEventListener('click', function() {
        alertDiv.remove();
    });

    // Remove any existing error alerts
    document.querySelectorAll('.nexus-alert-error').forEach(el => el.remove());

    // Insert new alert
    document.body.appendChild(alertDiv);

    // Auto-dismiss after 5 seconds (unless it's a redirect message)
    if (status !== 401) {
        setTimeout(() => {
            if (alertDiv.parentNode) {
                alertDiv.remove();
            }
        }, 5000);
    }

    // Log for debugging (non-sensitive info only)
    console.error(`[HTMX Error] ${status} on ${url}`);
});

// Global timeout handler
document.body.addEventListener('htmx:timeout', function(event) {
    const alertDiv = document.createElement('div');
    alertDiv.className = 'nexus-alert nexus-alert-warning nexus-alert-dismissible';
    alertDiv.setAttribute('role', 'alert');
    alertDiv.style.cssText = 'position: fixed; top: 80px; right: 20px; z-index: 10000; max-width: 400px; animation: slideInRight 0.3s ease;';

    alertDiv.innerHTML = `
        <div class="nexus-alert-icon">⚠️</div>
        <div class="nexus-alert-content">
            <div class="nexus-alert-title">TIMEOUT</div>
            <div class="nexus-alert-message">REQUEST TIMED OUT. THIS MAY BE NORMAL FOR BLOCKCHAIN OPERATIONS.</div>
        </div>
        <button class="nexus-alert-close" aria-label="Close alert">✕</button>
    `;

    // Add close handler
    const closeBtn = alertDiv.querySelector('.nexus-alert-close');
    closeBtn.addEventListener('click', function() {
        alertDiv.remove();
    });

    document.body.appendChild(alertDiv);

    setTimeout(() => {
        if (alertDiv.parentNode) {
            alertDiv.remove();
        }
    }, 5000);

    console.warn('[HTMX Timeout] Request timed out');
});

/**
 * Escape HTML to prevent XSS in alerts
 */
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
