/* ==========================================
   NEXUS - HEADER TYPEWRITER ANIMATION
   Cycling phrases under NX logo with erase/rewrite effect
   ========================================== */

(function() {
  'use strict';

  class HeaderTypewriter {
    constructor() {
      // Configuration
      this.phrases = [
        'SECURE • ANONYMOUS • UNTRACEABLE',
        '2-OF-3 MULTISIG ESCROW',
        'NO LOGS • NO TRACKING • NO TRACE',
        'TOR HIDDEN SERVICE ONLY',
        'NON-CUSTODIAL MARKETPLACE'
      ];

      this.config = {
        typeSpeed: 80,        // ms per character when typing
        deleteSpeed: 40,      // ms per character when deleting
        pauseAfterType: 2500, // ms to pause after fully typed
        pauseAfterDelete: 500 // ms to pause after fully deleted
      };

      // State
      this.currentPhraseIndex = 0;
      this.currentText = '';
      this.isDeleting = false;
      this.isWaiting = false;

      // DOM elements
      this.textElement = document.getElementById('nexus-typewriter-text');
      this.cursorElement = document.querySelector('.nexus-header-typewriter-cursor');

      // Initialize
      if (this.textElement) {
        this.start();
      }
    }

    start() {
      // Start the animation loop
      this.animate();
    }

    animate() {
      const currentPhrase = this.phrases[this.currentPhraseIndex];

      if (this.isWaiting) {
        // Waiting period - do nothing, just schedule next frame
        return;
      }

      if (!this.isDeleting) {
        // TYPING MODE: Add characters one by one
        if (this.currentText.length < currentPhrase.length) {
          this.currentText = currentPhrase.substring(0, this.currentText.length + 1);
          this.updateText();

          setTimeout(() => this.animate(), this.config.typeSpeed);
        } else {
          // Finished typing - pause then start deleting
          this.isWaiting = true;
          setTimeout(() => {
            this.isWaiting = false;
            this.isDeleting = true;
            this.animate();
          }, this.config.pauseAfterType);
        }
      } else {
        // DELETING MODE: Remove characters one by one
        if (this.currentText.length > 0) {
          this.currentText = currentPhrase.substring(0, this.currentText.length - 1);
          this.updateText();

          setTimeout(() => this.animate(), this.config.deleteSpeed);
        } else {
          // Finished deleting - pause then move to next phrase
          this.isWaiting = true;
          this.isDeleting = false;
          this.currentPhraseIndex = (this.currentPhraseIndex + 1) % this.phrases.length;

          setTimeout(() => {
            this.isWaiting = false;
            this.animate();
          }, this.config.pauseAfterDelete);
        }
      }
    }

    updateText() {
      if (this.textElement) {
        this.textElement.textContent = this.currentText;
      }
    }
  }

  // Initialize when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      new HeaderTypewriter();
    });
  } else {
    // DOMContentLoaded already fired
    new HeaderTypewriter();
  }
})();
