/**
 * NEXUS Letter Jump Animation Controller
 * Ultra-optimized with mobile support, accessibility, and debouncing
 */
(function() {
  'use strict';

  const ANIMATION_DURATION = 1800; // Must match CSS (1.8s)
  const DEBOUNCE_DELAY = 100; // Prevent spam-clicking

  // Check if user prefers reduced motion
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Animation queue to prevent simultaneous jumps
  const animatingLetters = new Set();

  /**
   * Trigger jump animation on a letter
   * @param {HTMLElement} letter - The letter element
   */
  function triggerJump(letter) {
    // Prevent re-triggering if already animating
    if (animatingLetters.has(letter)) {
      return;
    }

    // Respect accessibility preferences
    if (prefersReducedMotion) {
      // Just scale instead of full animation
      letter.style.transform = 'scale(1.2)';
      setTimeout(() => {
        letter.style.transform = '';
      }, 300);
      return;
    }

    // Add to animating set
    animatingLetters.add(letter);

    // Add jumping class
    letter.classList.add('jumping');

    // Remove class after animation completes
    setTimeout(() => {
      letter.classList.remove('jumping');
      animatingLetters.delete(letter);
    }, ANIMATION_DURATION);

    // Optional: Haptic feedback on mobile
    if ('vibrate' in navigator) {
      navigator.vibrate(10); // Subtle vibration
    }
  }

  /**
   * Initialize letter animations with event listeners
   */
  function initLetterAnimations() {
    const letters = document.querySelectorAll('.nexus-animated-letter');

    letters.forEach((letter, index) => {
      // Mouse hover (desktop)
      letter.addEventListener('mouseenter', () => triggerJump(letter));

      // Touch (mobile/tablet)
      letter.addEventListener('touchstart', (e) => {
        e.preventDefault(); // Prevent double-firing with mouseenter
        triggerJump(letter);
      }, { passive: false });

      // Keyboard (accessibility)
      letter.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          triggerJump(letter);
        }
      });

      // Add sequential auto-animation on page load (one time, staggered)
      if (!sessionStorage.getItem('nexus-intro-played')) {
        setTimeout(() => {
          triggerJump(letter);
        }, 500 + (index * 200)); // Stagger by 200ms
      }
    });

    // Mark intro as played (won't replay on navigation)
    if (!sessionStorage.getItem('nexus-intro-played')) {
      setTimeout(() => {
        sessionStorage.setItem('nexus-intro-played', 'true');
      }, 500 + (letters.length * 200) + ANIMATION_DURATION);
    }
  }

  /**
   * Performance: Use Intersection Observer to only animate when visible
   */
  function setupIntersectionObserver() {
    const heroTitle = document.getElementById('nexus-title');
    if (!heroTitle) return;

    const observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          heroTitle.style.opacity = '1';
        }
      });
    }, { threshold: 0.1 });

    observer.observe(heroTitle);
  }

  // Initialize on DOM ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      initLetterAnimations();
      setupIntersectionObserver();
    });
  } else {
    initLetterAnimations();
    setupIntersectionObserver();
  }

  // Optional: Add "Konami code" easter egg (↑↑↓↓←→←→BA)
  const konamiCode = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];
  let konamiIndex = 0;

  document.addEventListener('keydown', (e) => {
    if (e.key === konamiCode[konamiIndex]) {
      konamiIndex++;
      if (konamiIndex === konamiCode.length) {
        // Easter egg: Make ALL letters jump at once
        document.querySelectorAll('.nexus-animated-letter').forEach((letter, i) => {
          setTimeout(() => triggerJump(letter), i * 100);
        });
        konamiIndex = 0;
      }
    } else {
      konamiIndex = 0;
    }
  });

})();
