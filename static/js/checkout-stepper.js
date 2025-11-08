// static/js/checkout-stepper.js

/**
 * Checkout Stepper Widget
 *
 * Manages a multi-step checkout process with visual indicators.
 * Supports various step states (pending, active, completed, error, loading)
 * and programmatic navigation.
 */
class CheckoutStepper {
    constructor(containerId, options = {}) {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            console.error(`CheckoutStepper: Container with ID "${containerId}" not found.`);
            return;
        }

        this.steps = Array.from(this.container.querySelectorAll('.checkout-stepper-step'));
        this.currentStepIndex = 0; // Default to the first step
        this.options = {
            onStepChange: () => {},
            beforeStepChange: () => true,
            ...options
        };

        this._init();
    }

    _init() {
        this.steps.forEach((step, index) => {
            step.dataset.stepIndex = index;
            const indicator = step.querySelector('.checkout-stepper-step-indicator');
            if (indicator && !indicator.querySelector('.lucide')) {
                indicator.innerHTML = `<span>${index + 1}</span>`;
            }
            if (index === this.currentStepIndex) {
                this._setActiveStep(index);
            } else {
                step.classList.add('pending');
            }
        });
        this._updateConnectors();
        this._createCheckmarkIcon();
    }

    _createCheckmarkIcon() {
        // Pre-create a checkmark icon to avoid re-creating it for every step
        const checkmark = document.createElement('i');
        checkmark.dataset.lucide = 'check';
        checkmark.classList.add('lucide');
        if (typeof lucide !== 'undefined') {
            lucide.createIcons({ attrs: { class: 'lucide' }, icons: { check: lucide.icons.check } });
        }
        this.checkmarkIcon = checkmark;
    }

    _setActiveStep(index) {
        this.steps.forEach((step, i) => {
            step.classList.remove('active', 'completed', 'error', 'loading', 'pending');
            const indicator = step.querySelector('.checkout-stepper-step-indicator');
            if (indicator) {
                // Remove checkmark if present
                const existingCheckmark = indicator.querySelector('[data-lucide="check"]');
                if (existingCheckmark) {
                    existingCheckmark.remove();
                }
                // Restore number if it was hidden
                const numberSpan = indicator.querySelector('span');
                if (numberSpan) {
                    numberSpan.style.display = 'block';
                }
            }

            if (i < index) {
                step.classList.add('completed');
                if (indicator && !indicator.querySelector('[data-lucide="check"]')) {
                    const checkmarkClone = this.checkmarkIcon.cloneNode(true);
                    indicator.appendChild(checkmarkClone);
                    const numberSpan = indicator.querySelector('span');
                    if (numberSpan) {
                        numberSpan.style.display = 'none';
                    }
                }
            } else if (i === index) {
                step.classList.add('active');
            } else {
                step.classList.add('pending');
            }
        });
        this.currentStepIndex = index;
        this._updateConnectors();
        this.options.onStepChange(index);
    }

    _updateConnectors() {
        const totalSteps = this.steps.length;
        this.steps.forEach((step, i) => {
            let connector = step.querySelector('.checkout-stepper-connector');
            if (!connector && i < totalSteps - 1) {
                connector = document.createElement('div');
                connector.classList.add('checkout-stepper-connector');
                step.appendChild(connector);
            }

            if (connector) {
                if (i < this.currentStepIndex) {
                    connector.style.width = '100%';
                    connector.style.backgroundColor = 'var(--color-accent)';
                } else {
                    connector.style.width = '0%';
                    connector.style.backgroundColor = 'var(--color-border)';
                }
            }
        });
    }

    /**
     * Navigates to the next step.
     * @returns {boolean} True if navigation was successful, false otherwise.
     */
    next() {
        if (this.currentStepIndex < this.steps.length - 1) {
            if (this.options.beforeStepChange(this.currentStepIndex, this.currentStepIndex + 1)) {
                this._setActiveStep(this.currentStepIndex + 1);
                return true;
            }
        }
        return false;
    }

    /**
     * Navigates to the previous step.
     * @returns {boolean} True if navigation was successful, false otherwise.
     */
    previous() {
        if (this.currentStepIndex > 0) {
            if (this.options.beforeStepChange(this.currentStepIndex, this.currentStepIndex - 1)) {
                this._setActiveStep(this.currentStepIndex - 1);
                return true;
            }
        }
        return false;
    }

    /**
     * Navigates to a specific step.
     * @param {number} index - The 0-based index of the step to go to.
     * @returns {boolean} True if navigation was successful, false otherwise.
     */
    goToStep(index) {
        if (index >= 0 && index < this.steps.length) {
            if (this.options.beforeStepChange(this.currentStepIndex, index)) {
                this._setActiveStep(index);
                return true;
            }
        }
        return false;
    }

    /**
     * Sets a step to an error state.
     * @param {number} index - The 0-based index of the step.
     */
    setError(index) {
        if (index >= 0 && index < this.steps.length) {
            this.steps[index].classList.add('error');
            this.steps[index].classList.remove('active', 'completed', 'loading', 'pending');
        }
    }

    /**
     * Clears the error state for a step.
     * @param {number} index - The 0-based index of the step.
     */
    clearError(index) {
        if (index >= 0 && index < this.steps.length) {
            this.steps[index].classList.remove('error');
            // Re-apply correct state based on current step
            if (index < this.currentStepIndex) {
                this.steps[index].classList.add('completed');
            } else if (index === this.currentStepIndex) {
                this.steps[index].classList.add('active');
            } else {
                this.steps[index].classList.add('pending');
            }
        }
    }

    /**
     * Sets a step to a loading state.
     * @param {number} index - The 0-based index of the step.
     */
    setLoading(index) {
        if (index >= 0 && index < this.steps.length) {
            this.steps[index].classList.add('loading');
            this.steps[index].classList.remove('active', 'completed', 'error', 'pending');
        }
    }

    /**
     * Resets the stepper to its initial state (first step, no errors/completed).
     */
    reset() {
        this._setActiveStep(0);
        this.steps.forEach(step => {
            step.classList.remove('error', 'loading');
        });
    }
}

