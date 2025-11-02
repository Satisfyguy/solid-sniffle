document.addEventListener('DOMContentLoaded', function() {
    const escrowId = document.getElementById('order-data').getAttribute('data-order-id');
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(wsProtocol + '//' + window.location.host + '/ws/');

    ws.onopen = function() {
        console.log('WebSocket connected for escrow updates');
        ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'escrow:' + escrowId
        }));
    };

    ws.onmessage = function(event) {
        const data = JSON.parse(event.data);
        if (data.type === 'escrow_update' && data.escrow_id === escrowId) {
            console.log('Escrow status updated:', data.status);
            // Reload page to show updated escrow status
            location.reload();
        }
    };

    ws.onerror = function(error) {
        console.error('WebSocket error:', error);
    };

    ws.onclose = function() {
        console.log('WebSocket disconnected');
        // Reconnect after 5 seconds
        setTimeout(() => location.reload(), 5000);
    };
});