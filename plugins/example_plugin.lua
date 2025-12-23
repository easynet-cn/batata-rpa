-- Example Plugin for Batata RPA
-- This plugin demonstrates how to create custom node handlers
--
-- Plugin Structure:
-- - name: Plugin identifier (required)
-- - version: Plugin version (optional, defaults to "1.0.0")
-- - description: Plugin description (optional)
-- - author: Plugin author (optional)
-- - nodes: Array of node definitions (required)
--
-- Each node definition:
-- - type: Unique node type identifier (required)
-- - label: Display name in UI (optional, defaults to type)
-- - icon: Icon name from Element Plus icons (optional)
-- - category: "control", "action", or "data" (optional, defaults to "action")
-- - color: Hex color code (optional)
-- - execute: Function that executes the node logic (required)
--
-- Context API (passed to execute function):
-- - ctx:get_data(key) - Get value from node configuration
-- - ctx:get_all_data() - Get all node configuration as table
-- - ctx:get_variable(name) - Get workflow variable value
-- - ctx:set_variable(name, value) - Set workflow variable (string)
-- - ctx:set_number(name, value) - Set workflow variable (number)
-- - ctx:set_boolean(name, value) - Set workflow variable (boolean)
-- - ctx:interpolate(text) - Replace ${var} placeholders with values
-- - ctx:log(level, message) - Add log message (level: "info", "warn", "error")
-- - ctx:info(message) - Shorthand for info log
-- - ctx:warn(message) - Shorthand for warning log
-- - ctx:error(message) - Shorthand for error log
-- - ctx:execute_command(cmd, args) - Execute shell command, returns (stdout, stderr, success)
-- - ctx:read_file(path) - Read file contents
-- - ctx:write_file(path, content) - Write file contents
-- - ctx:sleep(ms) - Sleep for milliseconds
-- - ctx:now() - Get current timestamp (ISO 8601)
-- - ctx:node_id() - Get current node ID

