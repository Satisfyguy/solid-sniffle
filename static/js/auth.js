// Authentication page JavaScript
// Handles Login/SignUp tabs, Buyer/Seller role selection, and password toggle

document.addEventListener('DOMContentLoaded', function() {
    // Auth Tabs (Login/SignUp)
    const loginBtn = document.getElementById('tab-login');
    const signupBtn = document.getElementById('tab-signup');
    const loginContent = document.getElementById('content-login');
    const signupContent = document.getElementById('content-signup');

    if (loginBtn && signupBtn && loginContent && signupContent) {
        loginBtn.addEventListener('click', function() {
            loginBtn.classList.add('active');
            signupBtn.classList.remove('active');
            loginContent.style.display = 'block';
            signupContent.style.display = 'none';
        });

        signupBtn.addEventListener('click', function() {
            signupBtn.classList.add('active');
            loginBtn.classList.remove('active');
            signupContent.style.display = 'block';
            loginContent.style.display = 'none';
        });
    }

    // Role Tabs (Buyer/Seller)
    const buyerBtn = document.getElementById('role-buyer');
    const sellerBtn = document.getElementById('role-seller');
    const roleInput = document.getElementById('role-input');
    const walletField = document.getElementById('wallet-field');
    const walletInput = document.getElementById('wallet-address');

    if (buyerBtn && sellerBtn && roleInput && walletField && walletInput) {
        buyerBtn.addEventListener('click', function() {
            buyerBtn.classList.add('active');
            sellerBtn.classList.remove('active');
            roleInput.value = 'buyer';
            walletField.style.display = 'none';
            walletInput.disabled = true;
        });

        sellerBtn.addEventListener('click', function() {
            sellerBtn.classList.add('active');
            buyerBtn.classList.remove('active');
            roleInput.value = 'vendor';
            walletField.style.display = 'block';
            walletInput.disabled = false;
        });
    }

    // Password Toggle
    const passwordToggles = document.querySelectorAll('.password-toggle');
    passwordToggles.forEach(function(toggle) {
        toggle.addEventListener('click', function() {
            const targetId = this.getAttribute('data-target');
            const input = document.getElementById(targetId);
            const icon = this.querySelector('.eye-icon');

            if (input && icon) {
                if (input.type === 'password') {
                    input.type = 'text';
                    icon.innerHTML = '<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/>';
                } else {
                    input.type = 'password';
                    icon.innerHTML = '<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>';
                }
            }
        });
    });
});
