// Navigation functionality

// Navigate to a page
function navigate(page) {
  // Hide all pages
  document.querySelectorAll('.page').forEach(p => {
    p.classList.remove('active');
  });
  
  // Show selected page
  const pageEl = document.getElementById(`${page}-page`);
  if (pageEl) {
    pageEl.classList.add('active');
  }
  
  // Update nav items
  document.querySelectorAll('.nav-item').forEach(item => {
    item.classList.remove('active');
    if (item.dataset.page === page) {
      item.classList.add('active');
    }
  });
  
  // Load page-specific data
  if (page === 'agents' && window.loadAgents) {
    window.loadAgents();
  } else if (page === 'dashboard' && window.loadDashboard) {
    window.loadDashboard();
  } else if (page === 'workflows' && window.loadWorkflows) {
    window.loadWorkflows();
  } else if (page === 'tasks' && window.loadTasks) {
    window.loadTasks();
  } else if (page === 'orchestrator' && window.loadRegisteredAgents) {
    window.loadRegisteredAgents();
  }
}

// Make function available globally
window.navigate = navigate;

// Initialize navigation on page load
document.addEventListener('DOMContentLoaded', () => {
  // Check hash for initial page
  const hash = window.location.hash.slice(1);
  if (hash && ['dashboard', 'agents', 'workflows', 'tasks', 'orchestrator'].includes(hash)) {
    navigate(hash);
  } else {
    navigate('dashboard');
  }
  
  // Update hash on navigation
  const originalNavigate = window.navigate;
  window.navigate = function(page) {
    window.location.hash = page;
    originalNavigate(page);
  };
  
  // Handle hash changes
  window.addEventListener('hashchange', () => {
    const hash = window.location.hash.slice(1);
    if (hash && ['dashboard', 'agents', 'workflows', 'tasks', 'orchestrator'].includes(hash)) {
      navigate(hash);
    }
  });
});

