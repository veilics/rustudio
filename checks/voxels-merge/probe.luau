-- Play probe for Terrain:ReadVoxels / ReadVoxelChannels / WriteVoxels and
-- EditableMesh:MergeVertices: prints PASS/FAIL lines and timings.
--   rustudio --write-corpus /tmp/corpus
--   rustudio /tmp/corpus/control.rbxlx --play --from-play \
--     --luau server checks/voxels-merge/probe.luau@3 --shot-after 8 --screenshot s.png --log out.txt
-- Expected: 9 PASS, no FAIL. Measured (debug build, 2 cores, software GPU):
-- MergeVertices 28 800 verts 4.5 s before the grid -> 16 ms; ReadVoxels 64x16x64 ~50 ms -> ~22 ms.
local T = workspace.Terrain
local function check(ok, what) print((ok and "PASS " or "FAIL ") .. what) end

-- Terrain: fill, read back a 64x16x64 voxel region (65 536 voxels), round-trip
T:FillBlock(CFrame.new(0, -32, 0), Vector3.new(256, 32, 256), Enum.Material.Grass)
T:FillBlock(CFrame.new(40, -20, 40), Vector3.new(32, 8, 32), Enum.Material.Rock)
local region = Region3.new(Vector3.new(-128, -64, -128), Vector3.new(128, 0, 128))
local t0 = os.clock()
local mats, occs = T:ReadVoxels(region, 4)
local dt = os.clock() - t0
print(string.format("ReadVoxels 64x16x64: %.1f ms", dt * 1000))
check(mats.Size == Vector3.new(64, 16, 64), "ReadVoxels size " .. tostring(mats.Size))
check(mats[1][5][1] == Enum.Material.Grass and occs[1][5][1] > 0.99, "grass voxel read")
check(mats[42][12][42] == Enum.Material.Rock, "rock voxel read: " .. tostring(mats[42][12][42]))
check(mats[1][16][1] == Enum.Material.Air, "air above ground")
local ch = T:ReadVoxelChannels(region, 4, {"SolidMaterial", "SolidOccupancy"})
check(ch.SolidMaterial[42][12][42] == Enum.Material.Rock, "ReadVoxelChannels material")
-- write a sand slab and read it back
local small = Region3.new(Vector3.new(0, 0, 0), Vector3.new(8, 4, 8))
local m, o = {}, {}
for x = 1, 2 do m[x], o[x] = {}, {} for y = 1, 1 do m[x][y], o[x][y] = {}, {} for z = 1, 2 do
  m[x][y][z] = Enum.Material.Sand; o[x][y][z] = 0.5 end end end
T:WriteVoxels(small, 4, m, o)
local m2, o2 = T:ReadVoxels(small, 4)
check(m2[2][1][2] == Enum.Material.Sand and math.abs(o2[2][1][2] - 0.5) < 0.01, "WriteVoxels round trip " .. tostring(o2[2][1][2]))

-- EditableMesh: a 120x120 grid written twice (28 800 vertices), merged
local AS = game:GetService("AssetService")
local em = AS:CreateEditableMesh()
local ids = {}
for pass = 1, 2 do
  for x = 0, 119 do for z = 0, 119 do
    ids[#ids + 1] = em:AddVertex(Vector3.new(x, 0, z))
  end end
end
local a, b, c = ids[1], ids[2], ids[121]
em:AddTriangle(ids[14401], ids[14402], ids[14521])
local t1 = os.clock()
local map = em:MergeVertices(0.001)
local dt2 = os.clock() - t1
local n = 0 for _ in pairs(map) do n += 1 end
print(string.format("MergeVertices 28800 verts: %.1f ms, merged %d", dt2 * 1000, n))
check(n == 14400, "merge count " .. n)
check(map[ids[14401]] == a, "second copy maps to first")
local f = em:GetFaces()[1]
local fv = em:GetFaceVertices(f)
check(fv[1] == a and fv[2] == b and fv[3] == c, "face rewired to merged vertices")
print("PROBE DONE")
