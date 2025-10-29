/* ==========================================
   NEXUS GRID HEADER - THEME TOGGLE
   ========================================== */

(function() {
  'use strict';

  function initToggle() {
    const toggle = document.getElementById('nexus-toggle-dot');
    if (!toggle) {
      console.warn('⚠️ Toggle button not found');
      return;
    }

    // Restore saved theme
    const savedTheme = localStorage.getItem('nexus-theme') || 'dark';
    if (savedTheme === 'light') {
      document.body.classList.add('light-mode');
      toggle.classList.add('light');
    }

    // Toggle on click
    toggle.addEventListener('click', function() {
      const isLight = document.body.classList.toggle('light-mode');
      toggle.classList.toggle('light');
      localStorage.setItem('nexus-theme', isLight ? 'light' : 'dark');
      console.log('🎨 Theme switched to:', isLight ? 'light' : 'dark');
    });

    console.log('✅ Theme toggle initialized');
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initToggle);
  } else {
    initToggle();
  }
})();
