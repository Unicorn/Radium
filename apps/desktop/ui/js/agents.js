// Agent management functionality

let agents = [];
let editingAgentId = null;

// Initialize Tauri API
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

// Load agents from server
async function loadAgents() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const loadingEl = document.getElementById('agents-loading');
  const errorEl = document.getElementById('agents-error');
  const tableEl = document.getElementById('agents-table');
  const emptyEl = document.getElementById('agents-empty');

  try {
    loadingEl.style.display = 'block';
    errorEl.style.display = 'none';
    tableEl.style.display = 'none';
    emptyEl.style.display = 'none';

    const result = await invoke('list_agents');
    agents = JSON.parse(result);

    loadingEl.style.display = 'none';

    if (agents.length === 0) {
      emptyEl.style.display = 'block';
    } else {
      tableEl.style.display = 'table';
      renderAgentsTable();
    }
  } catch (e) {
    loadingEl.style.display = 'none';
    errorEl.style.display = 'block';
    errorEl.textContent = `Failed to load agents: ${e}`;
    console.error('Failed to load agents:', e);
  }
}

// Render agents table
function renderAgentsTable() {
  const tbody = document.querySelector('#agents-table tbody');
  tbody.innerHTML = '';

  agents.forEach(agent => {
    const row = document.createElement('tr');
    row.innerHTML = `
      <td>${escapeHtml(agent.id)}</td>
      <td>${escapeHtml(agent.name)}</td>
      <td>${escapeHtml(agent.description || '')}</td>
      <td><span class="badge">${escapeHtml(agent.state || 'idle')}</span></td>
      <td>
        <button class="btn-small" onclick="editAgent('${escapeHtml(agent.id)}')">Edit</button>
        <button class="btn-small btn-danger" onclick="deleteAgent('${escapeHtml(agent.id)}')">Delete</button>
      </td>
    `;
    tbody.appendChild(row);
  });
}

// Show create agent form
function showCreateForm() {
  editingAgentId = null;
  document.getElementById('agent-form-title').textContent = 'Create Agent';
  document.getElementById('agent-id').value = '';
  document.getElementById('agent-name').value = '';
  document.getElementById('agent-description').value = '';
  document.getElementById('agent-form').style.display = 'block';
  document.getElementById('agent-form').scrollIntoView({ behavior: 'smooth' });
}

// Edit agent
async function editAgent(agentId) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  try {
    const result = await invoke('get_agent', { agentId });
    const agent = JSON.parse(result);

    editingAgentId = agentId;
    document.getElementById('agent-form-title').textContent = 'Edit Agent';
    document.getElementById('agent-id').value = agent.id;
    document.getElementById('agent-id').disabled = true;
    document.getElementById('agent-name').value = agent.name;
    document.getElementById('agent-description').value = agent.description;
    document.getElementById('agent-form').style.display = 'block';
    document.getElementById('agent-form').scrollIntoView({ behavior: 'smooth' });
  } catch (e) {
    showError(`Failed to load agent: ${e}`);
    console.error('Failed to load agent:', e);
  }
}

// Save agent (create or update)
async function saveAgent() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const id = document.getElementById('agent-id').value.trim();
  const name = document.getElementById('agent-name').value.trim();
  const description = document.getElementById('agent-description').value.trim();

  if (!id || !name) {
    showError('ID and name are required');
    return;
  }

  const submitBtn = document.getElementById('agent-submit-btn');
  const originalText = submitBtn.textContent;

  try {
    submitBtn.disabled = true;
    submitBtn.textContent = 'Saving...';

    if (editingAgentId) {
      // Update
      await invoke('update_agent', {
        id: editingAgentId,
        name: name || null,
        description: description || null
      });
    } else {
      // Create
      await invoke('create_agent', {
        id,
        name,
        description
      });
    }

    // Hide form and reload agents
    document.getElementById('agent-form').style.display = 'none';
    await loadAgents();
    showSuccess(editingAgentId ? 'Agent updated successfully' : 'Agent created successfully');
  } catch (e) {
    showError(`Failed to save agent: ${e}`);
    console.error('Failed to save agent:', e);
  } finally {
    submitBtn.disabled = false;
    submitBtn.textContent = originalText;
  }
}

// Delete agent
async function deleteAgent(agentId) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  if (!confirm(`Are you sure you want to delete agent "${agentId}"?`)) {
    return;
  }

  try {
    await invoke('delete_agent', { agentId });
    await loadAgents();
    showSuccess('Agent deleted successfully');
  } catch (e) {
    showError(`Failed to delete agent: ${e}`);
    console.error('Failed to delete agent:', e);
  }
}

// Cancel form
function cancelForm() {
  document.getElementById('agent-form').style.display = 'none';
  editingAgentId = null;
}

// Utility functions
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function showError(message) {
  const errorEl = document.getElementById('global-error');
  if (errorEl) {
    errorEl.textContent = message;
    errorEl.style.display = 'block';
    setTimeout(() => {
      errorEl.style.display = 'none';
    }, 5000);
  } else {
    alert(message);
  }
}

function showSuccess(message) {
  const successEl = document.getElementById('global-success');
  if (successEl) {
    successEl.textContent = message;
    successEl.style.display = 'block';
    setTimeout(() => {
      successEl.style.display = 'none';
    }, 3000);
  }
}

// Make functions available globally
window.loadAgents = loadAgents;
window.showCreateForm = showCreateForm;
window.editAgent = editAgent;
window.saveAgent = saveAgent;
window.deleteAgent = deleteAgent;
window.cancelForm = cancelForm;

