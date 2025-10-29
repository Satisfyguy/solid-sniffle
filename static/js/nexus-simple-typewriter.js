/* ==========================================
   NEXUS SIMPLE HEADER - TYPEWRITER ANIMATION
   Cycles through absurd crypto phrases
   ========================================== */

(function() {
  'use strict';

  // 150 ABSURD CRYPTO PHRASES
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
    'RPC CALL FAILED AGAIN',
    'VALIDATING EXISTENTIAL DREAD',
    'ORPHAN BLOCKS ADOPTED',
    'NONCE MAKES NO SENSE',
    'DIFFICULTY ADJUSTED DOWN',
    'MINING YOUR BUSINESS',
    'PROOF OF WORK AVOIDED',
    'CONSENSUS SPLIT PERSONALITY',
    'FORK HAPPENED AGAIN',
    'SOFT FORK HARD FEELINGS',
    'CHAIN REORGANIZING LIFE',
    'DOUBLE SPEND REGRET',
    'WALLET SYNC FOREVER',
    'SEED PHRASE ANXIETY',
    'MNEMONIC FORGOT ALREADY',
    'PRIVATE KEY PUBLIC SHAME',
    'ADDRESS REUSE JUDGMENT',
    'DUST ATTACK TICKLES',
    'FEE MARKET HOSTILE',
    'TRANSACTION STUCK LIMBO',
    'REPLACE BY FEE THERAPY',
    'CHILD PAYS PARENT ISSUES',
    'UTXO SET BLOATED',
    'SCRIPTPUBKEY CONFUSED',
    'OPRETURN SCREAMING VOID',
    'LOCKTIME EXPIRED PATIENCE',
    'SEQUENCE NUMBER LOST COUNT',
    'WITNESS DATA TESTIFYING',
    'SEGWIT BUT REGRET',
    'TAPROOT PLANTED DOUBT',
    'SCHNORR SIGNATURE SHRUGGING',
    'LIGHTNING NETWORK SCARED',
    'CHANNEL CAPACITY EMOTIONAL',
    'ROUTING FAILED AGAIN',
    'INVOICE EXPIRED MOOD',
    'ONION ROUTING CRYING',
    'PAYMENT HASH BROWNS',
    'PREIMAGE REVEALED TRAUMA',
    'HTLC TIMEOUT FOREVER',
    'SUBMARINE SWAP DROWNING',
    'WATCHTOWER SLEEPING',
    'JUSTICE TRANSACTION BITTER',
    'COMMITMENT TX ISSUES',
    'REVOCATION KEY REGRETS',
    'FUNDING TX AWKWARD',
    'CLOSING CHANNEL CLOSURE',
    'FORCE CLOSE VIOLENCE',
    'COOPERATIVE CLOSE LIES',
    'ANCHOR OUTPUT DRIFTING',
    'DUST LIMIT EXISTENTIAL',
    'SATOSHI VISION BLURRY',
    'BLOCK REWARD DISAPPOINTING',
    'HALVING CRISIS MIDLIFE',
    'DIFFICULTY BOMB THERAPY',
    'UNCLE BLOCK ESTRANGED',
    'GAS LIMIT SUFFOCATING',
    'NONCE OVERFLOW PANIC',
    'REORG DEPTH VERTIGO',
    'STALE BLOCK MOLDY',
    'SELFISH MINING HONEST',
    'ECLIPSE ATTACK LONELY',
    'SYBIL ATTACK IDENTITY',
    'FIFTY ONE PERCENT DOUBT',
    'NOTHING AT STAKE PHILOSOPHY',
    'LONG RANGE ATTACK NOSTALGIA',
    'GRINDING ATTACK EXHAUSTED',
    'TIMESTAMP MANIPULATION LIES',
    'FRONT RUNNING LATE',
    'SANDWICH ATTACK HUNGRY',
    'MEV EXTRACTING SOUL',
    'FLASHBOTS REGRETTING',
    'DARK FOREST LOST',
    'SLIPPAGE TOLERANCE ZERO',
    'IMPERMANENT LOSS PERMANENT',
    'LIQUIDITY POOL DROWNING',
    'AUTOMATED MARKET CONFUSED',
    'BONDING CURVE BROKEN',
    'PRICE ORACLE LYING',
    'FLASH LOAN REGRET',
    'SMART CONTRACT DUMB',
    'SOLIDITY MELTING',
    'GAS OPTIMIZATION FUTILE',
    'REENTRANCY GUARD SLEEPING',
    'OVERFLOW UNDERFLOW EXISTENTIAL',
    'DELEGATE CALL TRUST ISSUES',
    'SELFDESTRUCT TEMPTING',
    'FALLBACK FUNCTION CRYING',
    'PROXY CONTRACT IMPOSTER',
    'UPGRADEABLE REGRETS',
    'IMMUTABLE LIES',
    'DETERMINISTIC CHAOS',
    'NONDETERMINISTIC CERTAINTY',
    'STATE MACHINE BROKEN',
    'EVENT LOG TRAUMA',
    'ABI ENCODING GIBBERISH',
    'BYTECODE INCOMPREHENSIBLE',
    'EVM COMPATIBLE BARELY',
    'CROSS CHAIN BRIDGE COLLAPSED',
    'ATOMIC SWAP NUCLEAR',
    'ZERO CONF TRUST FALL',
    'REPLACE BY FEE BETRAYAL'
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
    // Check for prefers-reduced-motion
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      console.log('⏸️ Typewriter animation disabled (prefers-reduced-motion)');
      return;
    }

    const element = document.getElementById('nexus-simple-typewriter-text');
    console.log('🔍 Typewriter init - Element found:', element);
    console.log('📝 Phrases count:', phrases.length);
    if (element && phrases.length > 0) {
      console.log('✅ Starting typewriter animation');
      new TypewriterEffect(element, phrases);
    } else {
      console.error('❌ Typewriter failed - Element:', element, 'Phrases:', phrases.length);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initTypewriter);
  } else {
    initTypewriter();
  }
})();
