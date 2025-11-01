document.addEventListener('DOMContentLoaded', function() {
    // Tab switching logic
    const tabsContainer = document.getElementById('auth-tabs');
    if (tabsContainer) {
        const tabs = tabsContainer.querySelectorAll('.tabs-trigger');
        const contents = tabsContainer.querySelectorAll('.tabs-content');

        tabs.forEach(tab => {
            tab.addEventListener('click', () => {
                const targetId = tab.getAttribute('data-tab');

                tabs.forEach(t => t.setAttribute('data-state', ''));
                tab.setAttribute('data-state', 'active');

                contents.forEach(c => c.setAttribute('data-state', ''));
                document.getElementById(targetId).setAttribute('data-state', 'active');
            });
        });
    }

    // Password visibility toggle
    const passwordToggles = document.querySelectorAll('.password-toggle-btn');
    passwordToggles.forEach(button => {
        button.addEventListener('click', () => {
            const inputId = button.getAttribute('data-input-id');
            const input = document.getElementById(inputId);
            if (!input) return;

            const eyeOpen = button.querySelector('.eye-open');
            const eyeClosed = button.querySelector('.eye-closed');

            if (input.type === 'password') {
                input.type = 'text';
                if(eyeOpen) eyeOpen.style.display = 'none';
                if(eyeClosed) eyeClosed.style.display = 'block';
            } else {
                input.type = 'password';
                if(eyeOpen) eyeOpen.style.display = 'block';
                if(eyeClosed) eyeClosed.style.display = 'none';
            }
        });
    });
});