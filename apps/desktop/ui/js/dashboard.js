// Dashboard functionality

let invoke;
(async () => {
  try {
    const tauri = await import('@tauri-apps/api/core');
    invoke = tauri.invoke;
    console.log('Tauri API loaded');
  } catch (e) {
    console.error('Failed to load Tauri API:', e);
    invoke = null;
  }
})();

// Load dashboard data
async function loadDashboard() {
  if (!invoke) {
    return;
  }

  // Update connection status
  await updateConnectionStatus();

  // Load counts
  try {
    // Load agents count
    try {
      const agentsResult = await invoke('list_agents');
      const agents = JSON.parse(agentsResult);
      document.getElementById('dashboard-agents-count').textContent = agents.length;
    } catch (e) {
      document.getElementById('dashboard-agents-count').textContent = '-';
    }

    // Load workflows count
    try {
      const workflowsResult = await invoke('list_workflows');
      const workflows = JSON.parse(workflowsResult);
      document.getElementById('dashboard-workflows-count').textContent = workflows.length;
    } catch (e) {
      document.getElementById('dashboard-workflows-count').textContent = '-';
    }

    // Load tasks count
    try {
      const tasksResult = await invoke('list_tasks');
      const tasks = JSON.parse(tasksResult);
      document.getElementById('dashboard-tasks-count').textContent = tasks.length;
    } catch (e) {
      document.getElementById('dashboard-tasks-count').textContent = '-';
    }

    // Load registered agents count
    try {
      const registeredAgentsResult = await invoke('get_registered_agents');
      const registeredAgents = JSON.parse(registeredAgentsResult);
      document.getElementById('dashboard-registered-agents-count').textContent = registeredAgents.length;
    } catch (e) {
      document.getElementById('dashboard-registered-agents-count').textContent = '-';
    }
  } catch (e) {
    console.error('Failed to load dashboard data:', e);
  }
}

// Update connection status
async function updateConnectionStatus() {
  if (!invoke) {
    return;
  }

  const statusDot = document.getElementById('status-dot');
  const statusText = document.getElementById('status-text');

  try {
    await invoke('ping_server', { message: 'status-check' });
    statusDot.classList.add('connected');
    statusText.textContent = 'Connected';
  } catch (e) {
    statusDot.classList.remove('connected');
    statusText.textContent = 'Disconnected';
  }
}

// Make functions available globally
window.loadDashboard = loadDashboard;
window.updateConnectionStatus = updateConnectionStatus;

// Auto-update connection status periodically
setInterval(() => {
  if (document.getElementById('dashboard-page').classList.contains('active')) {
    updateConnectionStatus();
  }
}, 30000); // Every 30 seconds

