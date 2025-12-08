// Policy Editor JavaScript

import { invoke } from '@tauri-apps/api/core';

let currentPolicy = {
    approval_mode: 'ask',
    rules: [],
    file_exists: false,
    file_path: ''
};

// Load policy on page load
document.addEventListener('DOMContentLoaded', async () => {
    await loadPolicy();
    updateTOMLPreview();
});

// Load policy from file
async function loadPolicy() {
    try {
        const config = await invoke('get_policy_config');
        currentPolicy = JSON.parse(config);
        renderRules();
        document.getElementById('approval-mode').value = currentPolicy.approval_mode;
        updateTOMLPreview();
        showMessage('Policy loaded successfully', 'success');
    } catch (error) {
        showMessage(`Failed to load policy: ${error}`, 'error');
    }
}

// Save policy to file
async function savePolicy() {
    try {
        const approvalMode = document.getElementById('approval-mode').value;
        const result = await invoke('save_policy_config', {
            approvalMode,
            rules: currentPolicy.rules
        });
        showMessage(result, 'success');
        await loadPolicy(); // Reload to get updated file path
    } catch (error) {
        showMessage(`Failed to save policy: ${error}`, 'error');
    }
}

// Validate policy
async function validatePolicy() {
    try {
        const approvalMode = document.getElementById('approval-mode').value;
        const result = await invoke('validate_policy_config', {
            approvalMode: approvalMode,
            rules: currentPolicy.rules
        });
        showMessage(result, 'success');
    } catch (error) {
        showMessage(`Validation failed: ${error}`, 'error');
    }
}

// Check for conflicts
async function checkConflicts() {
    try {
        const result = await invoke('detect_policy_conflicts');
        const conflicts = JSON.parse(result);
        renderConflicts(conflicts.conflicts);
        if (conflicts.count === 0) {
            showMessage('No conflicts detected', 'success');
        } else {
            showMessage(`Found ${conflicts.count} conflict(s)`, 'error');
        }
    } catch (error) {
        showMessage(`Failed to check conflicts: ${error}`, 'error');
    }
}

// Test policy with a tool
async function testPolicy() {
    const toolName = document.getElementById('test-tool-name').value.trim();
    const argsStr = document.getElementById('test-tool-args').value.trim();
    
    if (!toolName) {
        showMessage('Please enter a tool name', 'error');
        return;
    }
    
    const args = argsStr ? argsStr.split(/\s+/) : [];
    
    try {
        const result = await invoke('check_policy_tool', {
            toolName: toolName,
            args: args
        });
        const decision = JSON.parse(result);
        renderTestResult(decision, toolName, args);
    } catch (error) {
        showMessage(`Failed to test policy: ${error}`, 'error');
    }
}

// Add a new rule
function addRule() {
    const name = document.getElementById('rule-name').value.trim();
    const priority = document.getElementById('rule-priority').value;
    const action = document.getElementById('rule-action').value;
    const toolPattern = document.getElementById('rule-tool-pattern').value.trim();
    const argPattern = document.getElementById('rule-arg-pattern').value.trim();
    const reason = document.getElementById('rule-reason').value.trim();
    
    if (!name || !toolPattern) {
        showMessage('Rule name and tool pattern are required', 'error');
        return;
    }
    
    const rule = {
        name,
        priority,
        action,
        tool_pattern: toolPattern,
        arg_pattern: argPattern || null,
        reason: reason || null
    };
    
    currentPolicy.rules.push(rule);
    renderRules();
    updateTOMLPreview();
    
    // Clear form
    document.getElementById('rule-name').value = '';
    document.getElementById('rule-tool-pattern').value = '';
    document.getElementById('rule-arg-pattern').value = '';
    document.getElementById('rule-reason').value = '';
    
    showMessage('Rule added', 'success');
}

// Remove a rule
function removeRule(index) {
    currentPolicy.rules.splice(index, 1);
    renderRules();
    updateTOMLPreview();
    showMessage('Rule removed', 'success');
}

