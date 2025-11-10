/**
 * Checkout Stepper Initialization
 * Moved from inline script to comply with CSP
 */

document.addEventListener('DOMContentLoaded', () => {
    const stepContents = {
        shipping: document.getElementById('shipping-address-form'),
        escrow: document.getElementById('escrow-init'),
        payment: document.getElementById('payment-instructions'),
        confirmation: document.getElementById('payment-confirmed')
    };

    const stepper = new CheckoutStepper('checkout-stepper', {
        onStepChange: (newStepIndex) => {
            Object.values(stepContents).forEach(el => el.style.display = 'none');
            const newStepId = document.querySelector(`.checkout-stepper-step[data-step-index="${newStepIndex}"]`).dataset.stepId;
            if (stepContents[newStepId]) {
                stepContents[newStepId].style.display = 'block';
            }
        },
        beforeStepChange: (oldStepIndex, newStepIndex) => {
            // Add validation logic here if needed
            return true;
        }
    });

    window.checkoutStepper = stepper; // Make stepper globally accessible for other scripts

    // Initial step based on escrow status or default
    const escrowStatus = document.getElementById('escrow-status')?.value;
    const escrowExists = document.getElementById('escrow-id')?.value;

    if (escrowExists && escrowStatus === 'active') {
        stepper.goToStep(3); // Confirmation
    } else if (escrowExists) {
        stepper.goToStep(2); // Payment
    } else {
        stepper.goToStep(0); // Shipping Info
    }

    // Example of how to advance the stepper (e.g., after shipping form submission)
    // DISABLED: This interferes with the real form submission in checkout.js
    // The stepper will be advanced automatically when the order is created
    /*
    document.getElementById('submit-shipping-btn')?.addEventListener('click', (e) => {
        e.preventDefault();
        // Simulate successful submission
        setTimeout(() => {
            stepper.next();
        }, 500);
    });
    */
});
