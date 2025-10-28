/* ==========================================
   NEXUS SIMPLE HEADER - TYPEWRITER ANIMATION
   Cycles through absurd crypto phrases
   ========================================== */

(function() {
  'use strict';

  // 50+ ABSURD CRYPTO PHRASES
  const phrases = [
    'ENCRYPTING YOUR POTATO',
    'HASHING THE VIBES',
    'PEER REVIEWING NOTHING',
    'MULTISIG YOUR CAT',
    'ESCROW FOR THOUGHTS',
    'RING SIGNATURE PENDING',
    'STEALTH MODE: OBVIOUS',
    'ZERO KNOWLEDGE ACHIEVED',
    'ENCRYPTING SILENCE',
    'ANONYMIZING YOUR COFFEE',
    'BLOCKCHAIN SPAGHETTI',
    'OBFUSCATING TRANSPARENCY',
    'VALIDATING VIBES ONLY',
    'CONSENSUS BUT CONFUSED',
    'DOUBLE SPENDING TIME',
    'PRIVACY FOR PIGEONS',
    'CRYPTOGRAPHICALLY CONFUSED',
    'MINING FOR MEANING',
    'WALLET.DAT MISSING',
    'SEED PHRASE FORGOTTEN',
    'MONERO BUT ANXIOUS',
    'TOR BUT LOST',
    'PROOF OF NOTHING',
    'SMART CONTRACT REGRET',
    'NFT YOUR EXISTENTIAL DREAD',
    'ATOMIC SWAP GONE WRONG',
    'LIGHTNING BUT SLOW',
    'STAKING YOUR SANITY',
    'COLD STORAGE COLDER HEARTS',
    'HOT WALLET HOT MESS',
    'PAPER WALLET PAPER CUTS',
    'BRAIN WALLET BRAIN FOG',
    'HARDWARE WALLET SOFT FEELINGS',
    'FULL NODE EMPTY THOUGHTS',
    'LIGHT CLIENT HEAVY SOUL',
    'MEMPOOL MELTDOWN',
    'BLOCK HEIGHT ANXIETY',
    'DIFFICULTY ADJUSTMENT NEEDED',
    'TIMESTAMP TANTRUM',
    'NONCE SENSE',
    'MERKLE TREE BUT LOST',
    'UTXO LIMBO',
    'SATOSHI HIDING',
    'HALVING YOUR HOPES',
    'GENESIS BLOCK REGRET',
    'COINBASE BUT NOT THE EXCHANGE',
    'SCRIPTPUBKEY BUT SCARED',
    'ECDSA EXISTENTIAL CRISIS',
    'SHA256 SHATTERED DREAMS',
    'RPC CALL FAILED AGAIN'
  ];

  class TypewriterEffect {
    constructor(element, phrases) {
      this.element = element;
      this.phrases = phrases;
      this.currentPhraseIndex = 0;
      this.currentText = '';
      this.isDeleting = false;
      this.typingSpeed = 80;  // ms per character (typing)
      this.deletingSpeed = 40;  // ms per character (deleting - faster)
      this.pauseDuration = 2000;  // ms to pause after typing full phrase

      this.start();
    }

    start() {
      this.type();
    }

    type() {
      const currentPhrase = this.phrases[this.currentPhraseIndex];

      if (this.isDeleting) {
        // Deleting mode - remove characters
        this.currentText = currentPhrase.substring(0, this.currentText.length - 1);
      } else {
        // Typing mode - add characters
        this.currentText = currentPhrase.substring(0, this.currentText.length + 1);
      }

      // Update DOM
      this.element.textContent = this.currentText;

      // Calculate next delay
      let delay = this.isDeleting ? this.deletingSpeed : this.typingSpeed;

      // Check if we've completed typing the phrase
      if (!this.isDeleting && this.currentText === currentPhrase) {
        // Pause before deleting
        delay = this.pauseDuration;
        this.isDeleting = true;
      } else if (this.isDeleting && this.currentText === '') {
        // Move to next phrase
        this.isDeleting = false;
        this.currentPhraseIndex = (this.currentPhraseIndex + 1) % this.phrases.length;
        delay = 500;  // Short pause before next phrase
      }

      // Schedule next update
      setTimeout(() => this.type(), delay);
    }
  }

  // Initialize when DOM is ready
  function initTypewriter() {
    const element = document.getElementById('nexus-simple-typewriter-text');
    if (element && phrases.length > 0) {
      new TypewriterEffect(element, phrases);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initTypewriter);
  } else {
    initTypewriter();
  }
})();
