/* ==========================================
   NEXUS BRUTALIST - THEME TOGGLE
   Light mode = default, Dark mode = toggle
   Circle texture swaps via CSS (brutalist-textures.css)
   ========================================== */

(function() {
  'use strict';

  function initToggle() {
    const toggle = document.getElementById('nexus-toggle-dot');
    if (!toggle) {
      console.warn('⚠️ Toggle button not found');
      return;
    }

    // Restore saved theme (default: light)
    const savedTheme = localStorage.getItem('nexus-theme') || 'light';
    if (savedTheme === 'dark') {
      document.body.classList.add('dark-mode');
    }

    // Toggle on click
    toggle.addEventListener('click', function() {
      const isDark = document.body.classList.toggle('dark-mode');
      localStorage.setItem('nexus-theme', isDark ? 'dark' : 'light');
      console.log('🎨 Theme switched to:', isDark ? 'dark' : 'light');
    });

    console.log('✅ Brutalist theme toggle initialized (default: light)');
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initToggle);
  } else {
    initToggle();
  }
})();
