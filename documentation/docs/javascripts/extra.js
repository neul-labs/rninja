// rninja Documentation Custom JavaScript

// Initialize when DOM is ready
document.addEventListener("DOMContentLoaded", function() {
  // Add copy feedback to code blocks
  initCopyFeedback();

  // Initialize table sorting if tablesort is available
  initTableSort();
});

// Copy button feedback
function initCopyFeedback() {
  document.querySelectorAll('.md-clipboard').forEach(function(button) {
    button.addEventListener('click', function() {
      const original = button.getAttribute('data-clipboard-text');
      if (original) {
        // Visual feedback is handled by Material theme
        console.log('Copied to clipboard');
      }
    });
  });
}

// Table sorting initialization
function initTableSort() {
  if (typeof Tablesort !== 'undefined') {
    document.querySelectorAll('table.sortable').forEach(function(table) {
      new Tablesort(table);
    });
  }
}

// Smooth scroll for anchor links
document.querySelectorAll('a[href^="#"]').forEach(function(anchor) {
  anchor.addEventListener('click', function(e) {
    const target = document.querySelector(this.getAttribute('href'));
    if (target) {
      e.preventDefault();
      target.scrollIntoView({
        behavior: 'smooth',
        block: 'start'
      });
    }
  });
});
