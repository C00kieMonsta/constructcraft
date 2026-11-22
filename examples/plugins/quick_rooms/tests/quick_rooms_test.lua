-- Tests of the quick_rooms plugin: `ccad plugins test examples/plugins/quick_rooms`.
-- Each test_* function runs over a new project with the plugin loaded.
local t = ccad.test

local function walls()
  return ccad.run("elements_list", { category = "wall" })
end

function test_a_room_has_four_walls_a_floor_and_a_room()
  local out = ccad.run("quick_rooms__rectangle_room", { width = 4, depth = 3, name = "Office" })
  t.eq(#walls(), 4, "walls")
  for _, side in ipairs({ "south", "east", "north", "west" }) do
    t.ok(out.walls[side], "the " .. side .. " wall")
  end
  local south = ccad.run("element_get", { id = out.walls.south })
  local east = ccad.run("element_get", { id = out.walls.east })
  t.near(south.derived.length, 4, 1e-6, "south wall length")
  t.near(east.derived.length, 3, 1e-6, "east wall length")
  t.ok(out.floor and out.room, "a floor and a room")
  t.eq(out.door, nil, "no door by default")
end

function test_the_room_keeps_its_size_as_plugin_data()
  local out = ccad.run("quick_rooms__rectangle_room", { width = 5, depth = 2.5 })
  local data = ccad.run("plugin_data_get", { plugin = "quick_rooms", element = out.room })
  t.eq(data, { width = 5, depth = 2.5 })
end

function test_walls_take_the_default_height_setting()
  local out = ccad.run("quick_rooms__rectangle_room", { width = 4, depth = 3 })
  local wall = ccad.run("element_get", { id = out.walls.north })
  t.near(wall.params.height, 2.8, 1e-6, "the default_height setting")
  local higher = ccad.run("quick_rooms__rectangle_room", { x = 10, width = 4, depth = 3, height = 3.5 })
  t.near(ccad.run("element_get", { id = higher.walls.north }).params.height, 3.5, 1e-6, "a height given")
end

function test_a_door_on_the_side_asked_for()
  local out = ccad.run("quick_rooms__rectangle_room", { width = 4, depth = 3, door = "south" })
  local door = ccad.run("element_get", { id = out.door })
  t.ok(door, "a door")
end

function test_sizes_must_be_positive()
  t.fails(function()
    ccad.run("quick_rooms__rectangle_room", { width = 0, depth = 3 })
  end, "must be positive")
end
