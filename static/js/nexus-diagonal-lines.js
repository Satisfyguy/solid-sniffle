/* ==========================================
   NEXUS - DIAGONAL LINES ANIMATED
   Chaque ligne bouge indépendamment
   ========================================== */

(function() {
  'use strict';

  class DiagonalLines {
    constructor() {
      this.container = document.querySelector('.nexus-hero-pattern');

      if (!this.container) {
        console.warn('⚠️ .nexus-hero-pattern not found');
        return;
      }

      // Configuration
      this.config = {
        lineCount: 30,          // Nombre de lignes diagonales
        lineWidth: 3,           // Épaisseur des lignes
        lineSpacing: 60,        // Espacement entre les lignes
        waveAmplitude: 20,      // Amplitude de l'ondulation
        waveSpeed: 0.02,        // Vitesse de l'animation
        angle: 45               // Angle des lignes (45 degrés)
      };

      this.lines = [];
      this.time = 0;
      this.animationId = null;

      this.init();
    }

    init() {
      // Check for prefers-reduced-motion
      if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
        console.log('⏸️ Diagonal lines animation disabled (prefers-reduced-motion)');
        return;
      }

      // Créer un canvas pour dessiner les lignes
      this.canvas = document.createElement('canvas');
      this.ctx = this.canvas.getContext('2d');
      this.canvas.style.position = 'absolute';
      this.canvas.style.top = '0';
      this.canvas.style.left = '0';
      this.canvas.style.width = '100%';
      this.canvas.style.height = '100%';
      this.canvas.style.pointerEvents = 'none';

      this.container.appendChild(this.canvas);

      // Initialiser les dimensions
      this.resize();
      window.addEventListener('resize', () => this.resize());

      // Créer les lignes avec décalage de phase pour effet ondulant
      for (let i = 0; i < this.config.lineCount; i++) {
        this.lines.push({
          offset: i * this.config.lineSpacing,
          phase: (i / this.config.lineCount) * Math.PI * 2 // Décalage de phase
        });
      }

      // Démarrer l'animation
      this.animate();

      console.log('✅ Diagonal lines initialized with', this.config.lineCount, 'lines');
    }

    resize() {
      const rect = this.container.getBoundingClientRect();
      this.canvas.width = rect.width;
      this.canvas.height = rect.height;
    }

    drawLine(startX, startY, endX, endY) {
      this.ctx.strokeStyle = 'rgba(0, 0, 0, 0.2)';
      this.ctx.lineWidth = this.config.lineWidth;
      this.ctx.lineCap = 'round';

      this.ctx.beginPath();
      this.ctx.moveTo(startX, startY);
      this.ctx.lineTo(endX, endY);
      this.ctx.stroke();
    }

    animate() {
      // Effacer le canvas
      this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

      const diagonal = Math.sqrt(this.canvas.width ** 2 + this.canvas.height ** 2);
      const angleRad = (this.config.angle * Math.PI) / 180;

      this.lines.forEach((line) => {
        // Calculer l'ondulation pour cette ligne
        const wave = Math.sin(this.time + line.phase) * this.config.waveAmplitude;

        // Position de base de la ligne + ondulation
        const offset = line.offset + wave;

        // Calculer les points de départ et d'arrivée pour une ligne diagonale
        const startX = -diagonal + offset;
        const startY = 0;
        const endX = this.canvas.width;
        const endY = this.canvas.height + diagonal - offset;

        this.drawLine(startX, startY, endX, endY);
      });

      // Incrémenter le temps pour l'animation
      this.time += this.config.waveSpeed;

      // Continuer l'animation
      this.animationId = requestAnimationFrame(() => this.animate());
    }

    destroy() {
      if (this.animationId) {
        cancelAnimationFrame(this.animationId);
      }
      if (this.canvas) {
        this.canvas.remove();
      }
    }
  }

  // Initialize when DOM is ready
  let diagonalLinesInstance = null;

  function init() {
    // Cleanup previous instance if exists (HTMX navigation)
    if (diagonalLinesInstance) {
      diagonalLinesInstance.destroy();
    }
    diagonalLinesInstance = new DiagonalLines();
  }

  // Cleanup on page unload
  window.addEventListener('beforeunload', () => {
    if (diagonalLinesInstance) {
      diagonalLinesInstance.destroy();
      diagonalLinesInstance = null;
    }
  });

  // Cleanup on HTMX navigation
  document.body.addEventListener('htmx:beforeSwap', () => {
    if (diagonalLinesInstance) {
      diagonalLinesInstance.destroy();
      diagonalLinesInstance = null;
    }
  });

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