// Render rules list
function renderRules() {
    const container = document.getElementById('rules-list');
    
    if (currentPolicy.rules.length === 0) {
        container.innerHTML = '<div class="empty-state">No rules configured. Add a rule below.</div>';
        return;
    }
    
    container.innerHTML = currentPolicy.rules.map((rule, index) => `
        <div class="rule-item">
            <div class="rule-item-header">
                <span class="rule-item-name">${escapeHtml(rule.name)}</span>
                <div class="rule-item-actions">
                    <button class="btn btn-small btn-danger" onclick="removeRule(${index})">Remove</button>
                </div>
            </div>
            <div class="rule-item-details">
                <div><strong>Priority:</strong> <span class="badge ${rule.priority}">${rule.priority}</span></div>
                <div><strong>Action:</strong> <span class="badge">${rule.action}</span></div>
                <div><strong>Tool Pattern:</strong> <code>${escapeHtml(rule.tool_pattern)}</code></div>
                ${rule.arg_pattern ? `<div><strong>Arg Pattern:</strong> <code>${escapeHtml(rule.arg_pattern)}</code></div>` : ''}
                ${rule.reason ? `<div><strong>Reason:</strong> ${escapeHtml(rule.reason)}</div>` : ''}
            </div>
        </div>
    `).join('');
}

// Render conflicts
function renderConflicts(conflicts) {
    const container = document.getElementById('conflicts-list');
    
    if (conflicts.length === 0) {
        container.innerHTML = '<div class="empty-state">No conflicts detected</div>';
        return;
    }
    
    container.innerHTML = conflicts.map(conflict => `
        <div class="conflict-item">
            <strong>${escapeHtml(conflict.rule1)}</strong> conflicts with <strong>${escapeHtml(conflict.rule2)}</strong>
            <br><small>Type: ${escapeHtml(conflict.conflict_type)}</small>
        </div>
    `).join('');
}

// Render test result
function renderTestResult(decision, toolName, args) {
    const container = document.getElementById('test-result');
    const argsStr = args.length > 0 ? args.join(' ') : '(none)';
    
    let resultClass = 'ask';
    let resultText = 'Requires Approval';
    let icon = '❓';
    
    if (decision.allowed) {
        resultClass = 'allowed';
        resultText = 'Allowed';
        icon = '✅';
    } else if (decision.denied) {
        resultClass = 'denied';
        resultText = 'Denied';
        icon = '❌';
    }
    
    container.className = `test-result ${resultClass} show`;
    container.innerHTML = `
        <h4>${icon} ${resultText}</h4>
        <div><strong>Tool:</strong> ${escapeHtml(toolName)}</div>
        <div><strong>Arguments:</strong> ${escapeHtml(argsStr)}</div>
        <div><strong>Action:</strong> ${escapeHtml(decision.action)}</div>
        ${decision.matched_rule ? `<div><strong>Matched Rule:</strong> ${escapeHtml(decision.matched_rule)}</div>` : ''}
        ${decision.reason ? `<div><strong>Reason:</strong> ${escapeHtml(decision.reason)}</div>` : ''}
    `;
}

// Update TOML preview
function updateTOMLPreview() {
    const container = document.getElementById('toml-preview');
    const approvalMode = document.getElementById('approval-mode').value;
    
    let toml = `approval_mode = "${approvalMode}"\n\n`;
    
    if (currentPolicy.rules.length === 0) {
        toml += '# No rules configured\n';
    } else {
        toml += '[[rules]]\n';
        currentPolicy.rules.forEach((rule, index) => {
            if (index > 0) {
                toml += '\n[[rules]]\n';
            }
            toml += `name = "${escapeToml(rule.name)}"\n`;
            toml += `priority = "${rule.priority}"\n`;
            toml += `action = "${rule.action}"\n`;
            toml += `tool_pattern = "${escapeToml(rule.tool_pattern)}"\n`;
            if (rule.arg_pattern) {
                toml += `arg_pattern = "${escapeToml(rule.arg_pattern)}"\n`;
            }
            if (rule.reason) {
                toml += `reason = "${escapeToml(rule.reason)}"\n`;
            }
        });
    }
    
    container.textContent = toml;
}

// Set pattern from suggestion
function setPattern(pattern) {
    document.getElementById('rule-tool-pattern').value = pattern;
}

// Show message
function showMessage(text, type) {
    const messageEl = document.getElementById('message');
    messageEl.textContent = text;
    messageEl.className = `message ${type} show`;
    
    setTimeout(() => {
        messageEl.classList.remove('show');
    }, 5000);
}

// Escape HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Escape TOML string
function escapeToml(text) {
    return text.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n');
}

// Make functions available globally
window.loadPolicy = loadPolicy;
window.savePolicy = savePolicy;
window.validatePolicy = validatePolicy;
window.checkConflicts = checkConflicts;
window.testPolicy = testPolicy;
window.addRule = addRule;
window.removeRule = removeRule;
window.setPattern = setPattern;

