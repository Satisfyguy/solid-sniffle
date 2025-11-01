// Simple tab functionality for the profile page
document.addEventListener('DOMContentLoaded', function() {
    const tabsContainer = document.getElementById('profile-tabs');
    if (tabsContainer) {
        const triggers = tabsContainer.querySelectorAll('.tabs-trigger');
        const contents = tabsContainer.querySelectorAll('.tabs-content');

        triggers.forEach(trigger => {
            trigger.addEventListener('click', function() {
                // Deactivate all triggers and contents
                triggers.forEach(t => t.classList.remove('active'));
                contents.forEach(c => c.classList.remove('active'));

                // Activate clicked trigger and corresponding content
                this.classList.add('active');
                const tabId = this.getAttribute('data-tab');
                const content = document.getElementById(tabId);
                if (content) {
                    content.classList.add('active');
                }
            });
        });
    }
});
