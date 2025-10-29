/**
 * Hero Cinematic Animation Controller
 * Séquence d'intro en 5 phases (~11 secondes total)
 * 
 * Phase 1: Jump Sync       (0.5s → 2.3s)  - Toutes lettres sautent ensemble
 * Phase 2: Explosion       (2.3s → 2.8s)  - Lettres explosent dans 5 directions
 * Phase 3: Typewriter      (2.8s → 8.0s)  - Message lent (100ms/lettre)
 * Phase 4: Zoom + Shake    (8.0s → 9.5s)  - Flash blanc + tremblement + zoom
 * Phase 5: Fall from Sky   (9.5s → 11.1s) - NEXUS retombe en désordre
 * 
 * Hover individuel réactivé après séquence complète
 */

class HeroCinematicController {
  constructor() {
    this.phase = 0;
    this.timers = [];
    
    // Sélecteurs DOM
    this.elements = {
      heroSection: null,
      nexusLetters: null,
      letters: [],
      typewriterContainer: null,
      typewriterText: null,
      zoomStrike: null,
      flash: null
    };
    
    // Configuration des timings (en millisecondes)
    this.config = {
      timings: {
        phase1Start: 500,      // 0.5s après chargement
        phase2Start: 2300,     // +1.8s (durée phase 1)
        phase3Start: 2800,     // +0.5s (durée explosion)
        phase4Start: 9000,     // +5.2s (typewriter) + 1s (pause pour lire)
        phase5Start: 10500     // +1.5s (durée zoom + shake)
      },
      typewriterSpeed: 100,    // 100ms par lettre (LENT pour effet calme)
      typewriterText: "Here you are no one. Here you are nowhere."
    };
  }

  /**
   * Initialisation - Récupère les éléments DOM et lance la séquence
   */
  init() {
    // Récupérer les éléments DOM
    this.elements.heroSection = document.querySelector('.nexus-hero-true');
    this.elements.nexusLetters = document.getElementById('cinematic-nexus-letters');
    this.elements.typewriterContainer = document.getElementById('cinematic-typewriter');
    this.elements.typewriterText = document.getElementById('cinematic-typewriter-text');
    this.elements.zoomStrike = document.getElementById('cinematic-zoom-strike');
    this.elements.flash = document.getElementById('cinematic-flash');
    this.elements.letters = Array.from(document.querySelectorAll('.nexus-animated-letter'));
    
    // Vérifier que tous les éléments sont présents
    if (!this.elements.heroSection || !this.elements.nexusLetters ||
        this.elements.letters.length === 0) {
      console.error('[HeroCinematic] Éléments DOM manquants');
      return;
    }

    // Check for prefers-reduced-motion
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      console.log('[HeroCinematic] Animation disabled (prefers-reduced-motion)');
      return;
    }

    // Ajouter classe pour désactiver hover pendant cinématique
    this.elements.heroSection.classList.add('cinematic-active');
    
