/* ==========================================
   NEXUS - HORIZONTAL LINES MAGNETIC EFFECT
   Lignes horizontales ondulantes qui réagissent au curseur
   ========================================== */

(function() {
  'use strict';

  class MagneticGrid {
    constructor() {
      this.grid = document.querySelector('.nexus-hero-grid');

      if (!this.grid) {
        console.warn('⚠️ .nexus-hero-grid not found');
        return;
      }

      // Configuration
      this.config = {
        lineSpacing: 50,        // Espacement entre les lignes (50px comme dans le CSS)
        amplitude: 30,          // Amplitude de l'ondulation
        magnetRadius: 120,      // Rayon d'influence du curseur
        magnetStrength: 0.4,    // Force de l'attraction
        returnSpeed: 0.12,      // Vitesse de retour à la position initiale
        lineWidth: 2            // Épaisseur des lignes
      };

      this.mouse = { x: 0, y: 0 };
      this.lines = [];
      this.animationId = null;

      this.init();
    }

    init() {
      // Calculer le nombre de lignes nécessaires
      const gridHeight = this.grid.offsetHeight;
      const lineCount = Math.ceil(gridHeight / this.config.lineSpacing) + 2;

      // Créer les lignes
      for (let i = 0; i < lineCount; i++) {
        this.lines.push({
          baseY: i * this.config.lineSpacing,
          currentOffset: 0,
          targetOffset: 0
        });
      }

      // Event listeners
      document.addEventListener('mousemove', (e) => this.onMouseMove(e));

      // Commencer l'animation
      this.animate();

      console.log('✅ Magnetic grid initialized with', lineCount, 'lines');
    }

    onMouseMove(e) {
      const rect = this.grid.getBoundingClientRect();
      this.mouse.x = e.clientX - rect.left;
      this.mouse.y = e.clientY - rect.top;
    }

    calculateMagneticEffect(lineY) {
      const dx = 0; // Pas de décalage horizontal pour l'instant
      const dy = this.mouse.y - lineY;
      const distance = Math.sqrt(dx * dx + dy * dy);

      if (distance < this.config.magnetRadius) {
        // Force d'attraction basée sur la distance
        const force = (1 - distance / this.config.magnetRadius) * this.config.magnetStrength;
        return dy * force;
      }

      return 0;
    }

    animate() {
      // Dessiner les lignes avec l'effet magnétique
      let gradient = 'repeating-linear-gradient(0deg, transparent, transparent ';

      this.lines.forEach((line, i) => {
        // Calculer l'effet magnétique
        const targetOffset = this.calculateMagneticEffect(line.baseY);

        // Interpolation douce vers la cible
        line.currentOffset += (targetOffset - line.currentOffset) * this.config.returnSpeed;

        // Position finale de la ligne
        const linePos = line.baseY + line.currentOffset;

        // Ajouter à la gradient
        const start = linePos - 1;
        const end = linePos + 1;
        gradient += `${start}px, #000 ${start}px, #000 ${end}px, transparent ${end}px`;

        if (i < this.lines.length - 1) {
          gradient += ', transparent ';
        }
      });

      gradient += ')';

      // Appliquer le gradient
      this.grid.style.backgroundImage = gradient;

      // Continuer l'animation
      this.animationId = requestAnimationFrame(() => this.animate());
    }

    destroy() {
      if (this.animationId) {
        cancelAnimationFrame(this.animationId);
      }
    }
  }

  // Initialize when DOM is ready
  function init() {
    new MagneticGrid();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
