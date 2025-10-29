/**
 * NEXUS Birds Natural Animation
 *
 * Chaque oiseau individuel (path) a sa propre animation unique
 * 38 oiseaux uniques extraits du SVG et animés indépendamment
 */

class BirdsNaturalAnimation {
  constructor() {
    this.section = document.querySelector('.nexus-listings-section');
    this.birdsContainer = null;
    this.birds = [];
    this.animationStyleElement = null;

    // Configuration
    this.config = {
      svgPath: '/static/birds.svg',
      isMobile: window.innerWidth < 768,
      prefersReducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches
    };
  }

  /**
   * Initialize the animation
   */
  async init() {
    if (!this.section) {
      console.warn('NEXUS Birds: .nexus-listings-section not found');
      return;
    }

    // Don't init if user prefers reduced motion
    if (this.config.prefersReducedMotion) {
      console.info('NEXUS Birds: Animation disabled (prefers-reduced-motion)');
      return;
    }

    try {
      await this.createBirdsContainer();
      await this.loadIndividualBirds();
      this.createUniqueAnimations();
      this.applyAnimationsToBirds();

      console.info(`NEXUS Birds: Initialized with ${this.birds.length} individual birds (each with unique animation)`);
    } catch (error) {
      console.error('NEXUS Birds: Initialization failed', error);
    }
  }

  /**
   * Create the container div for birds
   */
  createBirdsContainer() {
    this.birdsContainer = document.createElement('div');
    this.birdsContainer.className = 'nexus-listings-birds-bg';
    this.birdsContainer.setAttribute('aria-hidden', 'true');

    // Insert as first child of section (behind content)
    this.section.insertBefore(this.birdsContainer, this.section.firstChild);
  }

  /**
   * Load and extract individual bird paths from SVG
   */
  async loadIndividualBirds() {
    const response = await fetch(this.config.svgPath);

    if (!response.ok) {
      throw new Error(`Failed to fetch ${this.config.svgPath}: ${response.statusText}`);
    }

    const svgText = await response.text();
    const parser = new DOMParser();
    const svgDoc = parser.parseFromString(svgText, 'image/svg+xml');
    const originalSvg = svgDoc.querySelector('svg');

    if (!originalSvg) {
      throw new Error('No SVG element found in birds.svg');
    }

    // Extract individual bird paths
    const paths = originalSvg.querySelectorAll('path');
    console.info(`NEXUS Birds: Found ${paths.length} individual bird paths in SVG`);

    if (paths.length === 0) {
      throw new Error('No bird paths found in SVG');
    }

    // Get SVG viewBox for proper scaling
    const viewBox = originalSvg.getAttribute('viewBox');
    const [vbX, vbY, vbWidth, vbHeight] = viewBox ? viewBox.split(' ').map(Number) : [0, 0, 1000, 1000];

    // Determine how many birds to show (less on mobile for performance)
    const birdsToShow = this.config.isMobile ? 20 : 38; // Show all 38 on desktop, 20 on mobile
    const actualBirdsToShow = Math.min(birdsToShow, paths.length);

    // Create individual bird elements from each path
    for (let i = 0; i < actualBirdsToShow; i++) {
      const path = paths[i];
      const bird = this.createIndividualBirdElement(path, i, vbWidth, vbHeight);
      this.birdsContainer.appendChild(bird);
      this.birds.push(bird);
    }
  }

  /**
   * Create a single bird element from ONE path
   */
  createIndividualBirdElement(pathElement, index, viewBoxWidth, viewBoxHeight) {
    const bird = document.createElement('div');
    bird.className = 'nexus-bird';
    bird.dataset.birdIndex = index;

    // Create a NEW SVG containing ONLY this one bird path
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('viewBox', `0 0 ${viewBoxWidth} ${viewBoxHeight}`);
    svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
    svg.style.width = '100%';
    svg.style.height = '100%';
    svg.style.display = 'block';

    // Clone the individual bird path
    const pathClone = pathElement.cloneNode(true);

    // Ensure path is black and filled
    pathClone.setAttribute('fill', '#000000');
    pathClone.setAttribute('stroke', 'none');

    svg.appendChild(pathClone);
    bird.appendChild(svg);

    // Position will be set in applyAnimationsToBirds
    bird.style.position = 'absolute';
    bird.style.pointerEvents = 'none';

    return bird;
  }

