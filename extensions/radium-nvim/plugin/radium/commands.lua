-- Radium Neovim plugin commands
-- Core commands: RadiumSendSelection, RadiumChat, RadiumApplyBlock

local utils = require('radium.utils')

local M = {}

--- Send visual selection to Radium for processing
function M.send_selection()
    -- Check if we have a visual selection
    local mode = vim.fn.mode()
    if mode ~= 'v' and mode ~= 'V' and mode ~= '' then
        vim.notify("No visual selection. Select code first with visual mode.", vim.log.levels.WARN)
        return
    end
    
    -- Get context
    local context = utils.get_file_context()
    
    if not context.selection or context.selection == "" then
        vim.notify("Selection is empty.", vim.log.levels.WARN)
        return
    end
    
    -- Default agent ID (can be configured)
    local agent_id = vim.g.radium_default_agent or "code-agent"
    
    vim.notify("Sending selection to Radium...", vim.log.levels.INFO)
    
    -- Execute rad step command
    utils.execute_radium_command("step " .. agent_id, context, function(output, error)
        if error and error ~= "" then
            vim.notify("Error: " .. error, vim.log.levels.ERROR)
            return
        end
        
        if output then
            -- Store output for apply command
            utils.store_agent_output(output)
            
            -- Display in a split window
            local buf = vim.api.nvim_create_buf(false, true)
            local win = vim.api.nvim_open_win(buf, true, {
                relative = 'editor',
                width = math.floor(vim.o.columns * 0.8),
                height = math.floor(vim.o.lines * 0.6),
                col = math.floor(vim.o.columns * 0.1),
                row = math.floor(vim.o.lines * 0.2),
                border = 'single',
                title = 'Radium Output'
            })
            
            vim.api.nvim_buf_set_lines(buf, 0, -1, false, vim.split(output, '\n'))
            vim.api.nvim_buf_set_option(buf, 'filetype', 'markdown')
            vim.api.nvim_buf_set_option(buf, 'readonly', true)
            
            vim.notify("Radium processing complete. Use :RadiumApplyBlock to apply code.", vim.log.levels.INFO)
        end
    end)
end

--- Open interactive chat session with Radium
function M.chat()
    local agent_id = vim.g.radium_default_agent or "code-agent"
    
    -- Create terminal buffer for chat
    vim.cmd('belowright split')
    local buf = vim.api.nvim_create_buf(false, true)
    local win = vim.api.nvim_get_current_win()
    vim.api.nvim_win_set_buf(win, buf)
    
    -- Start rad chat command in terminal
    vim.fn.termopen('rad chat ' .. agent_id, {
        env = {
            RADIUM_EDITOR_FILE_PATH = vim.fn.expand('%:p'),
            RADIUM_EDITOR_LANGUAGE = vim.bo.filetype,
        }
    })
    
    vim.cmd('startinsert')
    vim.notify("Radium chat session started. Type your messages and press Enter.", vim.log.levels.INFO)
end

--- Apply code block from last agent output
function M.apply_block()
    local output = utils.get_last_agent_output()
    
    if not output or output == "" then
        vim.notify("No agent output found. Run :RadiumSendSelection first.", vim.log.levels.WARN)
        return
    end
    
    -- Parse code blocks from markdown (simple regex-based extraction)
    local code_blocks = {}
    for code_block in output:gmatch("```%w*\n(.-)```") do
        table.insert(code_blocks, code_block)
    end
    
    if #code_blocks == 0 then
        vim.notify("No code blocks found in agent output.", vim.log.levels.WARN)
        return
    end
    
    -- For now, use first code block (Task 4 will add diff preview and selection)
    local selected_code = code_blocks[1]
    
    -- Get current buffer lines
    local current_lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
    local current_content = table.concat(current_lines, "\n")
    
    -- Show diff preview (simple implementation - Task 4 will enhance)
    vim.notify("Applying code block...", vim.log.levels.INFO)
    
    -- Replace selection or append at cursor
    local mode = vim.fn.mode()
    if mode == 'v' or mode == 'V' or mode == '' then
        -- Replace selection
        local start_pos = vim.fn.getpos("'<")
        local end_pos = vim.fn.getpos("'>")
        local start_line = start_pos[2] - 1  -- 0-indexed
        local end_line = end_pos[2]  -- 1-indexed for end
        
        -- Replace lines
        local new_lines = vim.split(selected_code, '\n')
        vim.api.nvim_buf_set_lines(0, start_line, end_line, false, new_lines)
    else
        -- Insert at cursor
        local cursor = vim.api.nvim_win_get_cursor(0)
        local new_lines = vim.split(selected_code, '\n')
        vim.api.nvim_buf_set_lines(0, cursor[1], cursor[1], false, new_lines)
    end
    
    vim.notify("Code applied successfully.", vim.log.levels.INFO)
end

return M

