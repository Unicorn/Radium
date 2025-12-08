-- Radium Neovim Plugin
-- Main entry point for Radium editor integration

local commands = require('radium.commands')

-- Register commands
vim.api.nvim_create_user_command('RadiumSendSelection', function()
    commands.send_selection()
end, {
    desc = 'Send visual selection to Radium for processing'
})

vim.api.nvim_create_user_command('RadiumChat', function()
    commands.chat()
end, {
    desc = 'Open interactive chat session with Radium agent'
})

vim.api.nvim_create_user_command('RadiumApplyBlock', function()
    commands.apply_block()
end, {
    desc = 'Apply last agent-generated code block to buffer'
})

-- Default configuration
vim.g.radium_default_agent = vim.g.radium_default_agent or "code-agent"

