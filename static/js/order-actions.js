/**
 * order-actions.js - Handles order action responses and UI updates
 *
 * This script enhances HTMX responses for order actions like
 * ship, complete, cancel, and dispute.
 * Also manages WebSocket connection for live order updates.
 */

document.addEventListener('DOMContentLoaded', function() {
    // Setup WebSocket for live order updates
    const orderDataDiv = document.getElementById('order-data');
    if (orderDataDiv) {
        const orderId = orderDataDiv.dataset.orderId;
        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const ws = new WebSocket(wsProtocol + '//' + window.location.host + '/ws/');

        ws.onopen = function() {
            console.log('[Order WS] Connected');
            ws.send(JSON.stringify({
                type: 'subscribe',
                channel: 'order:' + orderId
            }));
        };

        ws.onmessage = function(event) {
            const data = JSON.parse(event.data);
            if (data.type === 'order_update' && data.order_id === orderId) {
                console.log('[Order WS] Order updated, reloading page');
                location.reload();
            }
        };

        ws.onerror = function(error) {
            console.error('[Order WS] Error:', error);
        };

        ws.onclose = function() {
            console.log('[Order WS] Disconnected, will reload in 5 seconds');
            setTimeout(() => location.reload(), 5000);
        };
    }

    // HTMX response handlers
    // Listen for HTMX after-request event
    document.body.addEventListener('htmx:afterRequest', function(event) {
        const actionResult = document.getElementById('action-result');
        if (!actionResult) return;

        const xhr = event.detail.xhr;

        try {
            const response = JSON.parse(xhr.responseText);

            if (xhr.status >= 200 && xhr.status < 300) {
                // Success response
                let message = response.message || 'Action completed successfully';

                actionResult.innerHTML = `
                    <div style="padding: 12px; background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3); border-radius: 4px; margin-bottom: 15px;">
                        <p style="font-size: 12px; color: rgba(34, 197, 94, 0.9); margin: 0;">
                            ✓ ${escapeHtml(message)}
                        </p>
                    </div>
                `;

                // Reload page after 1.5 seconds to show updated status
                setTimeout(() => location.reload(), 1500);

            } else {
                // Error response
                let errorMessage = response.error || 'An error occurred';

                actionResult.innerHTML = `
                    <div style="padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 4px; margin-bottom: 15px;">
                        <p style="font-size: 12px; color: rgba(239, 68, 68, 0.9); margin: 0;">
                            ❌ ${escapeHtml(errorMessage)}
                        </p>
                    </div>
                `;
            }
        } catch (err) {
            console.error('Error parsing response:', err);

            actionResult.innerHTML = `
                <div style="padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 4px; margin-bottom: 15px;">
                    <p style="font-size: 12px; color: rgba(239, 68, 68, 0.9); margin: 0;">
                        ❌ Failed to process response. Please refresh the page.
                    </p>
                </div>
            `;
        }
    });

    // Listen for HTMX errors
    document.body.addEventListener('htmx:responseError', function(event) {
        const actionResult = document.getElementById('action-result');
        if (!actionResult) return;

        actionResult.innerHTML = `
            <div style="padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 4px; margin-bottom: 15px;">
                <p style="font-size: 12px; color: rgba(239, 68, 68, 0.9); margin: 0;">
                    ❌ Network error. Please check your connection and try again.
                </p>
            </div>
        `;
    });
});

/**
 * Escape HTML to prevent XSS
 */
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
