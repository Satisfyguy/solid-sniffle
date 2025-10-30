/**
 * Dispute Modal Handler
 * Production-ready modal for raising order disputes
 */

document.addEventListener('DOMContentLoaded', () => {
    const disputeModal = document.getElementById('dispute-modal');
    const disputeForm = document.getElementById('dispute-form');
    const disputeTextarea = document.getElementById('dispute-reason');
    const disputeCharCount = document.getElementById('dispute-char-count');
    const disputeCancelBtn = document.getElementById('dispute-cancel');
    const disputeSubmitBtn = document.getElementById('dispute-submit');

    if (!disputeModal) return;

    // Character counter
    if (disputeTextarea && disputeCharCount) {
        disputeTextarea.addEventListener('input', () => {
            const length = disputeTextarea.value.length;
            disputeCharCount.textContent = `${length}/1000`;

            // Validation feedback
            if (length < 10) {
                disputeCharCount.style.color = 'var(--color-error)';
            } else if (length > 900) {
                disputeCharCount.style.color = 'var(--color-warning)';
            } else {
                disputeCharCount.style.color = 'var(--color-text-secondary)';
            }
        });
    }

    // Open modal handler
    window.openDisputeModal = (orderId) => {
        disputeModal.style.display = 'flex';
        disputeForm.setAttribute('data-order-id', orderId);
        disputeTextarea.value = '';
        disputeTextarea.focus();
        if (disputeCharCount) disputeCharCount.textContent = '0/1000';
    };

    // Close modal handler
    const closeModal = () => {
        disputeModal.style.display = 'none';
        disputeTextarea.value = '';
    };

    if (disputeCancelBtn) {
        disputeCancelBtn.addEventListener('click', closeModal);
    }

    // Close on outside click
    disputeModal.addEventListener('click', (e) => {
        if (e.target === disputeModal) {
            closeModal();
        }
    });

    // Close on ESC key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && disputeModal.style.display === 'flex') {
            closeModal();
        }
    });

    // Form submission
    if (disputeForm) {
        disputeForm.addEventListener('submit', async (e) => {
            e.preventDefault();

            const orderId = disputeForm.getAttribute('data-order-id');
            const reason = disputeTextarea.value.trim();

            // Client-side validation
            if (reason.length < 10) {
                alert('Dispute reason must be at least 10 characters long.');
                return;
            }

            if (reason.length > 1000) {
                alert('Dispute reason must be 1000 characters or less.');
                return;
            }

            // Disable submit button
            disputeSubmitBtn.disabled = true;
            disputeSubmitBtn.textContent = 'Submitting...';

            try {
                // Get CSRF token
                const csrfToken = document.querySelector('meta[name="csrf-token"]')?.content;

                const response = await fetch(`/api/orders/${orderId}/dispute`, {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-CSRF-Token': csrfToken
                    },
                    body: JSON.stringify({ reason })
                });

                const data = await response.json();

                if (response.ok) {
                    closeModal();
                    // Reload page to show updated status
                    window.location.reload();
                } else {
                    alert(`Error: ${data.error || 'Failed to raise dispute'}`);
                    disputeSubmitBtn.disabled = false;
                    disputeSubmitBtn.textContent = 'Submit Dispute';
                }
            } catch (error) {
                console.error('Dispute submission error:', error);
                alert('Network error. Please try again.');
                disputeSubmitBtn.disabled = false;
                disputeSubmitBtn.textContent = 'Submit Dispute';
            }
        });
    }
});
