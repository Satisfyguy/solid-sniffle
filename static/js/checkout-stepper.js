/**
 * Checkout Stepper Controller
 *
 * Manages multi-step checkout flow with visual progress indication.
 * Production-grade with error handling and accessibility support.
 *
 * @module checkout-stepper
 */

class CheckoutStepper {
    /**
     * Create a checkout stepper controller
     * @param {string} containerSelector - CSS selector for stepper container
     * @param {Object} options - Configuration options
     * @param {Function} [options.onStepChange] - Callback when step changes
     * @param {Function} [options.beforeStepChange] - Callback before step change (can prevent)
     */
    constructor(containerSelector, options = {}) {
        this.container = document.querySelector(containerSelector);

        if (!this.container) {
            console.error(`Checkout Stepper: Container not found (${containerSelector})`);
            return;
        }

        this.options = {
            onStepChange: options.onStepChange || null,
            beforeStepChange: options.beforeStepChange || null,
        };

        this.steps = Array.from(this.container.querySelectorAll('.checkout-step'));
        this.connectors = Array.from(this.container.querySelectorAll('.checkout-step-connector'));
        this.currentStepIndex = 0;

        this.init();
    }

    /**
     * Initialize stepper (find active step, set up initial state)
     */
    init() {
        // Find initially active step
        const activeStep = this.container.querySelector('.checkout-step.active');
        if (activeStep) {
            this.currentStepIndex = this.steps.indexOf(activeStep);
        }

        // Set initial states
        this.updateStates();

        console.log('✅ Checkout Stepper initialized:', {
            steps: this.steps.length,
            currentStep: this.currentStepIndex + 1
        });
    }

    /**
     * Update visual states of all steps and connectors
     */
    updateStates() {
        this.steps.forEach((step, index) => {
            const connector = this.connectors[index];

            // Remove all state classes
            step.classList.remove('pending', 'active', 'completed', 'error', 'loading');
            if (connector) {
                connector.classList.remove('pending', 'active', 'completed');
            }

            // Add appropriate state class
            if (index < this.currentStepIndex) {
                // Previous steps are completed
                step.classList.add('completed');
                if (connector) connector.classList.add('completed');

                // Show checkmark icon instead of number
                this.showCheckmark(step);

            } else if (index === this.currentStepIndex) {
                // Current step is active
                step.classList.add('active');
                if (connector) connector.classList.add('active');

            } else {
                // Future steps are pending
                step.classList.add('pending');
                if (connector) connector.classList.add('pending');
            }
        });
    }

    /**
     * Show checkmark icon for completed step
     * @param {HTMLElement} step - Step element
     */
    showCheckmark(step) {
        const indicator = step.querySelector('.checkout-step-indicator');
        if (!indicator) return;

        // Check if checkmark already exists
        let checkIcon = indicator.querySelector('.checkout-step-icon-check');

        if (!checkIcon) {
            // Create checkmark icon
            checkIcon = document.createElement('i');
            checkIcon.setAttribute('data-lucide', 'check');
            checkIcon.className = 'checkout-step-icon-check';
            indicator.appendChild(checkIcon);

            // Initialize Lucide icon
            if (typeof lucide !== 'undefined') {
                lucide.createIcons();
            }
        }

        // Hide number, show checkmark
        const numberSpan = indicator.querySelector('.checkout-step-number');
        if (numberSpan) {
            numberSpan.style.display = 'none';
        }
    }

    /**
     * Go to a specific step
     * @param {number} stepIndex - Zero-based step index
     * @param {boolean} [force=false] - Force transition without validation
     * @returns {boolean} True if transition succeeded
     */
    goToStep(stepIndex, force = false) {
        if (stepIndex < 0 || stepIndex >= this.steps.length) {
            console.error(`Invalid step index: ${stepIndex}`);
            return false;
        }

        if (stepIndex === this.currentStepIndex) {
            return true; // Already on this step
        }

        // Call beforeStepChange hook
        if (!force && this.options.beforeStepChange) {
            const canProceed = this.options.beforeStepChange({
                from: this.currentStepIndex,
                to: stepIndex,
                stepper: this
            });

            if (canProceed === false) {
                console.log('Step change prevented by beforeStepChange hook');
                return false;
            }
        }

        // Update current step
        const oldStepIndex = this.currentStepIndex;
        this.currentStepIndex = stepIndex;

        // Update visual states
        this.updateStates();

        // Scroll to active step section if specified
        const targetSection = this.steps[stepIndex].dataset.targetSection;
        if (targetSection) {
            const section = document.getElementById(targetSection);
            if (section) {
                section.scrollIntoView({ behavior: 'smooth', block: 'start' });
            }
        }

        // Call onStepChange hook
        if (this.options.onStepChange) {
            this.options.onStepChange({
                from: oldStepIndex,
                to: stepIndex,
                stepper: this
            });
        }

        console.log(`Step changed: ${oldStepIndex + 1} → ${stepIndex + 1}`);
        return true;
    }

