-- Radium Neovim plugin diff utilities
-- Functions for parsing code blocks and showing diff previews

local M = {}

--- Parse markdown code blocks from agent output
-- @param output string Agent output text
-- @return table Array of code blocks with metadata
function M.parse_code_blocks(output)
    local code_blocks = {}
    
    -- Pattern to match markdown code blocks: ```lang\ncode\n```
    -- Captures language and code content
    for lang, code in output:gmatch("```(%w*)\n(.-)```") do
        table.insert(code_blocks, {
            language = lang ~= "" and lang or nil,
            content = code:match("^%s*(.+%s*)$"), -- trim
            raw = code
        })
    end
    
    return code_blocks
end

--- Generate diff between current buffer and new code
-- @param current_content string Current buffer content
-- @param new_content string New code content
-- @return string Diff text
function M.generate_diff(current_content, new_content)
    -- Use Neovim's built-in diff functionality (available in 0.5+)
    local original_lines = vim.split(current_content, '\n')
    local new_lines = vim.split(new_content, '\n')
    
    -- Generate diff using vim.diff if available, otherwise use simple comparison
    if vim.diff then
        local diff_result = vim.diff(
            table.concat(original_lines, '\n'),
            table.concat(new_lines, '\n'),
            {
                result_type = 'unified',
                context = 3,
                on_hunk = function() end
            }
        )
        return diff_result
    else
        -- Fallback: simple line-by-line comparison
        local diff_lines = {"--- Original", "+++ New", "@@ -1," .. #original_lines .. " +1," .. #new_lines .. " @@"}
        local max_lines = math.max(#original_lines, #new_lines)
        for i = 1, max_lines do
            local old_line = original_lines[i]
            local new_line = new_lines[i]
            if old_line == new_line then
                table.insert(diff_lines, " " .. (old_line or ""))
            else
                if old_line then
                    table.insert(diff_lines, "-" .. old_line)
                end
                if new_line then
                    table.insert(diff_lines, "+" .. new_line)
                end
            end
        end
        return table.concat(diff_lines, '\n')
    end
end

--- Show diff preview in a split window
-- @param diff_text string Diff text to display
-- @param callback function Callback when user confirms or cancels
function M.show_diff_preview(diff_text, callback)
    -- Create diff preview buffer
    local diff_buf = vim.api.nvim_create_buf(false, true)
    local diff_win = vim.api.nvim_open_win(diff_buf, true, {
        relative = 'editor',
        width = math.floor(vim.o.columns * 0.8),
        height = math.floor(vim.o.lines * 0.7),
        col = math.floor(vim.o.columns * 0.1),
        row = math.floor(vim.o.lines * 0.15),
        border = 'single',
        title = 'Radium Diff Preview'
    })
    
    -- Set diff content
    local diff_lines = vim.split(diff_text, '\n')
    vim.api.nvim_buf_set_lines(diff_buf, 0, -1, false, diff_lines)
    vim.api.nvim_buf_set_option(diff_buf, 'filetype', 'diff')
    vim.api.nvim_buf_set_option(diff_buf, 'readonly', true)
    
    -- Add key mappings for confirmation
    vim.api.nvim_buf_set_keymap(diff_buf, 'n', 'y', '', {
        callback = function()
            vim.api.nvim_win_close(diff_win, true)
            if callback then
                callback(true) -- confirmed
            end
        end,
        desc = 'Apply changes'
    })
    
    vim.api.nvim_buf_set_keymap(diff_buf, 'n', 'n', '', {
        callback = function()
            vim.api.nvim_win_close(diff_win, true)
            if callback then
                callback(false) -- cancelled
            end
        end,
        desc = 'Cancel'
    })
    
    -- Show help text
    vim.api.nvim_buf_set_lines(diff_buf, -1, -1, false, {
        "",
        "---",
        "Press 'y' to apply changes, 'n' to cancel"
    })
    
    vim.notify("Diff preview shown. Press 'y' to apply, 'n' to cancel.", vim.log.levels.INFO)
end

--- Apply code to buffer at specified lines
-- @param code string Code to apply
-- @param start_line number Start line (0-indexed)
-- @param end_line number End line (1-indexed, exclusive)
function M.apply_code(code, start_line, end_line)
    local new_lines = vim.split(code, '\n')
    vim.api.nvim_buf_set_lines(0, start_line, end_line, false, new_lines)
end

return M

