// Complete Task functionality

let invoke;
(async () => {
  try {
    const tauri = await import('@tauri-apps/api/core');
    const { listen } = await import('@tauri-apps/api/event');
    invoke = tauri.invoke;
    
    // Listen for progress events
    listen('complete-progress', (event) => {
      handleProgressEvent(event.payload);
    });
    
    console.log('Complete Task module loaded');
  } catch (e) {
    console.error('Failed to load Tauri API:', e);
    invoke = null;
  }
})();

function showCompleteTaskModal() {
  const modal = document.getElementById('complete-task-modal');
  modal.style.display = 'block';
  document.getElementById('complete-task-source').value = '';
  document.getElementById('complete-task-progress').style.display = 'none';
  document.getElementById('complete-task-result').style.display = 'none';
  document.getElementById('complete-task-progress-content').textContent = '';
  document.getElementById('complete-task-submit-btn').disabled = false;
}

function closeCompleteTaskModal() {
  const modal = document.getElementById('complete-task-modal');
  modal.style.display = 'none';
}

async function executeCompleteTask() {
  if (!invoke) {
    alert('Tauri API not available');
    return;
  }

  const source = document.getElementById('complete-task-source').value.trim();
  if (!source) {
    alert('Please enter a source');
    return;
  }

  const submitBtn = document.getElementById('complete-task-submit-btn');
  const progressDiv = document.getElementById('complete-task-progress');
  const resultDiv = document.getElementById('complete-task-result');
  const progressContent = document.getElementById('complete-task-progress-content');
  const resultContent = document.getElementById('complete-task-result-content');
  const resultText = document.getElementById('complete-task-result-text');

  // Reset UI
  submitBtn.disabled = true;
  progressDiv.style.display = 'block';
  resultDiv.style.display = 'none';
  progressContent.textContent = 'Starting completion workflow...\n';

  try {
    // Execute completion workflow
    const result = await invoke('complete_task', { source });
    const event = JSON.parse(result);
    
    // Handle final result
    if (event.type === 'Completed') {
      resultDiv.style.display = 'block';
      resultContent.className = 'result-content success';
      resultText.textContent = '✓ Completion workflow finished successfully!';
    } else if (event.type === 'Error') {
      resultDiv.style.display = 'block';
      resultContent.className = 'result-content error';
      resultText.textContent = `✗ Error: ${event.message || 'Unknown error'}`;
    }
  } catch (e) {
    resultDiv.style.display = 'block';
    resultContent.className = 'result-content error';
    resultText.textContent = `✗ Failed to execute: ${e}`;
  } finally {
    submitBtn.disabled = false;
  }
}

function handleProgressEvent(eventData) {
  const progressContent = document.getElementById('complete-task-progress-content');
  if (!progressContent) return;

  let event;
  try {
    event = typeof eventData === 'string' ? JSON.parse(eventData) : eventData;
  } catch (e) {
    console.error('Failed to parse event:', e);
    return;
  }

  let message = '';
  switch (event.type) {
    case 'Detected':
      message = `ℹ️  Detected source: ${event.source_type || 'Unknown'}\n`;
      break;
    case 'Fetching':
      message = '⬇️  Fetching requirements...\n';
      break;
    case 'Planning':
      message = '🧠 Generating plan...\n';
      break;
    case 'PlanGenerated':
      message = `✓ Generated plan with ${event.iterations || 0} iterations, ${event.tasks || 0} tasks\n`;
      break;
    case 'PlanPersisted':
      message = `✓ Plan saved to: ${event.path || 'Unknown'}\n`;
      break;
    case 'ExecutionStarted':
      message = `\n🚀 Executing ${event.total_tasks || 0} tasks...\n\n`;
      break;
    case 'TaskProgress':
      message = `  → Task ${event.current || 0}/${event.total || 0}: ${event.task_name || 'Unknown'}\n`;
      break;
    case 'TaskCompleted':
      message = `    ✓ Completed: ${event.task_name || 'Unknown'}\n`;
      break;
    case 'Completed':
      message = '\n✅ Completion workflow finished successfully!\n';
      break;
    case 'Error':
      message = `\n❌ Error: ${event.message || 'Unknown error'}\n`;
      break;
    default:
      message = `[${event.type || 'Unknown'}]\n`;
  }

  progressContent.textContent += message;
  progressContent.scrollTop = progressContent.scrollHeight;
}

// Make functions available globally
window.showCompleteTaskModal = showCompleteTaskModal;
window.closeCompleteTaskModal = closeCompleteTaskModal;
window.executeCompleteTask = executeCompleteTask;

