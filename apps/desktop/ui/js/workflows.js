// Workflow management functionality

let workflows = [];
let editingWorkflowId = null;

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

// Load workflows from server
async function loadWorkflows() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const loadingEl = document.getElementById('workflows-loading');
  const errorEl = document.getElementById('workflows-error');
  const tableEl = document.getElementById('workflows-table');
  const emptyEl = document.getElementById('workflows-empty');

  try {
    loadingEl.style.display = 'block';
    errorEl.style.display = 'none';
    tableEl.style.display = 'none';
    emptyEl.style.display = 'none';

    const result = await invoke('list_workflows');
    workflows = JSON.parse(result);

    loadingEl.style.display = 'none';

    if (workflows.length === 0) {
      emptyEl.style.display = 'block';
    } else {
      tableEl.style.display = 'table';
      renderWorkflowsTable();
    }
  } catch (e) {
    loadingEl.style.display = 'none';
    errorEl.style.display = 'block';
    errorEl.textContent = `Failed to load workflows: ${e}`;
    console.error('Failed to load workflows:', e);
  }
}

// Render workflows table
function renderWorkflowsTable() {
  const tbody = document.querySelector('#workflows-table tbody');
  tbody.innerHTML = '';

  workflows.forEach(workflow => {
    const row = document.createElement('tr');
    row.innerHTML = `
      <td>${escapeHtml(workflow.id)}</td>
      <td>${escapeHtml(workflow.name)}</td>
      <td>${escapeHtml(workflow.description || '')}</td>
      <td>${workflow.steps.length}</td>
      <td><span class="badge">${escapeHtml(workflow.state || 'idle')}</span></td>
      <td>
        <button class="btn-small" onclick="editWorkflow('${escapeHtml(workflow.id)}')">Edit</button>
        <button class="btn-small" onclick="executeWorkflow('${escapeHtml(workflow.id)}')">Execute</button>
        <button class="btn-small btn-danger" onclick="deleteWorkflow('${escapeHtml(workflow.id)}')">Delete</button>
      </td>
    `;
    tbody.appendChild(row);
  });
}

// Show create workflow form
function showCreateWorkflowForm() {
  editingWorkflowId = null;
  document.getElementById('workflow-form-title').textContent = 'Create Workflow';
  document.getElementById('workflow-id').value = '';
  document.getElementById('workflow-id').disabled = false;
  document.getElementById('workflow-name').value = '';
  document.getElementById('workflow-description').value = '';
  document.getElementById('workflow-form').style.display = 'block';
  document.getElementById('workflow-form').scrollIntoView({ behavior: 'smooth' });
}

// Edit workflow
async function editWorkflow(workflowId) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  try {
    const result = await invoke('get_workflow', { workflowId });
    const workflow = JSON.parse(result);

    editingWorkflowId = workflowId;
    document.getElementById('workflow-form-title').textContent = 'Edit Workflow';
    document.getElementById('workflow-id').value = workflow.id;
    document.getElementById('workflow-id').disabled = true;
    document.getElementById('workflow-name').value = workflow.name;
    document.getElementById('workflow-description').value = workflow.description;
    document.getElementById('workflow-form').style.display = 'block';
    document.getElementById('workflow-form').scrollIntoView({ behavior: 'smooth' });
  } catch (e) {
    showError(`Failed to load workflow: ${e}`);
    console.error('Failed to load workflow:', e);
  }
}

// Save workflow (create or update)
async function saveWorkflow() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const id = document.getElementById('workflow-id').value.trim();
  const name = document.getElementById('workflow-name').value.trim();
  const description = document.getElementById('workflow-description').value.trim();

  if (!id || !name) {
    showError('ID and name are required');
    return;
  }

  const submitBtn = document.getElementById('workflow-submit-btn');
  const originalText = submitBtn.textContent;

  try {
    submitBtn.disabled = true;
    submitBtn.textContent = 'Saving...';

    if (editingWorkflowId) {
      // Update
      await invoke('update_workflow', {
        id: editingWorkflowId,
        name: name || null,
        description: description || null
      });
    } else {
      // Create
      await invoke('create_workflow', {
        id,
        name,
        description
      });
    }

    // Hide form and reload workflows
    document.getElementById('workflow-form').style.display = 'none';
    await loadWorkflows();
    showSuccess(editingWorkflowId ? 'Workflow updated successfully' : 'Workflow created successfully');
  } catch (e) {
    showError(`Failed to save workflow: ${e}`);
    console.error('Failed to save workflow:', e);
  } finally {
    submitBtn.disabled = false;
    submitBtn.textContent = originalText;
  }
}

// Execute workflow
async function executeWorkflow(workflowId) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  if (!confirm(`Execute workflow "${workflowId}"?`)) {
    return;
  }

  const useParallel = confirm('Execute steps in parallel when possible?');

  try {
    const result = await invoke('execute_workflow', {
      workflowId,
      useParallel
    });
    const execution = JSON.parse(result);

    if (execution.success) {
      showSuccess(`Workflow executed successfully! Execution ID: ${execution.execution_id}`);
      await loadWorkflows();
    } else {
      showError(`Workflow execution failed: ${execution.error || 'Unknown error'}`);
    }
  } catch (e) {
    showError(`Failed to execute workflow: ${e}`);
    console.error('Failed to execute workflow:', e);
  }
}

// Delete workflow
async function deleteWorkflow(workflowId) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  if (!confirm(`Are you sure you want to delete workflow "${workflowId}"?`)) {
    return;
  }

  try {
    await invoke('delete_workflow', { workflowId });
    await loadWorkflows();
    showSuccess('Workflow deleted successfully');
  } catch (e) {
    showError(`Failed to delete workflow: ${e}`);
    console.error('Failed to delete workflow:', e);
  }
}

// Cancel form
function cancelWorkflowForm() {
  document.getElementById('workflow-form').style.display = 'none';
  editingWorkflowId = null;
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
window.loadWorkflows = loadWorkflows;
window.showCreateWorkflowForm = showCreateWorkflowForm;
window.editWorkflow = editWorkflow;
window.saveWorkflow = saveWorkflow;
window.executeWorkflow = executeWorkflow;
window.deleteWorkflow = deleteWorkflow;
window.cancelWorkflowForm = cancelWorkflowForm;