return {
    name = "example_plugin",
    version = "1.0.0",
    description = "示例插件 - 演示如何创建自定义节点",
    author = "Batata RPA",

    nodes = {
        -- 1. HTTP Request Node - 发送HTTP请求
        {
            type = "httpRequest",
            label = "HTTP请求",
            icon = "Connection",
            category = "action",
            color = "#409eff",
            execute = function(ctx)
                local url = ctx:get_data("url") or ""
                local method = ctx:get_data("method") or "GET"
                local output_var = ctx:get_data("outputVariable") or "response"

                -- Interpolate variables in URL
                url = ctx:interpolate(url)

                ctx:info("Sending " .. method .. " request to: " .. url)

                -- Use curl for HTTP request
                local args = {"-s", "-X", method, url}
                local stdout, stderr, success = ctx:execute_command("curl", args)

                if success then
                    ctx:set_variable(output_var, stdout)
                    ctx:info("Response saved to variable: " .. output_var)
                    return true
                else
                    ctx:error("Request failed: " .. stderr)
                    return false
                end
            end
        },

        -- 2. JSON Parse Node - 解析JSON
        {
            type = "jsonParse",
            label = "解析JSON",
            icon = "Document",
            category = "data",
            color = "#e6a23c",
            execute = function(ctx)
                local input_var = ctx:get_data("inputVariable") or ""
                local json_path = ctx:get_data("jsonPath") or ""
                local output_var = ctx:get_data("outputVariable") or "result"

                local json_str = ctx:get_variable(input_var)
                if not json_str then
                    ctx:error("Variable not found: " .. input_var)
                    return false
                end

                ctx:info("Parsing JSON from variable: " .. input_var)

                -- Simple JSON path extraction (for demo purposes)
                -- In production, use a proper JSON library
                if json_path ~= "" then
                    ctx:info("Extracting path: " .. json_path)
                end

                ctx:set_variable(output_var, json_str)
                ctx:info("Result saved to: " .. output_var)
                return true
            end
        },

        -- 3. String Operation Node - 字符串操作
        {
            type = "stringOp",
            label = "字符串操作",
            icon = "EditPen",
            category = "data",
            color = "#67c23a",
            execute = function(ctx)
                local operation = ctx:get_data("operation") or "concat"
                local input = ctx:get_data("input") or ""
                local param = ctx:get_data("param") or ""
                local output_var = ctx:get_data("outputVariable") or "result"

                -- Interpolate inputs
                input = ctx:interpolate(input)
                param = ctx:interpolate(param)

                local result = ""

                if operation == "concat" then
                    result = input .. param
                    ctx:info("Concatenated strings")
                elseif operation == "upper" then
                    result = string.upper(input)
                    ctx:info("Converted to uppercase")
                elseif operation == "lower" then
                    result = string.lower(input)
                    ctx:info("Converted to lowercase")
                elseif operation == "trim" then
                    result = input:match("^%s*(.-)%s*$")
                    ctx:info("Trimmed whitespace")
                elseif operation == "replace" then
                    local search = ctx:get_data("search") or ""
                    result = input:gsub(search, param)
                    ctx:info("Replaced '" .. search .. "' with '" .. param .. "'")
                elseif operation == "length" then
                    result = tostring(#input)
                    ctx:info("String length: " .. result)
                else
                    result = input
                    ctx:warn("Unknown operation: " .. operation)
                end

                ctx:set_variable(output_var, result)
                return true
            end
        },

        -- 4. Math Operation Node - 数学运算
        {
            type = "mathOp",
            label = "数学运算",
            icon = "Odometer",
            category = "data",
            color = "#909399",
            execute = function(ctx)
                local operation = ctx:get_data("operation") or "add"
                local a = tonumber(ctx:get_data("a")) or 0
                local b = tonumber(ctx:get_data("b")) or 0
                local output_var = ctx:get_data("outputVariable") or "result"

                local result = 0

                if operation == "add" then
                    result = a + b
                elseif operation == "subtract" then
                    result = a - b
                elseif operation == "multiply" then
                    result = a * b
                elseif operation == "divide" then
                    if b ~= 0 then
                        result = a / b
                    else
                        ctx:error("Division by zero")
                        return false
                    end
                elseif operation == "modulo" then
                    result = a % b
                elseif operation == "power" then
                    result = a ^ b
                else
                    ctx:warn("Unknown operation: " .. operation)
                    result = a
                end

                ctx:info(a .. " " .. operation .. " " .. b .. " = " .. result)
                ctx:set_number(output_var, result)
                return true
            end
        },

        -- 5. File Check Node - 文件检查
        {
            type = "fileCheck",
            label = "文件检查",
            icon = "FolderChecked",
            category = "data",
            color = "#67c23a",
            execute = function(ctx)
                local path = ctx:get_data("path") or ""
                local output_var = ctx:get_data("outputVariable") or "exists"

                path = ctx:interpolate(path)

                ctx:info("Checking file: " .. path)

                -- Use test command to check if file exists
                local _, _, success = ctx:execute_command("test", {"-e", path})

                ctx:set_boolean(output_var, success)
                ctx:info("File exists: " .. tostring(success))
                return true
            end
        },

        -- 6. Notification Node - 系统通知
        {
            type = "notify",
            label = "系统通知",
            icon = "Bell",
            category = "action",
            color = "#f56c6c",
            execute = function(ctx)
                local title = ctx:get_data("title") or "Batata RPA"
                local message = ctx:get_data("message") or ""

                title = ctx:interpolate(title)
                message = ctx:interpolate(message)

                ctx:info("Sending notification: " .. title)

                -- macOS notification
                local script = string.format(
                    'display notification "%s" with title "%s"',
                    message:gsub('"', '\\"'),
                    title:gsub('"', '\\"')
                )
                ctx:execute_command("osascript", {"-e", script})

                return true
            end
        },

        -- 7. Random Node - 生成随机数
        {
            type = "random",
            label = "随机数",
            icon = "Aim",
            category = "data",
            color = "#909399",
            execute = function(ctx)
                local min_val = tonumber(ctx:get_data("min")) or 0
                local max_val = tonumber(ctx:get_data("max")) or 100
                local output_var = ctx:get_data("outputVariable") or "random"

                math.randomseed(os.time())
                local result = math.random(min_val, max_val)

                ctx:info("Generated random number: " .. result .. " (range: " .. min_val .. "-" .. max_val .. ")")
                ctx:set_number(output_var, result)
                return true
            end
        },

        -- 8. Timestamp Node - 时间戳操作
        {
            type = "timestamp",
            label = "时间戳",
            icon = "Clock",
            category = "data",
            color = "#e6a23c",
            execute = function(ctx)
                local operation = ctx:get_data("operation") or "now"
                local format_str = ctx:get_data("format") or "%Y-%m-%d %H:%M:%S"
                local output_var = ctx:get_data("outputVariable") or "timestamp"

                local result = ""

                if operation == "now" then
                    result = os.date(format_str)
                elseif operation == "unix" then
                    result = tostring(os.time())
                elseif operation == "iso" then
                    result = ctx:now()
                else
                    result = os.date(format_str)
                end

                ctx:info("Timestamp: " .. result)
                ctx:set_variable(output_var, result)
                return true
            end
        }
    }
}