  /**
   * Create unique CSS keyframe animations for each bird
   */
  createUniqueAnimations() {
    // Create a <style> element to inject our animations
    this.animationStyleElement = document.createElement('style');
    this.animationStyleElement.id = 'nexus-birds-animations';

    let cssContent = '';

    // Generate unique animation for each bird
    this.birds.forEach((bird, index) => {
      const animationName = `bird-flight-${index}`;

      // Random movement pattern
      const pattern = this.generateRandomFlightPattern();

      cssContent += `
@keyframes ${animationName} {
  0% {
    transform: translate(${pattern.start.x}px, ${pattern.start.y}px) rotate(${pattern.start.rotate}deg) scale(${pattern.start.scale});
  }
  25% {
    transform: translate(${pattern.point1.x}px, ${pattern.point1.y}px) rotate(${pattern.point1.rotate}deg) scale(${pattern.point1.scale});
  }
  50% {
    transform: translate(${pattern.point2.x}px, ${pattern.point2.y}px) rotate(${pattern.point2.rotate}deg) scale(${pattern.point2.scale});
  }
  75% {
    transform: translate(${pattern.point3.x}px, ${pattern.point3.y}px) rotate(${pattern.point3.rotate}deg) scale(${pattern.point3.scale});
  }
  100% {
    transform: translate(${pattern.start.x}px, ${pattern.start.y}px) rotate(${pattern.start.rotate}deg) scale(${pattern.start.scale});
  }
}
`;

      // Store animation name for later use
      bird.dataset.animationName = animationName;
    });

    this.animationStyleElement.textContent = cssContent;
    document.head.appendChild(this.animationStyleElement);
  }

  /**
   * Generate a random flight pattern with 4 control points
   */
  generateRandomFlightPattern() {
    // Random movement range (in pixels)
    const xRange = this.config.isMobile ? 60 : 120;
    const yRange = this.config.isMobile ? 40 : 80;

    // Create 4 points for a smooth flight path
    return {
      start: {
        x: 0,
        y: 0,
        rotate: this.randomBetween(-10, 10),
        scale: this.randomBetween(0.95, 1.05)
      },
      point1: {
        x: this.randomBetween(-xRange, xRange),
        y: this.randomBetween(-yRange, yRange),
        rotate: this.randomBetween(-15, 15),
        scale: this.randomBetween(0.9, 1.1)
      },
      point2: {
        x: this.randomBetween(-xRange, xRange),
        y: this.randomBetween(-yRange, yRange),
        rotate: this.randomBetween(-20, 20),
        scale: this.randomBetween(0.85, 1.15)
      },
      point3: {
        x: this.randomBetween(-xRange, xRange),
        y: this.randomBetween(-yRange, yRange),
        rotate: this.randomBetween(-15, 15),
        scale: this.randomBetween(0.9, 1.1)
      }
    };
  }

  /**
   * Apply unique animations to each bird
   */
  applyAnimationsToBirds() {
    // Random positioning across the entire section
    this.birds.forEach((bird, index) => {
      // Completely random position (0% to 100% in both X and Y)
      const left = this.randomBetween(0, 100);
      const top = this.randomBetween(0, 100);

      bird.style.left = `${left}%`;
      bird.style.top = `${top}%`;

      // Random size variation (each bird has different size)
      const scale = this.randomBetween(0.5, 1.2);
      const baseWidth = this.config.isMobile ? 150 : 225; // 1.5x plus grand
      const baseHeight = this.config.isMobile ? 100 : 150; // 1.5x plus grand
      bird.style.width = `${baseWidth * scale}px`;
      bird.style.height = `${baseHeight * scale}px`;

      // Random animation duration (10s to 30s for natural variety)
      const duration = this.randomBetween(10, 30);

      // Random delay (0s to 15s so they start at different times)
      const delay = this.randomBetween(0, 15);

      // Random timing function for more variety
      const timingFunctions = [
        'ease-in-out',
        'cubic-bezier(0.45, 0.05, 0.55, 0.95)', // easeInOutQuad
        'cubic-bezier(0.65, 0, 0.35, 1)', // easeInOutCubic
        'cubic-bezier(0.86, 0, 0.07, 1)', // easeInOutQuart
        'cubic-bezier(0.25, 0.46, 0.45, 0.94)' // easeOutQuad
      ];
      const timingFunction = timingFunctions[Math.floor(Math.random() * timingFunctions.length)];

      // Apply the unique animation
      bird.style.animation = `${bird.dataset.animationName} ${duration}s ${timingFunction} ${delay}s infinite`;

      // Random z-index for depth (1-3 for behind cards)
      const zIndex = Math.floor(this.randomBetween(1, 3));
      bird.style.zIndex = zIndex;

      // Slight opacity variation for depth (0.6 to 1.0)
      const opacity = this.randomBetween(0.6, 1.0);
      bird.style.opacity = opacity;
    });
  }

  /**
   * Helper: random number between min and max
   */
  randomBetween(min, max) {
    return min + Math.random() * (max - min);
  }

  /**
   * Destroy the animation (cleanup)
   */
  destroy() {
    if (this.birdsContainer) {
      this.birdsContainer.remove();
    }

    if (this.animationStyleElement) {
      this.animationStyleElement.remove();
    }

    this.birds = [];
  }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    const birdsAnim = new BirdsNaturalAnimation();
    birdsAnim.init();
  });
} else {
  const birdsAnim = new BirdsNaturalAnimation();
  birdsAnim.init();
}
