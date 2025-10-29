/**
 * NEXUS User Menu Dropdown
 *
 * Handles the user menu dropdown in the header
 * FIX: Prevents event listener accumulation on HTMX navigation
 */

(function() {
  'use strict';

  // Store handlers to prevent accumulation
  let clickOutsideHandler = null;
  let escapeKeyHandler = null;
  let triggerClickHandler = null;
  let menuItemHandlers = [];

  // Wait for DOM to be ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  function cleanup() {
    // Remove global listeners if they exist
    if (clickOutsideHandler) {
      document.removeEventListener('click', clickOutsideHandler);
      clickOutsideHandler = null;
    }
    if (escapeKeyHandler) {
      document.removeEventListener('keydown', escapeKeyHandler);
      escapeKeyHandler = null;
    }
    // Menu item handlers will be removed when elements are removed from DOM
    menuItemHandlers = [];
  }

  function init() {
    // Cleanup previous listeners
    cleanup();

    const trigger = document.getElementById('nexus-user-menu-trigger');
    const dropdown = document.getElementById('nexus-user-dropdown');

    if (!trigger || !dropdown) {
      // User not logged in, no menu to handle
      return;
    }

    // Calculate dropdown position dynamically
    function positionDropdown() {
      const triggerRect = trigger.getBoundingClientRect();
      dropdown.style.left = `${triggerRect.left + triggerRect.width / 2}px`;
    }

    // Toggle dropdown on trigger click
    triggerClickHandler = function(e) {
      e.stopPropagation();
      positionDropdown();
      dropdown.classList.toggle('active');
    };
    trigger.addEventListener('click', triggerClickHandler);

    // Close dropdown when clicking outside
    clickOutsideHandler = function(e) {
      if (!dropdown.contains(e.target) && e.target !== trigger) {
        dropdown.classList.remove('active');
      }
    };
    document.addEventListener('click', clickOutsideHandler);

    // Close dropdown when clicking on a menu item
    const menuItems = dropdown.querySelectorAll('.nexus-dropdown-item');
    menuItems.forEach(item => {
      const handler = function() {
        dropdown.classList.remove('active');
      };
      item.addEventListener('click', handler);
      menuItemHandlers.push({ item, handler });
    });

    // Close dropdown on Escape key
    escapeKeyHandler = function(e) {
      if (e.key === 'Escape' && dropdown.classList.contains('active')) {
        dropdown.classList.remove('active');
      }
    };
    document.addEventListener('keydown', escapeKeyHandler);
  }

  // Cleanup on page unload
  window.addEventListener('beforeunload', cleanup);

  // Cleanup on HTMX navigation
  document.body.addEventListener('htmx:beforeSwap', cleanup);
})();
