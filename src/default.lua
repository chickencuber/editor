local function move(dx, dy)
    return function()
        panes:get(0, function(pane)
            local x, y = pane:get_cursor()
            x = x + dx
            y = y + dy
            if x >= 0 and y >= 0 then
                pane:set_cursor(x, y)
            end
        end)
    end
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
    move(-1, 0)()
    config.mode = "n"
end)

config:key("n", "v", function()
    config.mode = "v"
end)


config:key("nv", "h", "<Left>")
config:key("nv", "j", "<Down>")
config:key("nv", "k", "<Up>")
config:key("nv", "l", "<Right>")

config:key("vin", "<Left>", move(-1, 0));
config:key("vin", "<Down>", move(0, 1));
config:key("vin", "<Up>", move(0, -1));
config:key("vin", "<Right>", move(1, 0));

config:key("n", "dd", function()
    panes:get(0, function(pane)
        pane:delete_line()
    end)
end)