    // Lancer la séquence
    console.log('[HeroCinematic] Démarrage de la séquence');
    this.startSequence();
  }

  /**
   * Lancement de la séquence complète avec timings précis
   */
  startSequence() {
    // Phase 1: Jump Sync (0.5s)
    this.timers.push(
      setTimeout(() => this.phase1_JumpSync(), this.config.timings.phase1Start)
    );
    
    // Phase 2: Explosion (2.3s)
    this.timers.push(
      setTimeout(() => this.phase2_Explode(), this.config.timings.phase2Start)
    );
    
    // Phase 3: Typewriter (2.8s)
    this.timers.push(
      setTimeout(() => this.phase3_Typewriter(), this.config.timings.phase3Start)
    );
    
    // Phase 4: Zoom + Shake (8.0s)
    this.timers.push(
      setTimeout(() => this.phase4_ZoomShake(), this.config.timings.phase4Start)
    );
    
    // Phase 5: Fall from Sky (9.5s)
    this.timers.push(
      setTimeout(() => this.phase5_FallFromSky(), this.config.timings.phase5Start)
    );
  }

  /**
   * Phase 1: Jump Sync
   * Toutes les lettres sautent ensemble (durée: 1.8s)
   */
  phase1_JumpSync() {
    this.phase = 1;
    console.log('[HeroCinematic] Phase 1: Jump Sync');
    
    // Appliquer l'animation de saut à toutes les lettres
    this.elements.letters.forEach(letter => {
      letter.classList.add('cinematic-jump');
    });
  }

  /**
   * Phase 2: Explosion
   * Les lettres explosent dans 5 directions différentes (durée: 0.5s)
   */
  phase2_Explode() {
    this.phase = 2;
    console.log('[HeroCinematic] Phase 2: Explosion');
    
    // Retirer l'animation de phase 1
    this.elements.letters.forEach(letter => {
      letter.classList.remove('cinematic-jump');
    });
    
    // Appliquer les animations d'explosion individuelles
    this.elements.letters.forEach((letter, index) => {
      letter.classList.add(`cinematic-explode-${index}`);
    });
  }

  /**
   * Phase 3: Typewriter
   * Message s'écrit lettre par lettre LENTEMENT (durée: 5.2s)
   * Effet: Calme, rassurant, donne le temps de lire
   */
  phase3_Typewriter() {
    this.phase = 3;
    console.log('[HeroCinematic] Phase 3: Typewriter (SLOW)');
    
    // Cacher NEXUS, montrer typewriter
    this.elements.nexusLetters.style.display = 'none';
    this.elements.typewriterContainer.style.display = 'block';
    
    // Effet typewriter
    const text = this.config.typewriterText;
    let index = 0;
    
    const typeInterval = setInterval(() => {
      if (index < text.length) {
        this.elements.typewriterText.textContent = text.slice(0, index + 1);
        index++;
      } else {
        clearInterval(typeInterval);
      }
    }, this.config.typewriterSpeed);
  }

  /**
   * Phase 4: Zoom + Shake + Flash
   * Le texte zoom et frappe l'écran brutalement (durée: 1.5s)
   * Effet: BOOM! Contraste maximum avec le calme de Phase 3
   */
  phase4_ZoomShake() {
    this.phase = 4;
    console.log('[HeroCinematic] Phase 4: 💥 ZOOM + SHAKE + FLASH');
    
    // Cacher typewriter, montrer zoom-strike
    this.elements.typewriterContainer.style.display = 'none';
    this.elements.zoomStrike.style.display = 'block';
    this.elements.zoomStrike.classList.add('cinematic-zoom');
    
    // Flash blanc AVEUGLANT (opacity 1.0)
    this.elements.flash.classList.add('cinematic-flash');
    setTimeout(() => {
      this.elements.flash.classList.remove('cinematic-flash');
    }, 500);
    
    // Shake de l'écran VIOLENT (±10px)
    this.elements.heroSection.classList.add('cinematic-shake');
    setTimeout(() => {
      this.elements.heroSection.classList.remove('cinematic-shake');
    }, 500);
  }

  /**
   * Phase 5: Fall from Sky
   * Les lettres NEXUS retombent du ciel en désordre (durée: 1.6s)
   * Chaque lettre tombe avec un délai de 0.1s
   */
  phase5_FallFromSky() {
    this.phase = 5;
    console.log('[HeroCinematic] Phase 5: Fall from Sky');
    
    // Cacher zoom-strike, montrer NEXUS
    this.elements.zoomStrike.style.display = 'none';
    this.elements.zoomStrike.classList.remove('cinematic-zoom');
    this.elements.nexusLetters.style.display = 'flex';
    
    // Réinitialiser et appliquer animations de chute
    this.elements.letters.forEach((letter, index) => {
      // Retirer classes explosion
      letter.classList.remove(`cinematic-explode-${index}`);
      
      // Réinitialiser styles inline (de phase 2)
      letter.style.transform = '';
      letter.style.opacity = '1';
      letter.style.transition = '';
      
      // Appliquer animation de chute avec délai progressif
      letter.classList.add(`cinematic-fall-${index}`);
      letter.style.animationDelay = `${index * 0.1}s`;
    });
    
    // Réactiver hover après que toutes les lettres soient retombées
    // Durée: 1s (animation) + 0.4s (dernier delay) + 0.2s (buffer) = 1.6s
    setTimeout(() => this.reactivateHover(), 1600);
  }

  /**
   * Réactiver le hover individuel après la cinématique
   */
  reactivateHover() {
    console.log('[HeroCinematic] Séquence terminée - Hover réactivé');
    
    // Retirer classe qui désactive hover
    this.elements.heroSection.classList.remove('cinematic-active');
    
    // Nettoyer toutes les classes cinématiques
    this.elements.letters.forEach((letter, index) => {
      letter.classList.remove(`cinematic-fall-${index}`);
      letter.style.animationDelay = '';
    });
    
    // Les event listeners du fichier nexus-letters-animation.js
    // sont déjà attachés et fonctionneront automatiquement
  }

  /**
   * Cleanup - Nettoyer les timers si nécessaire
   */
  cleanup() {
    console.log('[HeroCinematic] Cleanup');
    this.timers.forEach(timer => clearTimeout(timer));
    this.timers = [];
  }
}

// ========================================
// INITIALISATION AUTOMATIQUE
// ========================================

/**
 * Lancer la cinématique automatiquement au chargement de la page
 * PAS de sessionStorage - rejoue à chaque chargement
 */
(function() {
  'use strict';
  
  function initCinematic() {
    const controller = new HeroCinematicController();
    controller.init();
  }
  
  // Init au chargement DOM
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initCinematic);
  } else {
    // DOM déjà chargé
    initCinematic();
  }
})();
