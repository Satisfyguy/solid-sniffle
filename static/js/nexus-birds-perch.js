/**
 * NEXUS Birds Perching System
 *
 * Aléatoirement, un grand oiseau vient se poser sur le haut d'une vignette produit
 * comme un perchoir, puis s'envole après quelques secondes
 */

class BirdsPerchingSystem {
  constructor() {
    this.productCards = [];
    this.perchingBird = null;
    this.isActive = false;
    this.observer = null;
    this.visibleCards = new Set();
    this.birdPaths = [];

    // Configuration
    this.config = {
      svgPath: '/static/birds.svg',
      minWaitTime: 5000,      // Minimum 5s entre chaque pose
      maxWaitTime: 15000,     // Maximum 15s entre chaque pose
      perchDuration: 4000,    // L'oiseau reste 4s sur le perchoir
      birdSize: 200,          // Taille de l'oiseau percheur (plus grand)
      isMobile: window.innerWidth < 768,
      prefersReducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches
    };
  }

  /**
   * Initialize the perching system
   */
  async init() {
    if (this.config.prefersReducedMotion) {
      console.info('NEXUS Perching: Animation disabled (prefers-reduced-motion)');
      return;
    }

    try {
      await this.loadBirdPaths();
      this.findProductCards();
      this.setupIntersectionObserver();
      this.startPerchingCycle();

      console.info('NEXUS Perching: System initialized');
    } catch (error) {
      console.error('NEXUS Perching: Initialization failed', error);
    }
  }

  /**
   * Load bird paths from SVG
   */
  async loadBirdPaths() {
    const response = await fetch(this.config.svgPath);
    if (!response.ok) {
      throw new Error(`Failed to fetch ${this.config.svgPath}`);
    }

    const svgText = await response.text();
    const parser = new DOMParser();
    const svgDoc = parser.parseFromString(svgText, 'image/svg+xml');
    const originalSvg = svgDoc.querySelector('svg');

    if (!originalSvg) {
      throw new Error('No SVG element found');
    }

    const paths = originalSvg.querySelectorAll('path');
    const viewBox = originalSvg.getAttribute('viewBox');
    const [vbX, vbY, vbWidth, vbHeight] = viewBox ? viewBox.split(' ').map(Number) : [0, 0, 1000, 1000];

    // Store all bird paths for random selection
    paths.forEach(path => {
      this.birdPaths.push({
        pathData: path.getAttribute('d'),
        viewBoxWidth: vbWidth,
        viewBoxHeight: vbHeight
      });
    });

    console.info(`NEXUS Perching: Loaded ${this.birdPaths.length} bird variations`);
  }

  /**
   * Find all product cards
   */
  findProductCards() {
    this.productCards = Array.from(document.querySelectorAll('.nexus-product-card'));
    console.info(`NEXUS Perching: Found ${this.productCards.length} product cards`);
  }

