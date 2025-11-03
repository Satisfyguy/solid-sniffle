// Settings page JavaScript
// Handles wallet update responses with enhanced UX

// Real-time validation
const walletInput = document.getElementById('wallet_address');
const walletForm = document.querySelector('form[hx-post="/api/settings/update-wallet"]');

if (walletInput) {
  walletInput.addEventListener('input', function(e) {
    const addr = e.target.value.trim();
    const isValid = /^[48][a-zA-Z0-9]{94,105}$/.test(addr);

    if (addr.length === 0) {
      e.target.style.borderColor = '';
      e.target.style.boxShadow = '';
      return;
    }

    e.target.style.borderColor = isValid ? 'rgba(34, 197, 94, 0.5)' : 'rgba(239, 68, 68, 0.5)';
    e.target.style.boxShadow = isValid
      ? '0 0 0 3px rgba(34, 197, 94, 0.1)'
      : '0 0 0 3px rgba(239, 68, 68, 0.1)';
  });

  // Auto-trim on paste
  walletInput.addEventListener('paste', function(e) {
    setTimeout(() => {
      e.target.value = e.target.value.trim();
      e.target.dispatchEvent(new Event('input'));
    }, 10);
  });
}

// Confirmation modal for updating existing address
if (walletForm) {
  walletForm.addEventListener('htmx:confirm', function(e) {
    const existingAddress = document.querySelector('.wallet-address-display .address-text');

    if (existingAddress && existingAddress.textContent.trim().length > 0) {
      const newAddress = walletInput.value.trim();

      // Show confirmation dialog
      const confirmed = confirm(
        '⚠️ WARNING: Update Wallet Address?\n\n' +
        'You are about to change your payment address.\n\n' +
        'All future payments will be sent to the NEW address.\n\n' +
        'Current: ' + existingAddress.textContent.substring(0, 20) + '...\n' +
        'New: ' + newAddress.substring(0, 20) + '...\n\n' +
        'Continue with this change?'
      );

      if (!confirmed) {
        e.preventDefault();
      }
    }
  });
}

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

      // Clear input field
      if (walletInput) {
        walletInput.value = '';
        walletInput.style.borderColor = '';
        walletInput.style.boxShadow = '';
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
