// Settings page JavaScript
// Handles wallet update responses

document.body.addEventListener('htmx:afterRequest', function(event) {
  if (event.detail.pathInfo.requestPath === '/api/settings/update-wallet') {
    const resultDiv = document.getElementById('wallet-result');

    if (event.detail.successful) {
      // Success - show toast
      if (window.notificationManager) {
        window.notificationManager.showToast(
          '✅ Wallet Updated',
          'Your Monero wallet address has been saved successfully.',
          'success',
          3000
        );
      }

      // HTMX already updates #wallet-result with the new address from backend
      // No need to manually set innerHTML or reload the page
      // The backend returns the complete HTML with success message + new address
    } else {
      // Error handling with toast
      const errorMessage = event.detail.xhr?.responseText || 'Failed to update wallet address';

      if (window.notificationManager) {
        window.notificationManager.showToast(
          '❌ Update Failed',
          errorMessage,
          'error',
          5000
        );
      }

      resultDiv.innerHTML = `
        <div class="nexus-alert nexus-alert-error" style="padding: 1rem; border-radius: 8px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); color: hsl(0, 85%, 70%); display: flex; align-items: center; gap: 0.75rem;">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="15" y1="9" x2="9" y2="15"></line>
            <line x1="9" y1="9" x2="15" y2="15"></line>
          </svg>
          <div>
            <strong style="display: block; margin-bottom: 0.25rem;">Update Failed</strong>
            <small>${errorMessage}</small>
          </div>
        </div>
      `;
    }
  }
});
