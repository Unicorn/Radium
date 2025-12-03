// Task viewer functionality

let tasks = [];
let selectedTask = null;

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

// Load tasks from server
async function loadTasks() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const loadingEl = document.getElementById('tasks-loading');
  const errorEl = document.getElementById('tasks-error');
  const tableEl = document.getElementById('tasks-table');
  const emptyEl = document.getElementById('tasks-empty');

  try {
    loadingEl.style.display = 'block';
    errorEl.style.display = 'none';
    tableEl.style.display = 'none';
    emptyEl.style.display = 'none';

    const result = await invoke('list_tasks');
    tasks = JSON.parse(result);

    loadingEl.style.display = 'none';

    if (tasks.length === 0) {
      emptyEl.style.display = 'block';
    } else {
      tableEl.style.display = 'table';
      renderTasksTable();
    }
  } catch (e) {
    loadingEl.style.display = 'none';
    errorEl.style.display = 'block';
    errorEl.textContent = `Failed to load tasks: ${e}`;
    console.error('Failed to load tasks:', e);
  }
}

// Render tasks table
function renderTasksTable() {
  const tbody = document.querySelector('#tasks-table tbody');
  tbody.innerHTML = '';

  tasks.forEach(task => {
    const row = document.createElement('tr');
    row.style.cursor = 'pointer';
    row.onclick = () => showTaskDetails(task.id);
    row.innerHTML = `
      <td>${escapeHtml(task.id)}</td>
      <td>${escapeHtml(task.name)}</td>
      <td>${escapeHtml(task.agent_id || '')}</td>
      <td><span class="badge">${escapeHtml(task.state || 'pending')}</span></td>
      <td>${escapeHtml(new Date(task.created_at).toLocaleString())}</td>
      <td>
        <button class="btn-small" onclick="event.stopPropagation(); showTaskDetails('${escapeHtml(task.id)}')">View</button>
      </td>
    `;
    tbody.appendChild(row);
  });
}

// Show task details
async function showTaskDetails(taskId) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  try {
    const result = await invoke('get_task', { taskId });
    const task = JSON.parse(result);
    selectedTask = task;

    // Populate detail view
    document.getElementById('task-detail-id').textContent = task.id;
    document.getElementById('task-detail-name').textContent = task.name;
    document.getElementById('task-detail-description').textContent = task.description || 'N/A';
    document.getElementById('task-detail-agent-id').textContent = task.agent_id || 'N/A';
    const stateEl = document.getElementById('task-detail-state');
    stateEl.innerHTML = `<span class="badge">${escapeHtml(task.state || 'pending')}</span>`;
    document.getElementById('task-detail-input').textContent = task.input_json || '{}';
    document.getElementById('task-detail-result').textContent = task.result_json || '{}';
    document.getElementById('task-detail-created').textContent = new Date(task.created_at).toLocaleString();
    document.getElementById('task-detail-updated').textContent = new Date(task.updated_at).toLocaleString();

    // Show detail modal
    document.getElementById('task-detail-modal').style.display = 'block';
  } catch (e) {
    showError(`Failed to load task: ${e}`);
    console.error('Failed to load task:', e);
  }
}

// Close task detail modal
function closeTaskDetail() {
  document.getElementById('task-detail-modal').style.display = 'none';
  selectedTask = null;
}

// Filter tasks
function filterTasks() {
  const filterState = document.getElementById('task-filter-state').value;
  const filterAgent = document.getElementById('task-filter-agent').value.toLowerCase();

  const tbody = document.querySelector('#tasks-table tbody');
  tbody.innerHTML = '';

  const filtered = tasks.filter(task => {
    const stateMatch = !filterState || task.state === filterState || task.state?.includes(filterState);
    const agentMatch = !filterAgent || (task.agent_id || '').toLowerCase().includes(filterAgent);
    return stateMatch && agentMatch;
  });

  if (filtered.length === 0) {
    document.getElementById('tasks-empty').style.display = 'block';
    document.getElementById('tasks-table').style.display = 'none';
  } else {
    document.getElementById('tasks-empty').style.display = 'none';
    document.getElementById('tasks-table').style.display = 'table';
    
    filtered.forEach(task => {
      const row = document.createElement('tr');
      row.style.cursor = 'pointer';
      row.onclick = () => showTaskDetails(task.id);
      row.innerHTML = `
        <td>${escapeHtml(task.id)}</td>
        <td>${escapeHtml(task.name)}</td>
        <td>${escapeHtml(task.agent_id || '')}</td>
        <td><span class="badge">${escapeHtml(task.state || 'pending')}</span></td>
        <td>${escapeHtml(new Date(task.created_at).toLocaleString())}</td>
        <td>
          <button class="btn-small" onclick="event.stopPropagation(); showTaskDetails('${escapeHtml(task.id)}')">View</button>
        </td>
      `;
      tbody.appendChild(row);
    });
  }
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

// Make functions available globally
window.loadTasks = loadTasks;
window.showTaskDetails = showTaskDetails;
window.closeTaskDetail = closeTaskDetail;
window.filterTasks = filterTasks;

