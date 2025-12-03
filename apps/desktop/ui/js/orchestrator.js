// Orchestrator management functionality

let registeredAgents = [];
let executionResult = null;

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

// Load registered agents from server
async function loadRegisteredAgents() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const loadingEl = document.getElementById('orchestrator-loading');
  const errorEl = document.getElementById('orchestrator-error');
  const tableEl = document.getElementById('registered-agents-table');
  const emptyEl = document.getElementById('registered-agents-empty');

  try {
    loadingEl.style.display = 'block';
    errorEl.style.display = 'none';
    tableEl.style.display = 'none';
    emptyEl.style.display = 'none';

    const result = await invoke('get_registered_agents');
    registeredAgents = JSON.parse(result);

    loadingEl.style.display = 'none';

    if (registeredAgents.length === 0) {
      emptyEl.style.display = 'block';
    } else {
      tableEl.style.display = 'table';
      renderRegisteredAgentsTable();
    }
  } catch (e) {
    loadingEl.style.display = 'none';
    errorEl.style.display = 'block';
    errorEl.textContent = `Failed to load registered agents: ${e}`;
    console.error('Failed to load registered agents:', e);
  }
}

// Render registered agents table
function renderRegisteredAgentsTable() {
  const tbody = document.querySelector('#registered-agents-table tbody');
  tbody.innerHTML = '';

  registeredAgents.forEach(agent => {
    const row = document.createElement('tr');
    row.innerHTML = `
      <td>${escapeHtml(agent.id)}</td>
      <td>${escapeHtml(agent.description || '')}</td>
      <td><span class="badge">${escapeHtml(agent.state || 'idle')}</span></td>
      <td>
        <button class="btn-small" onclick="startAgentLifecycle('${escapeHtml(agent.id)}')">Start</button>
        <button class="btn-small" onclick="stopAgentLifecycle('${escapeHtml(agent.id)}')">Stop</button>
        <button class="btn-small" onclick="showExecuteForm('${escapeHtml(agent.id)}')">Execute</button>
      </td>
    `;
    tbody.appendChild(row);
  });
}

// Show register agent form
function showRegisterForm() {
  document.getElementById('register-form-title').textContent = 'Register Agent';
  document.getElementById('register-agent-id').value = '';
  document.getElementById('register-agent-type').value = 'simple';
  document.getElementById('register-agent-description').value = '';
  document.getElementById('register-agent-form').style.display = 'block';
  document.getElementById('register-agent-form').scrollIntoView({ behavior: 'smooth' });
}

// Register an agent
async function registerAgent() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const agentId = document.getElementById('register-agent-id').value.trim();
  const agentType = document.getElementById('register-agent-type').value;
  const description = document.getElementById('register-agent-description').value.trim();

  if (!agentId) {
    showError('Agent ID is required');
    return;
  }

  const submitBtn = document.getElementById('register-submit-btn');
  const originalText = submitBtn.textContent;

  try {
    submitBtn.disabled = true;
    submitBtn.textContent = 'Registering...';

    await invoke('register_agent', {
      agentId,
      agentType,
      description
    });

    // Hide form and reload agents
    document.getElementById('register-agent-form').style.display = 'none';
    await loadRegisteredAgents();
    showSuccess('Agent registered successfully');
  } catch (e) {
    showError(`Failed to register agent: ${e}`);
    console.error('Failed to register agent:', e);
  } finally {
    submitBtn.disabled = false;
    submitBtn.textContent = originalText;
  }
}

// Start agent lifecycle
async function startAgentLifecycle(agentId) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  try {
    await invoke('start_agent', { agentId });
    await loadRegisteredAgents();
    showSuccess(`Agent "${agentId}" started successfully`);
  } catch (e) {
    showError(`Failed to start agent: ${e}`);
    console.error('Failed to start agent:', e);
  }
}

// Stop agent lifecycle
async function stopAgentLifecycle(agentId) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  try {
    await invoke('stop_agent', { agentId });
    await loadRegisteredAgents();
    showSuccess(`Agent "${agentId}" stopped successfully`);
  } catch (e) {
    showError(`Failed to stop agent: ${e}`);
    console.error('Failed to stop agent:', e);
  }
}

// Show execute agent form
function showExecuteForm(agentId) {
  document.getElementById('execute-form-title').textContent = `Execute Agent: ${escapeHtml(agentId)}`;
  document.getElementById('execute-agent-id').value = agentId;
  document.getElementById('execute-agent-id').disabled = true;
  document.getElementById('execute-input').value = '';
  document.getElementById('execute-model-type').value = '';
  document.getElementById('execute-model-id').value = '';
  document.getElementById('execute-result').style.display = 'none';
  document.getElementById('execute-agent-form').style.display = 'block';
  document.getElementById('execute-agent-form').scrollIntoView({ behavior: 'smooth' });
}

// Execute an agent
async function executeAgent() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const agentId = document.getElementById('execute-agent-id').value.trim();
  const input = document.getElementById('execute-input').value.trim();
  const modelType = document.getElementById('execute-model-type').value.trim() || null;
  const modelId = document.getElementById('execute-model-id').value.trim() || null;

  if (!agentId || !input) {
    showError('Agent ID and input are required');
    return;
  }

  const submitBtn = document.getElementById('execute-submit-btn');
  const originalText = submitBtn.textContent;
  const resultEl = document.getElementById('execute-result');
  const resultContentEl = document.getElementById('execute-result-content');

  try {
    submitBtn.disabled = true;
    submitBtn.textContent = 'Executing...';
    resultEl.style.display = 'none';

    const result = await invoke('execute_agent', {
      agentId,
      input,
      modelType,
      modelId
    });

    const executionResult = JSON.parse(result);

    if (executionResult.success) {
      resultContentEl.innerHTML = `
        <h4>Execution Successful</h4>
        <pre class="result-output">${escapeHtml(executionResult.output || 'No output')}</pre>
      `;
      resultContentEl.className = 'result-content success';
    } else {
      resultContentEl.innerHTML = `
        <h4>Execution Failed</h4>
        <pre class="result-output">${escapeHtml(executionResult.error || 'Unknown error')}</pre>
      `;
      resultContentEl.className = 'result-content error';
    }

    resultEl.style.display = 'block';
    resultEl.scrollIntoView({ behavior: 'smooth' });
  } catch (e) {
    showError(`Failed to execute agent: ${e}`);
    console.error('Failed to execute agent:', e);
  } finally {
    submitBtn.disabled = false;
    submitBtn.textContent = originalText;
  }
}

// Cancel register form
function cancelRegisterForm() {
  document.getElementById('register-agent-form').style.display = 'none';
}

// Cancel execute form
function cancelExecuteForm() {
  document.getElementById('execute-agent-form').style.display = 'none';
  document.getElementById('execute-result').style.display = 'none';
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
window.loadRegisteredAgents = loadRegisteredAgents;
window.showRegisterForm = showRegisterForm;
window.registerAgent = registerAgent;
window.startAgentLifecycle = startAgentLifecycle;
window.stopAgentLifecycle = stopAgentLifecycle;
window.showExecuteForm = showExecuteForm;
window.executeAgent = executeAgent;
window.cancelRegisterForm = cancelRegisterForm;
window.cancelExecuteForm = cancelExecuteForm;