    /**
     * Go to next step
     * @returns {boolean} True if transition succeeded
     */
    next() {
        if (this.currentStepIndex < this.steps.length - 1) {
            return this.goToStep(this.currentStepIndex + 1);
        }
        return false;
    }

    /**
     * Go to previous step
     * @returns {boolean} True if transition succeeded
     */
    previous() {
        if (this.currentStepIndex > 0) {
            return this.goToStep(this.currentStepIndex - 1);
        }
        return false;
    }

    /**
     * Mark current step as completed and move to next
     * @returns {boolean} True if transition succeeded
     */
    complete() {
        return this.next();
    }

    /**
     * Set error state for current step
     * @param {string} [message] - Optional error message
     */
    setError(message) {
        const currentStep = this.steps[this.currentStepIndex];
        if (currentStep) {
            currentStep.classList.add('error');

            if (message) {
                const label = currentStep.querySelector('.checkout-step-label');
                if (label) {
                    label.dataset.originalText = label.dataset.originalText || label.textContent;
                    label.textContent = message;
                }
            }
        }
    }

    /**
     * Clear error state from current step
     */
    clearError() {
        const currentStep = this.steps[this.currentStepIndex];
        if (currentStep) {
            currentStep.classList.remove('error');

            const label = currentStep.querySelector('.checkout-step-label');
            if (label && label.dataset.originalText) {
                label.textContent = label.dataset.originalText;
                delete label.dataset.originalText;
            }
        }
    }

    /**
     * Set loading state for current step
     */
    setLoading() {
        const currentStep = this.steps[this.currentStepIndex];
        if (currentStep) {
            currentStep.classList.add('loading');
        }
    }

    /**
     * Clear loading state from current step
     */
    clearLoading() {
        const currentStep = this.steps[this.currentStepIndex];
        if (currentStep) {
            currentStep.classList.remove('loading');
        }
    }

    /**
     * Get current step index (1-based for display)
     * @returns {number} Current step number
     */
    getCurrentStep() {
        return this.currentStepIndex + 1;
    }

    /**
     * Get total number of steps
     * @returns {number} Total steps
     */
    getTotalSteps() {
        return this.steps.length;
    }

    /**
     * Check if on first step
     * @returns {boolean}
     */
    isFirstStep() {
        return this.currentStepIndex === 0;
    }

    /**
     * Check if on last step
     * @returns {boolean}
     */
    isLastStep() {
        return this.currentStepIndex === this.steps.length - 1;
    }

    /**
     * Reset stepper to first step
     */
    reset() {
        this.goToStep(0, true);
        this.steps.forEach(step => {
            step.classList.remove('error', 'loading');
        });
    }
}

/**
 * Create stepper HTML structure dynamically
 * @param {string} containerId - ID of container element
 * @param {Array<Object>} steps - Array of step objects {label, icon}
 * @returns {CheckoutStepper|null} Initialized stepper instance
 */
function createCheckoutStepper(containerId, steps) {
    const container = document.getElementById(containerId);

    if (!container) {
        console.error(`Checkout Stepper: Container not found (${containerId})`);
        return null;
    }

    // Build HTML
    let html = '<div class="checkout-stepper">';

    steps.forEach((step, index) => {
        // Add step
        html += `
            <div class="checkout-step ${index === 0 ? 'active' : 'pending'}" data-step="${index + 1}">
                <div class="checkout-step-indicator">
                    <span class="checkout-step-number">${index + 1}</span>
                </div>
                <span class="checkout-step-label">${step.label}</span>
            </div>
        `;

        // Add connector (except after last step)
        if (index < steps.length - 1) {
            html += '<div class="checkout-step-connector"></div>';
        }
    });

    html += '</div>';

    container.innerHTML = html;

    // Initialize Lucide icons
    if (typeof lucide !== 'undefined') {
        lucide.createIcons();
    }

    // Return initialized stepper
    return new CheckoutStepper(`#${containerId} .checkout-stepper`);
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { CheckoutStepper, createCheckoutStepper };
}
