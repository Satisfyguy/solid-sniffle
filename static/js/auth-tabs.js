// Auth page tabs functionality
(function() {
    'use strict';

    // Tab switching
    function switchTab(tab) {
        console.log('Switching to tab:', tab);

        // Update buttons
        document.querySelectorAll('.tab-trigger').forEach(btn => {
            btn.classList.remove('active');
        });
        const activeButton = document.getElementById('tab-' + tab);
        if (activeButton) {
            activeButton.classList.add('active');
            console.log('Button activated:', activeButton);
        }

        // Update content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });
        const activeContent = document.getElementById('content-' + tab);
        if (activeContent) {
            activeContent.classList.add('active');
            console.log('Content activated:', activeContent);
        }
    }

    // Toggle password visibility
    function togglePassword(inputId, button) {
        const input = document.getElementById(inputId);
        const eyeIcon = button.querySelector('.eye-icon');
        const eyeOffIcon = button.querySelector('.eye-off-icon');

        if (input.type === 'password') {
            input.type = 'text';
            eyeIcon.classList.add('hidden');
            eyeOffIcon.classList.remove('hidden');
        } else {
            input.type = 'password';
            eyeIcon.classList.remove('hidden');
            eyeOffIcon.classList.add('hidden');
        }
    }

    // Initialize when DOM is loaded
    document.addEventListener('DOMContentLoaded', function() {
        console.log('Auth page loaded');

        // Attach click handlers to tab buttons
        const loginTab = document.getElementById('tab-login');
        const signupTab = document.getElementById('tab-signup');

        if (loginTab) {
            loginTab.addEventListener('click', function() {
                switchTab('login');
            });
        }

        if (signupTab) {
            signupTab.addEventListener('click', function() {
                switchTab('signup');
            });
        }

        // Attach password toggle handlers
        document.querySelectorAll('.password-toggle').forEach(button => {
            button.addEventListener('click', function() {
                const inputId = this.closest('.relative').querySelector('input').id;
                togglePassword(inputId, this);
            });
        });

        // Password confirmation validation
        const signupForm = document.querySelector('#content-signup form');
        if (signupForm) {
            signupForm.addEventListener('submit', function(e) {
                const password = document.getElementById('signup-password').value;
                const confirm = document.getElementById('signup-confirm').value;

                if (password !== confirm) {
                    e.preventDefault();
                    alert('Les mots de passe ne correspondent pas!');
                    return false;
                }
            });
        }

        // Debug info
        console.log('Login button:', loginTab);
        console.log('Signup button:', signupTab);
        console.log('Login content:', document.getElementById('content-login'));
        console.log('Signup content:', document.getElementById('content-signup'));
    });
})();
