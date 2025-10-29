/**
 * NEXUS User Menu Dropdown
 *
 * Handles the user menu dropdown in the header
 */

(function() {
  'use strict';

  // Wait for DOM to be ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  function init() {
    const trigger = document.getElementById('nexus-user-menu-trigger');
    const dropdown = document.getElementById('nexus-user-dropdown');

    if (!trigger || !dropdown) {
      // User not logged in, no menu to handle
      console.log('NEXUS User Menu: Not logged in or elements not found');
      return;
    }

    console.log('NEXUS User Menu: Initialized', { trigger, dropdown });

    // Calculate dropdown position dynamically
    function positionDropdown() {
      const triggerRect = trigger.getBoundingClientRect();
      dropdown.style.left = `${triggerRect.left + triggerRect.width / 2}px`;
    }

    // Toggle dropdown on trigger click
    trigger.addEventListener('click', function(e) {
      e.stopPropagation();
      console.log('NEXUS User Menu: Trigger clicked');

      positionDropdown(); // Position avant d'afficher
      dropdown.classList.toggle('active');

      console.log('NEXUS User Menu: Active class toggled', dropdown.classList.contains('active'));
    });

    // Close dropdown when clicking outside
    document.addEventListener('click', function(e) {
      if (!dropdown.contains(e.target) && e.target !== trigger) {
        dropdown.classList.remove('active');
      }
    });

    // Close dropdown when clicking on a menu item (navigation will happen)
    const menuItems = dropdown.querySelectorAll('.nexus-dropdown-item');
    menuItems.forEach(item => {
      item.addEventListener('click', function() {
        dropdown.classList.remove('active');
      });
    });

    // Close dropdown on Escape key
    document.addEventListener('keydown', function(e) {
      if (e.key === 'Escape' && dropdown.classList.contains('active')) {
        dropdown.classList.remove('active');
      }
    });
  }
})();
