document.addEventListener('DOMContentLoaded', function() {
    // Initialize Lucide icons (only if not already done by base.js)
    if (typeof lucide !== 'undefined' && !window.lucideInitialized) {
        lucide.createIcons();
        window.lucideInitialized = true;
    }

    const tabsContainer = document.querySelector('.tabs-container');
    if (tabsContainer) {
        tabsContainer.addEventListener('change', function(event) {
            if (event.target.classList.contains('tab-radio')) {
                const tabId = event.target.id.replace('tab-', 'content-');
                document.querySelectorAll('.tab-content').forEach(content => {
                    content.style.display = 'none';
                });
                const activeContent = document.getElementById(tabId);
                if (activeContent) {
                    activeContent.style.display = 'block';
                }
            }
        });

        // Set initial active tab content
        const initialActiveTab = tabsContainer.querySelector('.tab-radio:checked');
        if (initialActiveTab) {
            const tabId = initialActiveTab.id.replace('tab-', 'content-');
            const activeContent = document.getElementById(tabId);
            if (activeContent) {
                activeContent.style.display = 'block';
            }
        }
    }
});