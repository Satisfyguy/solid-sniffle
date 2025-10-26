// Order Actions - Auto-reload after successful HTMX requests
document.body.addEventListener('htmx:afterRequest', function(event) {
    // Check if request was successful (2xx status code)
    if (event.detail.successful && event.detail.xhr.status >= 200 && event.detail.xhr.status < 300) {
        const url = event.detail.pathInfo.requestPath;
        // Reload page after ship or complete actions
        if (url.includes('/ship') || url.includes('/complete')) {
            console.log('Order action successful, reloading page...');
            setTimeout(() => window.location.reload(), 1500);
        }
    } else if (!event.detail.successful) {
        // Log error for debugging
        console.error('Order action failed:', event.detail.xhr.status, event.detail.xhr.responseText);
        
        // Try to parse and display error message
        try {
            const response = JSON.parse(event.detail.xhr.responseText);
            if (response.error) {
                const resultDiv = document.getElementById('action-result');
                if (resultDiv) {
                    resultDiv.innerHTML = `
                        <div style="padding: 10px; background: #3a1a1a; border: 1px solid #ef4444; border-radius: 4px; margin-bottom: 15px;">
                            <p style="font-size: 11px; color: #ef4444; margin: 0;">
                                ❌ ${response.error}
                            </p>
                        </div>
                    `;
                }
            }
        } catch (e) {
            console.error('Failed to parse error response:', e);
        }
    }
});