  /**
   * Setup Intersection Observer to track visible cards
   */
  setupIntersectionObserver() {
    const options = {
      root: null,
      rootMargin: '0px',
      threshold: 0.5 // Card must be at least 50% visible
    };

    this.observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          this.visibleCards.add(entry.target);
        } else {
          this.visibleCards.delete(entry.target);
        }
      });
    }, options);

    // Observe all product cards
    this.productCards.forEach(card => {
      this.observer.observe(card);
    });
  }

  /**
   * Start the perching cycle
   */
  startPerchingCycle() {
    if (this.isActive) return;
    this.isActive = true;

    const scheduleNextPerch = () => {
      if (!this.isActive) return;

      const waitTime = this.randomBetween(
        this.config.minWaitTime,
        this.config.maxWaitTime
      );

      setTimeout(() => {
        this.perchOnRandomCard();
        scheduleNextPerch();
      }, waitTime);
    };

    scheduleNextPerch();
  }

  /**
   * Make a bird perch on a random visible card
   */
  async perchOnRandomCard() {
    // Get visible cards
    const visibleCardsArray = Array.from(this.visibleCards);

    if (visibleCardsArray.length === 0) {
      console.log('NEXUS Perching: No visible cards, skipping');
      return;
    }

    // Pick a random visible card
    const targetCard = visibleCardsArray[Math.floor(Math.random() * visibleCardsArray.length)];

    // Pick a random bird
    const randomBird = this.birdPaths[Math.floor(Math.random() * this.birdPaths.length)];

    // Create the perching bird
    await this.createAndAnimatePerchingBird(targetCard, randomBird);
  }

  /**
   * Create and animate a bird that perches on a card
   */
  async createAndAnimatePerchingBird(targetCard, birdData) {
    // Prevent multiple birds at once
    if (this.perchingBird) return;

    // Create bird element
    const bird = document.createElement('div');
    bird.className = 'nexus-perching-bird';

    // Create SVG
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('viewBox', `0 0 ${birdData.viewBoxWidth} ${birdData.viewBoxHeight}`);
    svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
    svg.style.width = '100%';
    svg.style.height = '100%';

    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute('d', birdData.pathData);
    path.setAttribute('fill', '#000000');
    path.setAttribute('stroke', 'none');

    svg.appendChild(path);
    bird.appendChild(svg);

    // Style the bird
    bird.style.position = 'fixed';
    bird.style.pointerEvents = 'none';
    bird.style.zIndex = '1000'; // Au-dessus de tout
    bird.style.width = `${this.config.birdSize}px`;
    bird.style.height = `${this.config.birdSize * 0.67}px`; // Aspect ratio
    bird.style.opacity = '1';
    bird.style.transition = 'none';

    // Get card position
    const cardRect = targetCard.getBoundingClientRect();
    const cardCenterX = cardRect.left + cardRect.width / 2;
    const cardTopY = cardRect.top;

    // Start position: from top-left corner of viewport, flying towards card
    const startX = -200;
    const startY = window.scrollY - 200;

    // End position: perched on top-center of card
    const endX = cardCenterX - (this.config.birdSize / 2);
    const endY = window.scrollY + cardTopY - (this.config.birdSize * 0.5); // Slightly above card top

    bird.style.left = `${startX}px`;
    bird.style.top = `${startY}px`;

    // Add to DOM
    document.body.appendChild(bird);
    this.perchingBird = bird;

    // Wait a frame for DOM to settle
    await this.waitFrame();

    // Animate flying in (approach)
    bird.style.transition = 'all 1.5s cubic-bezier(0.25, 0.46, 0.45, 0.94)';
    bird.style.left = `${endX}px`;
    bird.style.top = `${endY}px`;
    bird.style.transform = 'rotate(-15deg)'; // Slight tilt while flying

    // Wait for flight to complete
    await this.wait(1500);

    // Landing adjustment (settle down a bit)
    bird.style.transition = 'all 0.3s ease-out';
    bird.style.transform = 'rotate(0deg)'; // Level out when perched

    // Wait while perched
    await this.wait(this.config.perchDuration);

    // Fly away (exit animation)
    bird.style.transition = 'all 1.2s cubic-bezier(0.55, 0.085, 0.68, 0.53)';

    // Fly towards top-right corner
    const exitX = window.innerWidth + 200;
    const exitY = window.scrollY - 200;

    bird.style.left = `${exitX}px`;
    bird.style.top = `${exitY}px`;
    bird.style.transform = 'rotate(20deg) scale(0.8)';
    bird.style.opacity = '0';

    // Wait for exit animation
    await this.wait(1200);

    // Remove bird
    bird.remove();
    this.perchingBird = null;
  }

  /**
   * Helper: wait for next animation frame
   */
  waitFrame() {
    return new Promise(resolve => requestAnimationFrame(resolve));
  }

  /**
   * Helper: wait for milliseconds
   */
  wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Helper: random number between min and max
   */
  randomBetween(min, max) {
    return min + Math.random() * (max - min);
  }

  /**
   * Destroy the system
   */
  destroy() {
    this.isActive = false;

    if (this.observer) {
      this.observer.disconnect();
    }

    if (this.perchingBird) {
      this.perchingBird.remove();
    }

    this.visibleCards.clear();
  }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    const perchingSystem = new BirdsPerchingSystem();
    perchingSystem.init();
  });
} else {
  const perchingSystem = new BirdsPerchingSystem();
  perchingSystem.init();
}
