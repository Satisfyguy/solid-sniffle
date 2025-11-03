// Checkout Page - Step Navigation

function goToStep(step) {
    // Hide all steps
    document.getElementById('step-1').style.display = 'none';
    document.getElementById('step-2').style.display = 'none';
    document.getElementById('step-3').style.display = 'none';

    // Show current step
    document.getElementById('step-' + step).style.display = 'block';

    // Update step indicators
    updateStepIndicators(step);
}

function updateStepIndicators(currentStep) {
    // Step 2
    const step2Circle = document.getElementById('step-2-circle');
    const step2Line = document.getElementById('step-2-line');

    if (currentStep >= 2) {
        step2Circle.classList.add('active');
        if (currentStep > 2) {
            step2Circle.innerHTML = '<i data-lucide="check-circle-2"></i>';
            if (typeof lucide !== 'undefined') {
                lucide.createIcons();
            }
        } else {
            step2Circle.textContent = '2';
        }
    } else {
        step2Circle.classList.remove('active');
        step2Circle.textContent = '2';
    }

    if (currentStep > 2) {
        step2Line.classList.add('active');
    } else {
        step2Line.classList.remove('active');
    }

    // Step 3
    const step3Circle = document.getElementById('step-3-circle');

    if (currentStep >= 3) {
        step3Circle.classList.add('active');
        step3Circle.innerHTML = '<i data-lucide="check-circle-2"></i>';
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    } else {
        step3Circle.classList.remove('active');
        step3Circle.textContent = '3';
    }
}

// Initialize event listeners when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    // Step 1 - Next button
    const step1Next = document.getElementById('step-1-next');
    if (step1Next) {
        step1Next.addEventListener('click', function() {
            goToStep(2);
        });
    }

    // Step 2 - Back button
    const step2Back = document.getElementById('step-2-back');
    if (step2Back) {
        step2Back.addEventListener('click', function() {
            goToStep(1);
        });
    }

    // Step 2 - Next button
    const step2Next = document.getElementById('step-2-next');
    if (step2Next) {
        step2Next.addEventListener('click', function() {
            goToStep(3);
        });
    }

    // Step 3 - Back button
    const step3Back = document.getElementById('step-3-back');
    if (step3Back) {
        step3Back.addEventListener('click', function() {
            goToStep(2);
        });
    }

    // Step 3 - Confirm button
    const step3Confirm = document.getElementById('step-3-confirm');
    if (step3Confirm) {
        step3Confirm.addEventListener('click', function() {
            // TODO: Implement actual checkout logic
            alert('Commande confirmée! (En développement)');
        });
    }
});
