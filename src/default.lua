local function move(dx, dy)
    panes:get(0, function(pane)
        local x, y = pane:get_cursor()
        x = x + dx
        y = y + dy
        if x >= 0 and y >= 0 then
            pane:set_cursor(x, y)
        end
    end)
end

config:key("n", "i", function()
    config.mode = "i"
end)

config:key("nv", "<BS>", "<Left>")

config:key("n", "a", "<Right>i")

config:key("v", "<Esc>", function()
    config.mode = "n"
end)

config:key("i", "<Esc>", function()
    move(-1, 0)
    config.mode = "n"
end)

config:key("n", "v", function()
    config.mode = "v"
end)

config:key("nv", "$", function()
    panes:get(0, function(pane)
        local _, y = pane:get_cursor();
        pane:set_cursor(pane:linelen(y), y)
    end)
end)


function mvc(dx, dy)
    return function()
        move(dx*config.count, dy*config.count)
    end
end

config:key("nv", "h", mvc(-1, 0));
config:key("nv", "j", mvc(0, 1));
config:key("nv", "k",  mvc(0, -1));
config:key("nv", "l", mvc(1, 0));

config:key("vin", "<Left>", mvc(-1, 0));
config:key("vin", "<Down>", mvc(0, 1));
config:key("vin", "<Up>",  mvc(0, -1));
config:key("vin", "<Right>", mvc(1, 0));

config:key("vn", "0", function()
    panes:get(0, function(pane)
        local _, y = pane:get_cursor()
        pane:set_cursor(0, y)
    end)
end)

config:key("n", "dd", function()
    panes:get(0, function(pane)
        for _ = 1, config.count, 1 do
            pane:delete_line()
        end
    end)
end)


config:key("n", "o", "$a<CR><Esc>")

config:key("n", "O", "0i<CR><Esc>k")

