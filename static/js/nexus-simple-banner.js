/* ==========================================
   NEXUS SIMPLE HEADER - SCROLLING HASH BANNER
   Generates and displays fictional Monero transaction hashes
   ========================================== */

(function() {
  'use strict';

  class SimpleHashBanner {
    constructor() {
      this.bannerContent = document.getElementById('nexus-simple-banner-content');

      if (!this.bannerContent) {
        return;
      }

      // Generate fictional transaction hashes
      this.generateHashes();

      // Observe theme changes (optional)
      this.observeTheme();
    }

    generateRandomHash() {
      // Generate a realistic-looking Monero transaction hash (64 hex characters)
      const chars = '0123456789abcdef';
      let hash = '';
      for (let i = 0; i < 64; i++) {
        hash += chars[Math.floor(Math.random() * chars.length)];
      }
      return hash;
    }

    shortenHash(hash) {
      // Display shortened version: first 6 + ... + last 6
      return `${hash.substring(0, 6)}...${hash.substring(hash.length - 6)}`;
    }

    generateHashes() {
      // Generate multiple hashes for seamless loop
      const hashCount = 25; // Number of hashes to display
      const hashes = [];

      for (let i = 0; i < hashCount; i++) {
        const fullHash = this.generateRandomHash();
        const shortHash = this.shortenHash(fullHash);
        hashes.push(`TX: ${shortHash}`);
      }

      // Create HTML content - duplicate for seamless infinite scroll
      const hashesText = hashes.join(' • ');
      const duplicatedText = `${hashesText} • ${hashesText}`;

      // Create span for seamless infinite scroll
      this.bannerContent.innerHTML = `
        <span class="nexus-simple-banner-text">${duplicatedText}</span>
      `;
    }

    observeTheme() {
      // Watch for theme changes (banner color adjusts via CSS automatically)
      const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
          if (mutation.attributeName === 'class') {
            // Theme changed - banner color will adjust via CSS automatically
            // No need to regenerate hashes
          }
        });
      });

      observer.observe(document.body, {
        attributes: true,
        attributeFilter: ['class']
      });
    }
  }

  // Initialize when DOM is ready
  function initBanner() {
    new SimpleHashBanner();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initBanner);
  } else {
    initBanner();
  }
})();
