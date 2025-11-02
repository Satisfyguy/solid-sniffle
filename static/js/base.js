// Base template JavaScript
// Handles HTMX config, dark mode, and user menu

// HTMX configuration for Tor-safe navigation
document.body.addEventListener('htmx:configRequest', function(evt) {
    evt.detail.headers['X-Requested-With'] = 'XMLHttpRequest';
});

// Dark mode toggle (stored in localStorage)
const savedTheme = localStorage.getItem('theme') || 'light';
if (savedTheme === 'dark') {
    document.body.classList.add('dark-mode');
}

// User menu toggle
document.addEventListener('DOMContentLoaded', function() {
    const userMenuBtn = document.getElementById('user-menu-btn');
    const userMenuDropdown = document.getElementById('user-menu-dropdown');

    if (userMenuBtn && userMenuDropdown) {
        userMenuBtn.addEventListener('click', function(e) {
            e.stopPropagation();
            const isOpen = userMenuDropdown.style.display === 'block';
            userMenuDropdown.style.display = isOpen ? 'none' : 'block';
        });

        // Close menu when clicking outside
        document.addEventListener('click', function(e) {
            if (!e.target.closest('.user-menu-container')) {
                userMenuDropdown.style.display = 'none';
            }
        });

        // Close menu on escape key
        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape') {
                userMenuDropdown.style.display = 'none';
            }
        });
    }

    // Initialize Lucide icons (if available and not already initialized)
    if (typeof lucide !== 'undefined' && !window.lucideInitialized) {
        lucide.createIcons();
        window.lucideInitialized = true;
    }
});
