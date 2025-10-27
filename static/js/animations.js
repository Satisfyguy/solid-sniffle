// Animation des lettres NEXUS
document.addEventListener('DOMContentLoaded', function() {
  const letters = document.querySelectorAll('.animated-letter');

  letters.forEach(letter => {
    letter.addEventListener('mouseenter', function() {
      this.classList.add('jumping');

      setTimeout(() => {
        this.classList.remove('jumping');
      }, 1500);
    });
  });

  // Smooth scroll pour les liens d'ancrage
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
      e.preventDefault();
      const target = document.querySelector(this.getAttribute('href'));
      if (target) {
        target.scrollIntoView({
          behavior: 'smooth',
          block: 'start'
        });
      }
    });
  });
});
