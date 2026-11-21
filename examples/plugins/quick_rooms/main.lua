-- Quick rooms: an example ConstructCraft plugin.
local rect = require("lib.rect")

-- A length parameter: its title and description come from the locales.
local function length(key)
  return {
    type = "number", ui = "length", title = "%param." .. key .. ".title%", description = "%param." .. key .. ".description%",
  }
end

ccad.command {
  name = "rectangle_room",
  title = "%room.title%",
  description = "%room.description%",
  params = {
    properties = {
      x = length("x"),
      y = length("y"),
      width = length("width"),
      depth = length("depth"),
      height = length("height"),
      level = { type = "integer", ui = "level", title = "%param.level.title%", description = "%param.level.description%" },
      name = { type = "string", title = "%param.name.title%", description = "%param.name.description%" },
      door = {
        type = "string",
        enum = { "south", "east", "north", "west" },
        enum_labels = { "%side.south%", "%side.east%", "%side.north%", "%side.west%" },
        title = "%param.door.title%",
        description = "%param.door.description%",
      },
    },
    required = { "width", "depth" },
  },
  run = function(args)
    if args.width <= 0 or args.depth <= 0 then
      error("width and depth must be positive", 0)
    end
    local x, y = args.x or 0, args.y or 0
    -- The settings stand in for what the command is not given.
    local height = args.height or ccad.settings.default_height
    local door = args.door
    if door == nil and ccad.settings.default_door ~= "none" then
      door = ccad.settings.default_door
    end
    local p = rect.corners(x, y, args.width, args.depth)
    local walls = {}
    for i = 1, 4 do
      local wall = ccad.run("wall_create", {
        start = p[i], ["end"] = p[i % 4 + 1], height = height, level = args.level,
      })
      walls[rect.sides[i]] = wall.id
    end
    local floor = ccad.run("floor_create", { boundary = p, level = args.level })
    local room = ccad.run("room_create", { point = rect.center(x, y, args.width, args.depth), name = args.name, level = args.level })
    -- The plugin's own data on the room: the size it was drawn with.
    ccad.data.set(room.id, { width = args.width, depth = args.depth })
    local out = { walls = walls, floor = floor.id, room = room.id }
    if door then
      out.door = ccad.run("door_create", { wall = walls[door] }).id
    end
    return out
  end,
}

ccad.command {
  name = "wall_totals",
  title = "%totals.title%",
  description = "%totals.description%",
  kind = "query",
  run = function()
    local totals = {}
    for _, item in ipairs(ccad.run("elements_list", { category = "wall" })) do
      local wall = ccad.run("element_get", { id = item.id })
      local name = wall.type.name
      local t = totals[name] or { count = 0, length = 0 }
      t.count = t.count + 1
      t.length = t.length + wall.derived.length
      totals[name] = t
    end
    return totals
  end,
}
