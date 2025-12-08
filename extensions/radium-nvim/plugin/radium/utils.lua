-- Radium Neovim plugin utilities
-- Helper functions for context extraction and CLI communication

local M = {}

--- Get file context for Radium requests
-- @return table with file_path, language, selection, surrounding_lines
function M.get_file_context()
    local file_path = vim.fn.expand('%:p')
    local language = vim.bo.filetype
    local selection = M.get_visual_selection()
    local surrounding_lines = M.get_surrounding_lines()
    
    return {
        file_path = file_path,
        language = language,
        selection = selection,
        surrounding_lines = surrounding_lines
    }
end

--- Get visual selection text
-- @return string selected text or empty string
function M.get_visual_selection()
    local mode = vim.fn.mode()
    if mode ~= 'v' and mode ~= 'V' and mode ~= '' then
        return ""
    end
    
    local start_pos = vim.fn.getpos("'<")
    local end_pos = vim.fn.getpos("'>")
    local start_line = start_pos[2]
    local end_line = end_pos[2]
    local start_col = start_pos[3]
    local end_col = end_pos[3]
    
    local lines = {}
    if mode == 'V' then
        -- Linewise selection
        for i = start_line, end_line do
            table.insert(lines, vim.fn.getline(i))
        end
    else
        -- Characterwise or blockwise
        for i = start_line, end_line do
            local line = vim.fn.getline(i)
            if i == start_line and i == end_line then
                line = string.sub(line, start_col, end_col)
            elseif i == start_line then
                line = string.sub(line, start_col)
            elseif i == end_line then
                line = string.sub(line, 1, end_col)
            end
            table.insert(lines, line)
        end
    end
    
    return table.concat(lines, "\n")
end

--- Get surrounding lines around selection for context
-- @param context_lines number of lines before and after (default: 3)
-- @return string surrounding lines
function M.get_surrounding_lines(context_lines)
    context_lines = context_lines or 3
    
    local mode = vim.fn.mode()
    local start_line = vim.fn.line("'<")
    local end_line = vim.fn.line("'>")
    
    -- Get lines before selection
    local before_start = math.max(1, start_line - context_lines)
    local before_lines = {}
    for i = before_start, start_line - 1 do
        table.insert(before_lines, vim.fn.getline(i))
    end
    
    -- Get lines after selection
    local after_end = math.min(vim.fn.line('$'), end_line + context_lines)
    local after_lines = {}
    for i = end_line + 1, after_end do
        table.insert(after_lines, vim.fn.getline(i))
    end
    
    return table.concat(before_lines, "\n") .. "\n---\n" .. table.concat(after_lines, "\n")
end

--- Execute Radium CLI command with context
-- @param command string CLI command (e.g., "step code-agent")
-- @param context table Editor context
-- @param callback function Callback function with (output, error)
function M.execute_radium_command(command, context, callback)
    -- Format context as JSON
    local context_json = vim.json.encode(context)
    
    -- Set environment variables for hook
    local env = {
        RADIUM_EDITOR_FILE_PATH = context.file_path or "",
        RADIUM_EDITOR_LANGUAGE = context.language or "",
        RADIUM_EDITOR_SELECTION = context.selection or "",
        RADIUM_EDITOR_SURROUNDING_LINES = context.surrounding_lines or ""
    }
    
    -- Build command with context in stdin
    local full_command = string.format('echo %s | rad %s', 
        vim.fn.shellescape(context_json), 
        command
    )
    
    -- Execute command
    vim.fn.jobstart(full_command, {
        env = env,
        stdout_buffered = true,
        stderr_buffered = true,
        on_stdout = function(_, data)
            local output = table.concat(data, "\n")
            if callback then
                callback(output, nil)
            end
        end,
        on_stderr = function(_, data)
            local error = table.concat(data, "\n")
            if callback then
                callback(nil, error)
            end
        end
    })
end

--- Store agent output for later retrieval
-- @param output string Agent output
function M.store_agent_output(output)
    vim.b.radium_last_output = output
end

--- Get last agent output
-- @return string|nil Last agent output
function M.get_last_agent_output()
    return vim.b.radium_last_output
end

return M

