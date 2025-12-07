// Extension management functionality

let extensions = [];
let editingExtensionName = null;

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

// Load extensions from manager
async function loadExtensions() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const loadingEl = document.getElementById('extensions-loading');
  const errorEl = document.getElementById('extensions-error');
  const tableEl = document.getElementById('extensions-table');
  const emptyEl = document.getElementById('extensions-empty');

  try {
    loadingEl.style.display = 'block';
    errorEl.style.display = 'none';
    tableEl.style.display = 'none';
    emptyEl.style.display = 'none';

    const result = await invoke('list_extensions');
    extensions = JSON.parse(result);

    loadingEl.style.display = 'none';

    if (extensions.length === 0) {
      emptyEl.style.display = 'block';
    } else {
      tableEl.style.display = 'table';
      renderExtensionsTable();
    }
  } catch (e) {
    loadingEl.style.display = 'none';
    errorEl.style.display = 'block';
    errorEl.textContent = `Failed to load extensions: ${e}`;
    console.error('Failed to load extensions:', e);
  }
}

// Render extensions table
function renderExtensionsTable() {
  const tbody = document.querySelector('#extensions-table tbody');
  tbody.innerHTML = '';

  extensions.forEach(ext => {
    const row = document.createElement('tr');
    const componentCount = ext.components.prompts.length + 
                          ext.components.mcp_servers.length + 
                          ext.components.commands.length + 
                          ext.components.hooks.length;
    
    row.innerHTML = `
      <td>${escapeHtml(ext.name)}</td>
      <td>${escapeHtml(ext.version)}</td>
      <td>${escapeHtml(ext.description || '')}</td>
      <td>${escapeHtml(ext.author || '')}</td>
      <td>${componentCount}</td>
      <td>
        <button class="btn-small" onclick="showExtensionDetails('${escapeHtml(ext.name)}')">Info</button>
        <button class="btn-small btn-danger" onclick="uninstallExtension('${escapeHtml(ext.name)}')">Uninstall</button>
      </td>
    `;
    tbody.appendChild(row);
  });
}

// Show extension details
async function showExtensionDetails(name) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  try {
    const result = await invoke('get_extension_info', { name });
    const extension = JSON.parse(result);

    // Create details modal or show in a panel
    const detailsHtml = `
      <div class="modal" id="extension-details-modal">
        <div class="modal-content">
          <span class="close" onclick="closeExtensionDetails()">&times;</span>
          <h2>Extension: ${escapeHtml(extension.name)}</h2>
          <p><strong>Version:</strong> ${escapeHtml(extension.version)}</p>
          <p><strong>Author:</strong> ${escapeHtml(extension.author)}</p>
          <p><strong>Description:</strong> ${escapeHtml(extension.description)}</p>
          <p><strong>Install Path:</strong> ${escapeHtml(extension.install_path)}</p>
          
          <h3>Components</h3>
          <ul>
            <li>Prompts: ${extension.components.prompts.length}</li>
            <li>MCP Servers: ${extension.components.mcp_servers.length}</li>
            <li>Commands: ${extension.components.commands.length}</li>
            <li>Hooks: ${extension.components.hooks.length}</li>
          </ul>
          
          ${extension.dependencies.length > 0 ? `
            <h3>Dependencies</h3>
            <ul>
              ${extension.dependencies.map(dep => `<li>${escapeHtml(dep)}</li>`).join('')}
            </ul>
          ` : ''}
        </div>
      </div>
    `;

    // Remove existing modal if any
    const existingModal = document.getElementById('extension-details-modal');
    if (existingModal) {
      existingModal.remove();
    }

    document.body.insertAdjacentHTML('beforeend', detailsHtml);
    document.getElementById('extension-details-modal').style.display = 'block';
  } catch (e) {
    showError(`Failed to get extension info: ${e}`);
    console.error('Failed to get extension info:', e);
  }
}

// Close extension details modal
function closeExtensionDetails() {
  const modal = document.getElementById('extension-details-modal');
  if (modal) {
    modal.style.display = 'none';
    modal.remove();
  }
}

// Install extension
async function installExtension() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      multiple: false,
      directory: true,
      title: 'Select Extension Directory'
    });

    if (!selected) {
      return;
    }

    const path = Array.isArray(selected) ? selected[0] : selected;
    
    // Show loading
    const installBtn = document.getElementById('install-extension-btn');
    const originalText = installBtn.textContent;
    installBtn.disabled = true;
    installBtn.textContent = 'Installing...';

    try {
      const result = await invoke('install_extension', { path });
      const extension = JSON.parse(result);
      
      showSuccess(`Extension '${extension.name}' installed successfully`);
      await loadExtensions();
    } catch (e) {
      showError(`Failed to install extension: ${e}`);
    } finally {
      installBtn.disabled = false;
      installBtn.textContent = originalText;
    }
  } catch (e) {
    showError(`Failed to open file dialog: ${e}`);
    console.error('Failed to open file dialog:', e);
  }
}

// Uninstall extension
async function uninstallExtension(name) {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  if (!confirm(`Are you sure you want to uninstall '${name}'?`)) {
    return;
  }

  try {
    await invoke('uninstall_extension', { name });
    showSuccess(`Extension '${name}' uninstalled successfully`);
    await loadExtensions();
  } catch (e) {
    showError(`Failed to uninstall extension: ${e}`);
    console.error('Failed to uninstall extension:', e);
  }
}

// Search extensions
async function searchExtensions() {
  if (!invoke) {
    showError('Tauri API not available');
    return;
  }

  const query = document.getElementById('extension-search').value.trim();
  
  if (!query) {
    await loadExtensions();
    return;
  }

  const loadingEl = document.getElementById('extensions-loading');
  const errorEl = document.getElementById('extensions-error');
  const tableEl = document.getElementById('extensions-table');
  const emptyEl = document.getElementById('extensions-empty');

  try {
    loadingEl.style.display = 'block';
    errorEl.style.display = 'none';
    tableEl.style.display = 'none';
    emptyEl.style.display = 'none';

    const result = await invoke('search_extensions', { query });
    extensions = JSON.parse(result);

    loadingEl.style.display = 'none';

    if (extensions.length === 0) {
      emptyEl.style.display = 'block';
      emptyEl.textContent = `No extensions found matching '${query}'`;
    } else {
      tableEl.style.display = 'table';
      renderExtensionsTable();
    }
  } catch (e) {
    loadingEl.style.display = 'none';
    errorEl.style.display = 'block';
    errorEl.textContent = `Failed to search extensions: ${e}`;
    console.error('Failed to search extensions:', e);
  }
}

// Helper functions
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function showError(message) {
  // You can implement a toast notification system here
  alert(`Error: ${message}`);
}

function showSuccess(message) {
  // You can implement a toast notification system here
  alert(`Success: ${message}`);
}

// Make functions available globally
window.loadExtensions = loadExtensions;
window.showExtensionDetails = showExtensionDetails;
window.closeExtensionDetails = closeExtensionDetails;
window.installExtension = installExtension;
window.uninstallExtension = uninstallExtension;
window.searchExtensions = searchExtensions;

// Auto-load extensions on page load
document.addEventListener('DOMContentLoaded', () => {
  if (window.location.hash === '#extensions' || document.getElementById('extensions-page')?.classList.contains('active')) {
    loadExtensions();
  }
});

