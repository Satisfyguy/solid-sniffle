/* ==========================================
   NEXUS - INTERACTIVE WAVY LINES (VERTICAL like wodniack)
   Canvas-based VERTICAL wavy lines with magnetic cursor attraction
   Lines go from top to bottom, bulge horizontally
   ========================================== */

(function() {
  'use strict';

  class InteractiveWavyLines {
    constructor(canvas) {
      this.canvas = canvas;
      this.ctx = canvas.getContext('2d');

      // Configuration
      this.config = {
        lineCount: 100,             // BEAUCOUP PLUS de lignes (densité comme wodniack)
        pointSpacing: 12,           // px entre points sur chaque ligne (vertical)
        amplitude: 30,              // Ondulation horizontale
        frequency: 0.008,           // Fréquence ondulation
        speed: 0.015,               // Vitesse animation
        magnetRadius: 150,          // Distance attraction curseur
        magnetStrength: 0.3,        // Force d'attraction
        returnSpeed: 0.08,          // Vitesse retour
        lineWidth: 1.5,             // Lignes FINES (style wodniack - densité pas épaisseur)
        isMobile: this.checkMobile()
      };

      // State
      this.lines = [];
      this.mouse = {
        x: -1000,
        y: -1000,
        isActive: false,
        lastMoveTime: 0
      };
      this.time = 0;
      this.animationFrame = null;
      this.isInitialized = false;

      // Bind methods
      this.handleMouseMove = this.handleMouseMove.bind(this);
      this.handleMouseLeave = this.handleMouseLeave.bind(this);
      this.handleResize = this.handleResize.bind(this);
      this.animate = this.animate.bind(this);

      // Initialize
      this.init();
    }

    checkMobile() {
      return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
        || window.innerWidth < 768;
    }

    init() {
      // Set canvas size
      this.resize();

      // Generate lines
      this.generateLines();

      // Setup event listeners
      if (!this.config.isMobile) {
        document.addEventListener('mousemove', this.handleMouseMove);
        document.addEventListener('mouseleave', this.handleMouseLeave);
      }
      window.addEventListener('resize', this.handleResize);

      // Start animation
      this.isInitialized = true;
      this.animate();
    }

    resize() {
      const dpr = window.devicePixelRatio || 1;
      this.canvas.width = window.innerWidth * dpr;
      this.canvas.height = window.innerHeight * dpr;
      this.canvas.style.width = `${window.innerWidth}px`;
      this.canvas.style.height = `${window.innerHeight}px`;
      this.ctx.scale(dpr, dpr);

      // Regenerate lines on resize
      if (this.isInitialized) {
        this.generateLines();
      }
    }

    generateLines() {
      this.lines = [];
      const width = window.innerWidth;
      const height = window.innerHeight;
      const spacing = width / (this.config.lineCount + 1);
      const pointCount = Math.ceil(height / this.config.pointSpacing) + 1;

      for (let i = 0; i < this.config.lineCount; i++) {
        const line = {
          baseX: spacing * (i + 1), // Position X de base (répartition horizontale)
          points: [],
          phaseOffset: Math.random() * Math.PI * 2 // Phase random pour variété
        };

        // Generate points for this line (vertical)
        for (let j = 0; j < pointCount; j++) {
          line.points.push({
            y: j * this.config.pointSpacing,
            x: line.baseX,
            baseX: line.baseX,
            targetX: line.baseX,
            velocityX: 0
          });
        }

        this.lines.push(line);
      }
    }

    handleMouseMove(e) {
      this.mouse.x = e.clientX;
      this.mouse.y = e.clientY;
      this.mouse.isActive = true;
      this.mouse.lastMoveTime = Date.now();

      // Auto-deactivate if mouse hasn't moved for 500ms
      setTimeout(() => {
        if (Date.now() - this.mouse.lastMoveTime >= 500) {
          this.mouse.isActive = false;
        }
      }, 500);
    }

    handleMouseLeave() {
      this.mouse.isActive = false;
      this.mouse.x = -1000;
      this.mouse.y = -1000;
    }

    handleResize() {
      // Debounce resize
      clearTimeout(this.resizeTimeout);
      this.resizeTimeout = setTimeout(() => {
        this.config.isMobile = this.checkMobile();
        this.resize();
      }, 250);
    }

    update() {
      this.time += this.config.speed;

      this.lines.forEach(line => {
        line.points.forEach(point => {
          // Calculate base wave position (ondulation horizontale)
          const wavePhase = point.y * this.config.frequency + this.time + line.phaseOffset;
          point.baseX = line.baseX + Math.sin(wavePhase) * this.config.amplitude;

          // Apply magnetic attraction if mouse is active and within radius
          if (this.mouse.isActive && !this.config.isMobile) {
            const dx = point.x - this.mouse.x;
            const dy = point.y - this.mouse.y;
            const distance = Math.sqrt(dx * dx + dy * dy);

            if (distance < this.config.magnetRadius) {
              // Calculate attraction force (stronger when closer)
              const force = (1 - distance / this.config.magnetRadius) * this.config.magnetStrength;
              const attractionX = this.mouse.x + dx * (1 - force);

              // Set target position towards cursor
              point.targetX = point.baseX + (attractionX - point.baseX) * force * 2;
            } else {
              // Outside radius - return to base
              point.targetX = point.baseX;
            }
          } else {
            // Mouse not active - return to base wave
            point.targetX = point.baseX;
          }

          // Smooth interpolation towards target (easing)
          const diff = point.targetX - point.x;
          point.velocityX += diff * this.config.returnSpeed;
          point.velocityX *= 0.85; // Damping for smooth motion
          point.x += point.velocityX;
        });
      });
    }

    draw() {
      // Clear canvas
      this.ctx.clearRect(0, 0, window.innerWidth, window.innerHeight);

      // Set line style - Lignes FINES et DENSES comme wodniack
      this.ctx.strokeStyle = 'rgba(0, 0, 0, 0.45)';  // NOIR 45% opacité (avec 100 lignes)
      this.ctx.lineWidth = this.config.lineWidth;
      this.ctx.lineCap = 'round';
      this.ctx.lineJoin = 'round';

      // Draw each line (vertical)
      this.lines.forEach(line => {
        this.ctx.beginPath();

        // Draw smooth curve through all points
        if (line.points.length > 0) {
          this.ctx.moveTo(line.points[0].x, line.points[0].y);

          for (let i = 1; i < line.points.length - 1; i++) {
            const xMid = (line.points[i].x + line.points[i + 1].x) / 2;
            const yMid = (line.points[i].y + line.points[i + 1].y) / 2;
            this.ctx.quadraticCurveTo(line.points[i].x, line.points[i].y, xMid, yMid);
          }

          // Last point
          const lastPoint = line.points[line.points.length - 1];
          this.ctx.lineTo(lastPoint.x, lastPoint.y);
        }

        this.ctx.stroke();
      });
    }

    animate() {
      this.update();
      this.draw();
      this.animationFrame = requestAnimationFrame(this.animate);
    }

    destroy() {
      // Cleanup
      if (this.animationFrame) {
        cancelAnimationFrame(this.animationFrame);
      }
      document.removeEventListener('mousemove', this.handleMouseMove);
      document.removeEventListener('mouseleave', this.handleMouseLeave);
      window.removeEventListener('resize', this.handleResize);
    }
  }

  // Initialize when DOM is ready
  function initLines() {
    const canvas = document.getElementById('nexus-lines-canvas');
    if (canvas) {
      new InteractiveWavyLines(canvas);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initLines);
  } else {
    initLines();
  }
})();
