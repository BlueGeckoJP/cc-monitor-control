-- Prepare modules
local term = term
local os = os
local sleep = os.sleep or sleep
local peripheral = peripheral
local textutils = textutils
local http = http
local colors = colors

local monitor = peripheral.wrap("") -- This value is set before the download
print(monitor.getSize())

-- HTTP Settings
local frame_endpoint = "" -- This value is set before the download
print("HTTP client initialized")

local function drawFrame(frame)
    local width, height = monitor.getSize()

    monitor.clear()

    -- Write frames
    for y = 1, height do
        local startIdx = (y-1)*width + 1
        local line = frame:sub(startIdx, startIdx + width - 1)
        monitor.setCursorPos(1, y)
        for i = 1, #line do
            local c = line:sub(i,i)
            local col = colors.fromBlit(c) or colors.black
            monitor.setBackgroundColor(col)
            monitor.write(" ")
        end
    end
end


-- Main loop
print("Waiting for frames... Press Ctrl+T to stop")
while true do
    -- Request frame
    local response = http.get(frame_endpoint)
    if not response then
        print("Failed to fetch frame")
        sleep(1)  -- Wait on error
    else
        local msg = response.readAll()
        response.close()

        -- JSON parsing
        local ok, tbl = pcall(textutils.unserializeJSON, msg)
        if ok and tbl and tbl.frame then
            drawFrame(tbl.frame)
        end
    end

    -- Frame rate adjustment
    sleep(0.5)

    -- Stop condition (Press Ctrl+T to stop)
end
