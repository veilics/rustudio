# Engine gaps (measured)

What real games touch that rustudio doesn't implement, counted by the gap
telemetry (`RUSTUDIO_GAPS=FILE`, crates/rustudio-core/src/telemetry.rs) while playing the
fidelity corpus: **Place A** (lobby, START GAME, 60 s of a Map 1 match with
walking and firing), the **demo** scene and the **control** place (Play,
walking, jumping, clicking). Every member is checked against Studio's own API
dump (app/api-dump.json), so only real Roblox members are counted.

Gap kinds:

| Kind | Meaning |
|---|---|
| `missing` | a real member the runtime doesn't know (indexing it errors) |
| `call-unimplemented` | a known method with no implementation behind it |
| `default-only` | a property Roblox works out at run time (not saved in places) that rustudio only answered with its default |
| `write-ignored` | such a property written with nothing in rustudio acting on it |
| `event-unwired` | an event scripts connect to that nothing in rustudio fires (not named by any fire path in the engine's sources, and not fired during the run) |

## EditableMesh / EditableImage / Content (Place A's skinned guns)

Were silent stubs (AddVertex returned 0, Size (0, 0), ReadPixelsBuffer nil,
`Content.fromObject` missing): Place A's WeaponModels / CamoFx / CamoLayers fell
back inside their pcalls and every camo'd gun drew plain grey. Now
(crates/rustudio-engine/src/editable.rs, a runtime submodule):

- **Content** is a datatype (typeof "Content"; `SourceType`, `Uri`,
  `Object`): `Content.none`, `fromUri`, `fromAssetId`, `fromObject(EditableImage | EditableMesh)`.
  Every `*Content` property reads as one and takes one (or a string).
- **EditableMesh**: vertices, normals, UVs, colours and triangles under typed
  ids; Add*/Get*/Set*, Get/SetFace{Vertices,Normals,UVs,Colors},
  RemoveFace / RemoveUnused / Clear, GetVertexFaces, GetAdjacentVertices,
  GetCenter / GetSize, RaycastLocal, FindClosestVertex,
  FindVerticesWithinSphere, BatchAdd / BatchSetValues / BatchGetValues.
  AssetService:CreateEditableMesh(Async) (an asset URI or another
  EditableMesh; FixedSize kept). CreateMeshPartAsync(Content.fromObject(em))
  snapshots it as a generated 2.00 mesh (`rbxtemp://rustudio/editable/mesh/…`),
  so the MeshPart is drawn, ray-tested and collided as any MeshPart; Size /
  MeshSize = the mesh bounds.
- **EditableImage**: RGBA8 pixels at its Size; Read/WritePixelsBuffer,
  DrawRectangle / DrawCircle / DrawLine / DrawTriangle / DrawImage /
  DrawImageTransformed (BlendSourceOver, Overwrite, Add, Multiply,
  Subtract). Shown wherever a Content points at it (ImageLabel/ImageButton
  ImageContent, Decal/Texture TextureContent, SurfaceAppearance maps via
  AssetService:CreateSurfaceAppearanceAsync, clones included); each write
  re-uploads into the same GPU / egui texture slot (no growth per frame).
- **Any stub function**, on its first call, now prints
  `rustudio: Class.Member is a stub (it does nothing here)` to Output (once
  per member), so a test can't pass on one silently.

Since done (editable_mesh.rs, editable.rs, project.rs): bones / FACS,
DrawImageProjected / SampleImageProjected, MergeVertices / Triangulate and
the *WithAttribute queries. MergeVertices finds its pairs through a grid
(`merge_map`, the same pairs as the all-pairs scan it replaced, which was
n² distance tests: a 45 000-vertex unwelded mesh took seconds).

Still open: edits to an EditableMesh after CreateMeshPartAsync don't reach
the MeshPart (it holds a snapshot; CamoLayers' animated shell UVs stay at
their first frame); no Editable* memory budget (Create* never returns nil); EditableImage /
EditableMesh are Instances here (typeof "Instance", Roblox: "Object").

## Round 19: the whole corpus, and what was wrong rather than missing

Ranked by `RUSTUDIO_GAPS` calls summed over the six fidelity places (Place A's
lobby, Place A mid-match, Place C, Place B, demo, control; each place's
`checks/compare/<place>/rustudio/gaps.json`). Only two stubbed members are
still called anywhere in the corpus; the rest of the round's engine work is
members that existed but behaved wrongly, found by the diff against Studio:

| # | Calls | Member | Kind | Places | Status |
|---|---|---|---|---|---|
| 1 | 15 | Terrain:SetMaterialColor (and GetMaterialColor) | stub-call | Place C (Main: every biome's colours) | **done**: writes / reads Terrain.MaterialColors (23 RGB slots), replicated; the drawn terrain follows |
| 2 | 1 (+ every landing) | Terrain:FillBlock, with FillBall / FillCylinder / FillWedge / FillRegion / Clear / ReplaceMaterial | call-unimplemented | Place B (water blocks); Place C builds every planet surface with ~700 FillBlocks on landing (not reached in the 40 s run) | **done**: voxels edited and written back to SmoothGrid once a step (drawn, collided, replicated). WriteVoxels / ReadVoxels / Read/WriteVoxelChannels / CopyRegion / PasteRegion since done; SetCell (deprecated cell API) still open |
| – | every terrain ray | WorldRoot:Raycast, FindPartOnRay… | wrong: never hit terrain | any place with terrain | **done**: hits the voxels' surface with its material and normal (water not yet) |
| – | every frame | CFrame `toAxisAngle` (and `lerp`, `toEulerAngles*`, `toOrientation`, Color3 `lerp`) | missing lowercase aliases | Place B's Poppercam | **done** (Studio's list, probed) |
| – | once per PlayerModule | UserSettings():IsUserFeatureEnabled | wrong: errored on 16 names Studio answers | Place B, Place C, Place A | **done**: Studio's values; Place B's camera now pulls in like Studio's |
| – | every top-down frame | CFrame.lookAt / CFrame.new(pos, at) near vertical | wrong: turned 180° | the calibration scenes; any top-down camera | **done** |
| – | every frame | LinearVelocity (Vector) / AlignOrientation (soft) on a HumanoidRootPart | wrong: ignored | Place C's ship | **done**: the ship holds the player in space (camera matches Studio's exactly) |
| – | once | Humanoid.PlatformStanding on the client | event not fired on the client | Place C | **done** |
| – | every frame | parts welded to a character | wrong: simulated loose | Place C's ship | **done**: carried with the character |
| – | every landing | Humanoid.FloorMaterial on terrain | wrong: Plastic | any terrain place | **done**: the voxel's material |

Render fixes from the same places: Neon bloom (rings → Studio's smooth
two-Gaussian halo, self-test `bloom-smooth`), see-through parts drawn from
inside (Place B's white wash), Technology ShadowMap → Soft, a script
camera's roll. Measurements and sources: MECHANICS.md, round 19.

Open, by how much they block the corpus: Realistic local lights close to
the light (`lightcap-real`: Studio 2–4× brighter within a few studs);
water in terrain rays; WriteVoxels / ReadVoxels; Place C's `animation`
area reads 0 from an offline-Studio artifact (MECHANICS.md, round 19), and
Place B's walk / jump facts differ because offline Studio's meshes around its
spawn never load and collide as boxes.

## Round 12: ranked by real call count

Counts are touches across the three places (each place started once, the
pre-fix build). "Scripts" are the top callers. Behaviour is Roblox's
documented one (create.roblox.com/docs/reference/engine/classes/…).

| # | Count | Class.member | Kind | Studio-documented behaviour | Scripts | Status |
|---|---|---|---|---|---|---|
| 1 | 17 | Players.PlayerRemoving | event-unwired | Fires on the server and clients just before a Player leaves the game (also when a Studio test stops), with the Player | PlayerModule.CameraModule, ZoomController.Popper, RbxCharacterSounds, Place A LowGrav / OtherPlayers | **done**: fired on both sides when Play stops, with a dispatch so handlers run |
| 2 | 12 | Player.CharacterRemoving | event-unwired | Fires just before a player's Character is removed (respawn, LoadCharacter, leaving), with the old character | PlayerModule.CameraModule, Popper, ControlModule | **done**: fired on both sides before respawn / LoadCharacter removes it |
| 3 | 6 | UserInputService.TextBoxFocusReleased | event-unwired | Fires when a TextBox loses focus, with that TextBox | CameraInput, ControlModule.Keyboard | **done**: fired with each TextBox.FocusLost |
| 4 | 6 | UserInputService.WindowFocused | event-unwired | Fires when the Roblox window gains focus | CameraInput, ControlModule.Keyboard | **done**: from the OS window's focus each frame; UIS:IsWindowFocused() too |
| 5 | 5 | Humanoid.Seated | event-unwired | Fires when the humanoid sits in or leaves a Seat / VehicleSeat: (active, seatPart) | ControlModule, Animate | **done** (round 13): touching an enabled empty Seat / VehicleSeat or Seat:Sit sits the humanoid (SeatWeld built as Studio's does: C0 (0, Size.Y × 0.5, 0)·Angles(−π/2,0,0), C1 per rig type from the rig attachments; Seat.Occupant, Humanoid.SeatPart / Sit, state Seated); jumping, Sit = false, a removed weld or a disabled / gone seat gets it up. Proved in Play on the control place: Seated(true, TestSeat), root 2.00 over the seat's centre, jump → Seated(false, nil) + Jumping, touch re-sits after the 1 s cooldown |
| 6 | 3 | GuiService.MenuOpened | event-unwired | Fires when the player opens the Roblox Esc menu | CameraInput | **done** (round 13): Esc in Play opens a game menu (Resume / Reset Character / Leave); MenuOpened / MenuClosed fire on the client and GuiService.MenuIsOpen follows. Proved on the control place: MenuOpened(true) MenuClosed(false) ×2; Reset killed the character (Died on server and client) |
| 7 | 3 | UserInputService.TextBoxFocused | event-unwired | Fires when a TextBox gains focus, with that TextBox | ControlModule.Keyboard | **done**: fired with each TextBox.Focused |
| 8 | 3 | UserInputService.WindowFocusReleased | event-unwired | Fires when the Roblox window loses focus | CameraInput | **done** (as WindowFocused) |
| 9 | 2 | DataModel.PlaceId | default-only | The place's id; 0 for an unpublished place opened from a file in Studio | Place A PlayerSettings, Progression | **done**: 0, with GameId / CreatorId / PlaceVersion 0 and JobId / PrivateServerId "" |
| 10 | 2 | Humanoid.Climbing | event-unwired | Fires while climbing (a TrussPart or ladder) with the climb speed | Animate | **done** (round 13): walking into a TrussPart climbs at the move speed × 0.7, down when moving away from it, standing still holds on, jumping leaves away from it. Proved: Humanoid:MoveTo into a 24-stud truss rose 21.1 studs, Climbing(11.2) |
| 11 | 2 | Humanoid.FallingDown | event-unwired | Fires on entering / leaving the FallingDown state (active) | Animate | **done** (round 13): Humanoid:ChangeState(FallingDown) holds the humanoid 3 s, then GettingUp until upright, then Running; FallingDown(true/false) fire. Proved: StateChanged FallingDown@0.0 → GettingUp@3.0 → Running |
| 12 | 2 | Humanoid.GettingUp | event-unwired | Fires on entering / leaving the GettingUp state (active) | Animate | **done** (round 13): as FallingDown (GettingUp(true) on entering, (false) on leaving) |
| 13 | 2 | Humanoid.PlatformStanding | event-unwired | Fires when PlatformStand changes (active) | Animate | **done**: fired when PlatformStand changes |
| 14 | 2 | Humanoid.Swimming | event-unwired | Fires while swimming in terrain water, with the speed | Animate | **done** (round 13): the root in terrain water (occupancy > 0.5) swims: no gravity, Jump rises, a submerged idle humanoid floats up; Swimming(speed) on change. Proved by unit test `physics::water_and_floating` (no corpus place has water; Terrain editing from scripts is still unimplemented) |
| 15 | 2 | Instance.Archivable | default-only | Whether the instance is saved and cloned: Clone() returns nil for a non-archivable instance and leaves out non-archivable descendants | Place A Match | **done**: Clone() leaves non-archivable descendants out (the root case already returned nil) |
| 16 | 2 | ScriptContext.Error | event-unwired | Fires when a script errors: (message, stackTrace, script) | Place A DevErrors, Main | **done**: fired for every script error on that side, with the script when its name resolves |
| 17 | 2 | StudioTestService.GetTestArgs | missing | The arguments a test run was started with (ExecuteRunModeAsync / ExecutePlayModeAsync); nil for a plain Play | Place A DevTest, Progression | **done**: nil |
| 18 | 1 | UserInputService.GamepadDisconnected | event-unwired | Fires when a gamepad is unplugged, with its UserInputType | Place A GameInput | open: no gamepad input yet |
| 19 | 1 | UserInputService.LastInputTypeChanged | event-unwired | Fires when the kind of input the player last used changes (keyboard, mouse movement, a button…), with the new UserInputType | Place A GameInput | **done**: from the input stream; GetLastInputType() follows it, and before any input answers the device's kind (Touch on a touch screen, Keyboard on a desktop) as Roblox does — the PlayerModule picks its touch controls from that first answer |

After the fixes the same runs touch: Place A 3 open gaps (GuiService.MenuOpened,
Humanoid.Seated, UserInputService.GamepadDisconnected), demo and control 6
each (Humanoid.Seated / Climbing / FallingDown / GettingUp / Swimming,
GuiService.MenuOpened). Output stays free of errors in all three.

Telemetry fixes found on the way: the first Place A run counted 115,176
`write-ignored` writes to Motor6D.Transform (the engine does read it, through
`properties.get`), and `default-only` reads of Workspace.Terrain /
DataModel.Workspace (they resolve to children); the build-time scan of the
engine's sources (build.rs) now covers both.

## Round 14: every script-reachable member exists

**Stubs.** `tools/gen_stubs.py` writes `crates/rustudio-core/src/api_stubs.rs` from the dump:
4646 members a game script may reach (Security None; properties by read
security), each with a typed default (`false`, `0`, `""`, `{}`, the enum's
item 0, `Vector3.new()`…, a signal). `crates/rustudio-engine/src/stubs.rs` answers from it only
after the runtime's own lookups fail: reads give the default (or what a
script wrote; read-only ones refuse with Studio's message), writes are kept
per instance and fire Changed, calls return the typed empty value, events
are connectable signals nothing fires, callbacks can be assigned. Each use
is logged in the gap telemetry (`stub`, `stub-call`, `stub-write`). Left
out: NotScriptable members and the plugin / Studio-only classes (debugger,
dragger, Selection, ChangeHistoryService…). Lowercase deprecated spellings
(`sound:play`) still reach the real capitalised member first; a
reflection-known property without a reflection default (TextFits,
InterpolationThrottling) now reads its typed default instead of nil.
build.rs skips the generated file when scanning for the events the engine
fires (otherwise every event would look wired).

**Ranked by real calls.** Place A's match, the newer Place A build
(`place-a/build/place-a.rbxl`), Place B (og.rbxl), Place B'
laser tag, Godelz RPG, demo and control, each played (walk, jump, clicks,
E / F), with `RUSTUDIO_GAPS` (`../checks/coverage/gaps-*-r14.json`):

| # | Count | Member | Kind | Places | Status |
|---|---|---|---|---|---|
| 1 | (every read) | string attributes (BinaryString) | wrong value | laser tag (`viewModel`, `fireMode`) | **fixed**: rbx-dom reads string attributes from binary places as BinaryString and the runtime turned them into nil; the laser-tag blasters errored on every equip. Now strings; 0 Output errors there |
| 2 | 18 | TweenBase.PlaybackState | default-only | Place B | **done**: Begin / Delayed / Playing / Paused / Completed / Cancelled from the tween's run |
| 3 | 7 | Model.WorldPivot (write) | write-ignored | newer Place A build (Bo3Map) | **done**: with a PrimaryPart, sets its PivotOffset = PrimaryPart.CFrame⁻¹ · pivot; else the model's WorldPivotData; GetPivot / WorldPivot read it back |
| 4 | 2 | UIGridStyleLayout.AbsoluteContentSize | default-only | Godelz RPG | **done**: the extent of the visible GuiObjects it lays out, as drawn last frame (UDim2 rects before) |
| 5 | 2 | UserInputService.GamepadDisconnected | event-unwired | Place A, newer Place A | **done**: Play scans the joystick devices (/dev/input/by-id/*-event-joystick; Linux only, none on macOS or Windows yet) about once a second and fires GamepadConnected / GamepadDisconnected with Gamepad1…8; GetConnectedGamepads / GetNavigationGamepads / GetGamepadConnected answer from it (buttons and sticks: not yet) |
| 6 | 2 | Mouse.Icon | stub default | PlayerModule (shift-lock) | **done**: UserInputService.MouseIcon, else the PlayerMouse's Icon, is drawn centred on the pointer over the game view in place of the system arrow |

After these, a stub-hit count from the noise of unset object properties
(Player.Character, Humanoid.Torso read nil, which is Roblox's value, not a
stub) was removed from the telemetry. The remaining Output errors in those
places are Studio's too for an unpublished place: DataStore access, a free
model's `require(assetId)`, and the leaderstats those scripts then never
make.

Coverage (round 14, `../checks/coverage/round-14.json`), game scripts:
functions exist 1569 / real 342 of 1626; properties 2801 / 2279 of 2802;
events 398 / 172 of 410; callbacks 13 / 3 of 13 (OnInvoke,
OnServerInvoke, OnClientInvoke are real).

## Round 13

Every open row above is done except GamepadDisconnected (no gamepad input:
needs a gamepad library, 1 call). The same three runs on the round-13 build
touch one open gap in all: Place A `UserInputService.GamepadDisconnected`
(demo and control: none). Output free of errors in all three.

Also implemented while working the list: Humanoid:SetStateEnabled /
GetStateEnabled (a disabled state is never entered), ChangeState beyond
Jumping (Seated, Dead, FallingDown, GettingUp, Ragdoll, Physics, the
walking states), Seat:Sit / VehicleSeat:Sit, client-side HealthChanged /
Died from a replicated Health, Lighting.ClockTime / TimeOfDay /
SetMinutesAfterMidnight as one value.

### Place A as a game: why Round read nil and no one scored

Diagnosed with probes on both sides (round 13):

1. **Collision groups lived in each script VM.** Place A's server registers
   WeaponRay / PlayerClip / ZombieClip and sets them uncollidable; the
   client never saw that, so its weapon rays (`CollisionGroup = "WeaponRay"`)
   stopped on the window PlayerClip brushes and no hit was ever reported
   (0 `ZHit` remotes). With no kills the player was downed at ~88 s, the
   session ended and its folder (with Round / RoundState) was deleted:
   that is the nil the probes read. **Fixed**: one shared table per Play
   session (crates/rustudio-core/src/collision_groups.rs) used by both runtimes' queries and by
   the physics (rapier interaction groups; the character collides as its
   root part's group).
2. **SetAttribute fired on an unchanged value** (2236 AttributeChanged for
   ZombiesLeft in a minute). **Fixed**: no change, no event.
3. **Key taps within one frame were lost** (R to reload, F to buy).
   **Fixed**: they reach InputBegan / InputEnded.
4. The harness fired at a wall: the client probe now aims at the nearest
   enemy a WeaponRay reaches, as a player would.

Result (`place_a_match.py`, gate now requires ≥ 1 kill and points over 500):
round 1 active after 36 s, 6 kills, 1040 points, ZombiesLeft 6 → 1 and the
round ending into intermission within the 60 s watched, not downed, 0
error lines. Wall buys and doors (hold F), auto and R reload and the HUD
(points, weapon, clip / reserve, buy hints) work. Noted, not an engine
issue: `place-a.rbxl` is older than the Place A source (built 2026-10-26).

## What else came out of making Place A playable

Not API gaps but engine bugs the Place A match exposed (enemies never spawned):

- **Mesh raycasts reported the box face's normal** instead of the hit
  triangle's. Map 1's outdoor ground is terrain baked into MeshParts, so
  every downward hit there read Normal.Y = 0; Place A's nav-grid builder needs
  Normal.Y > 0.7, found no floor outside, and no spawn spot connected to a
  window. Now the triangle's normal (inverse-transpose of the mesh scale).
- **WorldRoot:GetPartsInPart compared bounding boxes**; Roblox tests the real
  shapes (Place A's headroom probes rely on it: a turned brush's or a terrain
  chunk's box covers far more than the part). Now exact: box vs box by
  separating axes, box vs mesh triangles (crates/rustudio-physics/src/query_shapes.rs).
- **Invoke before the callback existed errored** ("OnInvoke is not set" /
  "OnServerInvoke is not set"). Roblox holds the call: a BindableFunction's
  `Invoke` yields until `OnInvoke` is set (and never resumes if it never
  is), a RemoteFunction queues `InvokeServer` / `InvokeClient` until
  `OnServerInvoke` / `OnClientInvoke` is assigned (both class pages). A
  character's Controls LocalScript (LBG, Android) invoked a BindableFunction
  40 ms before the 70k-line renderer LocalScript set its callback and died
  at that line every spawn. Now every path waits a frame at a time
  (crates/rustudio-engine/src/runtime.rs PRELUDE `__rs_wait_callback`, INVOKE_PRELUDE
  `__rs_serve_invoke`; test `invoke_waits_for_the_callback`).
- **Studio 0.748's PlayerModule lost its InputContexts** (content
  `version-8d7a1009c17e4ff8`, deployed 2025-11-16). `PlayerScripts.rbxp` now
  points `PlayerModule.InputContexts` at a second project file
  (`"$path": "InputContexts.rbxp"`, 4 InputContexts / 30 InputActions / 70
  InputBindings) where 0.747 spelled the tree out inline; the Rojo-style
  loader only followed `$path`s to `.lua` / `.luau` files and folders, so
  the Folder came up empty. InputSetup warned "InputContexts not found" and
  ControlModule, BaseCamera, CameraInput, MouseLockController, the
  thumbsticks, ClickToMove and VehicleController all parked on
  `WaitForChild("InputContexts")`: the character spawned, nothing walked or
  looked, on every platform once the content updated. Fixed upstream in
  0.2.6 (`e3f06fd`, "Fixed Play controls and camera on fresh installs"):
  a `$path` ending in `.rbxp` is loaded as that project's `tree`, relative
  to its own file, `$properties` over an imported tree apply, and a missing
  source is an error Play reports ("Could not load Studio PlayerModule for
  Play: …") instead of a silent empty Folder (crates/rustudio-core/src/default_scripts.rs
  `project_tree` / `project_node`; test
  `project_imports_input_contexts_with_properties_and_rejects_missing_sources`).
  The port's own fix of the same gap was dropped for upstream's.
- **A new place got the Input Action System PlayerScripts.** Which player
  scripts a place gets is `Workspace.PlayerScriptsUseInputActionSystem`
  (Enum.RolloutState): the loader took the new layout
  (`StarterPlayer.PlayerModule` + its own `PlayerScriptsLoader`, nothing
  in `Player.PlayerScripts` but `RbxCharacterSounds`) for every place
  whose flag wasn't Disabled — so for Default, every Baseplate — where
  Roblox's rollout (phase 1, through 2026) resolves Default to Disabled:
  the live client gives those places the classic `PlayerScriptsLoader` +
  `PlayerModule` + `RbxCharacterSounds` in `Player.PlayerScripts`, and
  games `require(LocalPlayer.PlayerScripts.PlayerModule):GetControls()`.
  Now Default and Disabled are the classic layout, Enabled the Input
  Action System one (`StarterPlayer.CreateDefaultPlayerModule` false
  leaves its PlayerModule out), a fork in StarterPlayerScripts keeps the
  classic layout around it as before, and
  `mech::PLAYER_SCRIPTS_DEFAULT_IAS` is the one place to flip when
  Roblox's phase 2 (early 2027) makes Default mean Enabled
  (`input_action_system_on`; test
  `player_scripts_layout_follows_the_rollout_flag`).
- **A place's own PlayerModule never ran without Studio's content.**
  The classic loader, PlayerModule and RbxCharacterSounds come from
  Studio's content folder, and without it (a phone before the download,
  an offline desktop) `insert_player_scripts` added nothing — not even
  the loader — so a place with its own `PlayerModule` fork in
  StarterPlayerScripts (LBG: the shift lock's MouseLockController, its
  camera and touch controls) never had it required; the engine's
  keyboard stand-in moved the character instead. `default_scripts.rs`
  now carries a loader of its own (`PLAYER_SCRIPTS_LOADER`: require the
  module once, as Roblox's does) and inserts it when a place has a
  PlayerModule but no loader, content or not (`insert_own_loader`; test
  `own_player_module_gets_a_loader_without_studio_content`).
- **`GuiService.TopbarInset` was an empty Rect for ever.** TopbarPlus
  (`IconNew`, `TopbarPlusGui`) and LBG's ClientRenderer read
  `TopbarInset.Height`, wait on `GetPropertyChangedSignal("TopbarInset")`
  while it is 0 and only then build their icons — a wait nothing ended.
  In Play it is now the top bar's free area as Roblox's reference
  describes it — from the end of the menu and chat buttons (`play_game.rs
  top_bar`: 128 px with the chat button, 72 without) to the viewport's
  right edge, 58 high (the new topbar; the old one's 36 is what
  TopbarPlus tests for) — an empty Rect in the edit DataModel, and
  `gui_input.luau` `step_topbar` publishes the change when the viewport
  or the chat button changes (test
  `topbar_inset_is_the_bar_and_follows_the_viewport`). The left edge is
  this build's bar, not a measurement of Roblox's unibar.
- **`ScreenGui.ScreenInsets = TopbarSafeInsets` was laid out as
  CoreUISafeInsets** (below the bar, the whole view). TopbarPlus puts its
  icons in such a ScreenGui (v3's `TopbarStandard`: holders `Size (1, 0,
  1, …)`; v2's `TopbarPlusGui`: the container at Y scale 250.2 of it,
  icons at −0.5 of that) — both arithmetic on a gui that *is* the
  TopbarInset rect — so the icons landed at the left edge, over the menu
  button, a fifth of the view down. Now a TopbarSafeInsets ScreenGui is
  the TopbarInset: it starts after Roblox's buttons (72 / 128 px:
  `mech::gui_topbar_left`, the chat button from `Gui::topbar_chat`), is 58
  px tall and sits at AbsolutePosition (left, −58) (`gui_input.rs` roots,
  `runtime_ext.rs gui_rect`, `GetInsetArea(TopbarSafeInsets)`), and no
  ScreenGui's content is clipped to its inset box any more (Roblox draws
  a Frame at Y −58 under the bar; TopbarPlus's dropdowns hang below the
  bar). Test `topbar_safe_insets_gui_is_the_bars_free_area`.
- **UIListLayout's flex settings were ignored**: `Wraps`, `HorizontalFlex`
  / `VerticalFlex` (UIFlexAlignment Fill / SpaceAround / SpaceBetween /
  SpaceEvenly) and `ItemLineAlignment`. A wrapping list (90 of LBG's 250
  lists: shop rows, the player list, title tiles) was drawn as one long
  line running off its frame. Now a list breaks into lines at its parent's
  extent along the fill direction (the lines stacked by the same Padding),
  the flex alignments grow the items or spread them along a line and
  spread the lines across, Automatic / Start / Center / End / Stretch set
  the items within their line, and an AutomaticSize parent holds the
  lines (crates/rustudio-engine/src/gui_walk.rs `list_lines`; AbsoluteContentSize follows the
  drawn boxes; test `list_wraps_and_flexes`). Not done: UIFlexItem Shrink
  and a Fill line narrower than its items (nothing shrinks).
- **A layout inside a Folder laid out nothing**: `gui_walk.rs` flattened a
  Folder's GuiObjects into its parent's children and looked for a
  UIListLayout / UIGridLayout / UIPageLayout among the parent's own
  children only, so LBG's skill bar — `Skills.Container.Buttons`, a
  Folder holding the UIListLayout and the skill buttons cloned into it as
  skills are added — drew every button at the Container's top-left, one on
  top of the other. Now each group of GuiObjects (the parent's own, each
  Folder's) is placed by the layout among its owner's children, inside the
  parent's content, as Roblox does (`layout_group`; test
  `list_layout_in_a_folder`). Not done: an AutomaticSize parent measuring a
  Folder-held list's lines.
- **UIShadow was not drawn** (138 in LBG: glows behind its result frames,
  kill feed, close buttons). Now a Shape-mode shadow is drawn under its
  object's background: the box (with its UICorner) moved by Offset, grown
  by Spread, its edge softened over BlurRadius (scale of the shorter side),
  in Color at 1 - Transparency; ShowBehindParent false leaves out the part
  under the box (crates/rustudio-engine/src/gui_fx.rs `shadow`; test `ui_shadow_under_the_box`).
  Not done: Inset, Text mode, a UIGradient under the shadow.
- **No top bar over the game**: the Roblox menu (Esc) and the chat (`/`)
  had keys only, so a phone — no Esc, no `/` — never reached either, and a
  Studio playtest's view lacked the bar Studio draws in the 58 px inset.
  Now the CoreScripts' TopBar (Chrome layout, Constants.lua: 44 px
  buttons, 16 in from the side, 12 apart) is drawn over the game view in
  Play from Roblox's own TopBar textures (content/textures/ui/TopBar:
  iconBase.png as each button's plate, coloredlogo.png for the Roblox
  button, chatOn.png / chatOff.png for the chat button; the @2x / @3x
  sheets at a phone's density; while the menu is open the bar gives way
  to the round close button, as TopBarApp hides TopBarFrame for
  CloseMenuButtonRound; hand-drawn stand-ins until Studio's content is
  installed): the Roblox button opens and closes the menu, the unibar's
  chat button opens the chat window with its box focused (hidden with
  SetCoreGuiEnabled(Chat, false)). `StarterGui:SetCore("TopbarEnabled",
  false)` hides the bar and `SetCore("ChatActive", open)` opens / closes
  the chat, as the CoreScripts' registerSetCores does, and
  `GetCore("TopbarEnabled")` reads it back (TopbarPlus's IconController
  polls it; the other GetCore keys still return nil) (crates/rustudio-player/src/play_game.rs
  `top_bar`, crates/rustudio-engine/src/runtime_ext.rs; test `set_core_topbar_and_chat`). Not
  done: the unibar's other icons (captures, emotes, leaderboard,
  inventory), the health bar, notifications; a hidden bar keeps the 58 px
  GuiInset (Roblox drops it to 0 — ScreenGuis then start at the top).
- **A joined place's meshes and values were named after their class**:
  the replication stream leaves `Name` out of a NewInstance whose Name is
  still the constructor's, and `rustudio_net` filled the gap with the
  class name for every class but five (Event, Function, Wedge,
  CornerWedge, Union) — so a `SpecialMesh` the file calls `Mesh` arrived
  as `SpecialMesh`, a `StringValue` called `Value` as `StringValue`, and
  `part.Mesh` / `folder.Value` raised "Mesh is not a valid member of Part"
  in a place opened from Roblox while the same .rbxl opened from a file
  ran clean (LBG's `Wind1` on a cloned `ring`, its `MeshGroup.BeanWind.Mesh`).
  Now `default_instance_name` — one table, in
  `rustudio_core::doc` and again in `rustudio_net::model` (the net crate
  doesn't depend on core; `live_sync.rs` `default_name_tables_agree`
  keeps the copies equal) — holds Studio's constructor names (meshes
  "Mesh", the twelve ValueBase classes "Value", Lighting's effects
  "Bloom" / "Blur" / "ColorCorrection" / "ColorGrading" / "SunRays" /
  "DepthOfField", "Truss", "NegativePart", "Body Colors", "Shirt
  Graphic"; services keep their class name — `game.RunService` resolves
  by child name here); a Team Create join names what arrives by it, and
  `Instance.new` and Insert Object name new instances by it too
  (`part.Mesh` after `Instance.new("SpecialMesh", part)`). Not done:
  names only known from Studio itself for classes not in this list — a
  class that arrives as its class name but should not is a missing table
  row (tests `default_instance_names_are_studios`,
  `default_names_apply_to_direct_and_parked_children`,
  `instance_new_gives_studios_constructor_names`,
  `default_name_tables_agree`).
- **A short Team Create join played as if whole**: `rustudio_net`'s
  `Ready` is 1.5 s without an instance batch, not the server's end of
  transfer, and a child parked for a parent that never arrived, or
  instances still streaming when the server dropped the session, left no
  trace. The Output now counts the parked instances at Ready (a few
  named) and, at a disconnect, the instances that came after Ready or
  that the join never finished, each asking to reopen the place or open a
  saved .rbxl before playing (`model.rs` `parked_count`, `cloud_ui.rs`
  `ready_count`). (The LBG `BeanWind.Mesh` failure once blamed on this was
  the default-name gap above: the saved file holds 154,682 instances, not
  the 160,078 claimed, so the join had been complete.) Not done: the
  protocol's own end-of-join signal; a join that stalls under QUIC flow
  control while the phone is busy applying batches can still pass for
  quiet.
- **A server script's `Player.Character = rig` left the player with no
  controls.** Roblox treats the write as a spawn (the class page; staff on
  the DevForum, 2024: the old character is removed "the way things have
  worked since the beginning"): the old character's `CharacterRemoving`,
  then it goes, `Changed("Character")`, `CharacterAdded` with the rig on
  the server and on each client, the camera's `CameraSubject` on its
  Humanoid, the PlayerModule rebinding its controls to it — the custom-rig
  pattern games use (`CharacterAutoLoads = false`, a rig cloned from
  storage into Workspace, `player.Character = rig`). rustudio stored the
  property like any other: no event, so the camera and controls stayed on
  the old model (which the game then destroyed) and the rig stood in
  Workspace as an NPC, dead to input. Now the property's setter
  (crates/rustudio-engine/src/runtime.rs `newindex_routed`, Play's server)
  fires the old character's `CharacterRemoving` and `Changed` and hands
  the pair to the session, which makes the rig the character from where
  it stands, facing where it faces (crates/rustudio-player/src/play.rs
  `adopt_character`): out of the NPCs, Studio's limb collisions on its
  standard parts, the controller on it, the server's `CharacterAdded`,
  the old model out of Workspace and destroyed after it (a respawn's
  order), then the client's half once the rig has replicated
  (`character_client_half`: the property, the camera's `CameraSubject`,
  `CharacterAdded`, the rig's own LocalScripts started after it, as after
  a spawn). `Character = nil` leaves the player without one, the old
  character gone the same way. As on Roblox, nothing is cloned into the
  rig (no StarterCharacterScripts, Animate or Health — a custom rig brings
  its own) and the Backpack and PlayerGui stay: that is `LoadCharacter`'s
  work. Writing the model that is already the character changes nothing.
  Test `server_set_character_is_the_player_character` (rig from
  ReplicatedStorage, events on both sides, camera, the rig's LocalScript
  once, `Player:Move` walks it, then nil).

## Coverage of Studio's API

Measured by `rustudio --api-coverage OUT [GAPS…]` (src/coverage.rs): one
instance of every class in the dump (services at the root, the rest in a
folder, so even classes a script can't create are covered; abstract classes
on a concrete subclass; AnimationTrack / Tween / InputObject / Mouse made the
way scripts get them), every property read and every function and event
indexed in pcall through the real runtime. A member is implemented when that
worked and the telemetry recorded no gap (`missing`, `default-only`), minus
`call-unimplemented` seen in the real runs. Functions are indexed, not
called.

| | Round 12 start | Round 12 end | Of which game scripts may use (Security None) |
|---|---|---|---|
| Functions | 314 / 3402 | 344 / 3402 | 340 / 1626 |
| Properties | 2833 / 3806 | 2873 / 3806 | 2274 / 2802 |
| Events | 172 / 849 | 172 / 849 | 172 / 410 |

Round 13 end: functions **346 / 3402** (game scripts 342 / 1626),
properties **2875 / 3806** (2276 / 2802), events 172 / 849 (172 / 410)
(`../checks/coverage/round-13.json`, gap files `gaps-*-r13.json`).

Implemented this round from the list and the next most-used families:
WorldRoot:Spherecast / Blockcast / Shapecast; the collision-group functions on
WorldRoot (and PhysicsService:GetMaxCollisionGroups / GetCollisionGroupName /
CollisionGroupContainsPart); GuiObject:TweenPosition / TweenSize /
TweenSizeAndPosition; Humanoid:GetLimb / GetBodyPartR15 / GetMoveVelocity;
Instance:GetActor; BasePart:GetClosestPointOnSurface / GetJoints /
GetNoCollisionConstraints / GetRenderCFrame / ApplyImpulseAtPosition and
ExtentsCFrame / ExtentsSize / CenterOfMass / AssemblyCenterOfMass;
Lighting:GetSunDirection / GetMoonDirection / GetMoonPhase; Model.WorldPivot
(PrimaryPart's pivot, else WorldPivotData, else the bounds' centre);
Mouse.hit / target / TargetSurface; Tween.Instance / TweenInfo; Player
run-time properties (DisplayName, MembershipType, AccountAge, LocaleId,
CanLoadCharacterAppearance, …); Players.NumPlayers / localPlayer /
players(); DataModel JobId / PrivateServerId / CreatorType / Genre /
MatchmakingType; Attachment.Rotation / WorldRotation.

The measurements are kept in `../checks/coverage/` (`round-12-start.json`,
`round-12.json`, and the before / after gap files of each place).

## Render divergence (after Place A plays cleanly)

Round 12 fidelity scores (render, settled shots): Place A 49.5, demo 69.3
(was 64.2), control 68.5 (was 67.8); overall 62.4.

- **Bloom**: done. Studio glows over-bright pixels (Neon) with no
  BloomEffect in the place; rustudio's bloom pass now runs with BloomEffect's
  defaults (Intensity 1, Size 24, Threshold 2) when there isn't one. The
  control shot's halo around the Neon bar matches within ~8 levels.
- **Shadows**: the sun shadow map's darkness was already within 2 levels of
  Studio (self-test `lighting-day`); in the aligned control frames Studio's
  character shadow is softer and lighter at its edge. Not changed this round.
- **Glass**: open. A Transparency-0.5 Glass pane is almost invisible in
  Studio and a visible light-blue pane in rustudio (demo and control); needs
  a Studio calibration scene (transparency steps over light and dark).
- **Place A chapel brightness** (Soft spot lights) and the **skybox / stud scale**
  remain open.

Round 13 fidelity (`fidelity.py run --round 13`): render Place A 49.5, demo
69.3, control 68.5 (unchanged), all gates pass (MAE 32.4 / 21.7 / 23.1).
Place A camera_controls read 75: the menu camera looked 2.18° off against this
run's Studio capture (tolerance 2°); the round-12 build measures the same
2.18° against it, so it comes from the Studio reference's timing, not from
this round. A night calibration scene (Place A's chapel lighting in a plain
room, `tools/fidelity/corpus/night.rbxlx`) measured MAE 20.4 and located
the chapel gap in Soft spot lights; the spawn ForceField bubble Studio
draws at 10 s is the other visible gap in demo and control.

Harness determinism fixed on the way (not loosened): Studio spawns the
character at a random point on the SpawnLocation, so both engines now get a
prelude that puts it on the centre (demo settled MAE 40 → 21.7; thresholds
lowered to demo 24, control 25); Place A's menu camera sways on `os.clock()`, so
rustudio's `os.clock` now counts like Studio's (from the process start) and
the harness passes Studio's reading at Play (`RUSTUDIO_CLOCK_BASE`): Place A
camera 75 → 100 with the 2° tolerance unchanged.

## Roblox's CoreScripts from the data model patch (opt-in)

`RUSTUDIO_CORE_SCRIPTS=1` (or Studio Settings > rustudio > Play > Roblox's
CoreScripts) makes Play put Studio's `DataModelPatch.rbxm` in the client
(crates/rustudio-core/src/core_scripts.rs) and start its `StarterScript` at
the CoreScript identity (4). On the control place (Studio
version-8d7a1009c17e4ff8's patch: 49190 modules, 79 CoreScripts, about
350 ms to load) StarterScript now runs to its end with no CoreScript error
in Output: Roblox's top bar draws (Roblox menu, chat and "more" buttons in
the Builder Icons font) and so does the player list; rustudio's own menu,
chat, hotbar and emote wheel stay hidden while it's on.

Closed on the way (API dump members; stubs only where a Play session has no
backend, each noted in the code):

- ScriptContext:AddCoreScriptLocal; DataModel DefineFast*/GetFast*/
  GetEngineFeature/SetFlagVersion/GetPlaySessionId; settings() flags for
  CoreScripts.
- GuiService: SetTopbarInset/TopbarInset, SetMenuIsOpen, SetGlobalGuiInset,
  GetResolutionScale and the UI-scale/safe-zone/hardware-viewport getters,
  error getters, Add/RemoveCenterDialog, Add/Remove(Special)Key,
  GetNotificationTypeList, SendCoreUiNotification (any API-dump callback is
  now kept).
- StarterGui:RegisterSetCore / RegisterGetCore behind SetCore / GetCore;
  ContextActionService core actions (ahead of game actions).
- MessageBusService, MemStorageService, LocalStorageService /
  AppStorageService, IXPService (layers loaded, no variables),
  HttpService:RequestInternal (roblox.com APIs through the login session).
- RunService / UserInputService / Player / PlayerScripts / UserGameSettings /
  RenderSettings / VRService / AvatarChatService / ThirdPartyUserService
  members the CoreScripts read; HumanoidDescription.NumberEmotesLoaded;
  InsertService:LoadLocalAsset; UDim2:lerp; LocalizationTable:SetEntries
  without Source; analytics, ads, notifications, achievements and capture
  gallery as stubs.
- Rendering: CoreGui ScreenGuis (also inside Folders) draw above PlayerGui;
  GSUB ligatures for icon fonts; rbxasset:// reaches Studio's ExtraContent.

Client flags (crates/rustudio-core/src/client_flags.rs): Play loads
Roblox's public PCDesktopClient settings (25 850 flags) and the CoreScripts
take today's variants. With all of them, on a TextChatService copy of the
control place, StarterScript and the CoreScripts start with no error in
Output; about 8 s into Play the "Chrome" top bar draws (Roblox logo, menu
and chat buttons), with ExpChat's chat window and input bar and the newer
player list. Clicking the menu button opens Chrome's menu (Switch avatar,
Music, Report, Leaderboard, Emotes, Inventory, Respawn); clicking the input
bar, typing and Enter puts the message in the chat window. Closed for this:
Player.ChatAvailabilityStatus, AudioFocusService, SessionService,
ExperienceService, TelemetryService (stubs), LinkingService,
HttpService:GetUserAgent, Translator:RobloxOnlyTranslate,
GuiService:GetClosestDialogToPosition / GetClosestVisibleDialogToPosition;
SetCore / GetCore handlers run at their registrant's identity (a game
LocalScript's SetCore reaching a CoreScript handler that needs LocalUser).
Engine features (GetEngineFeature) have no public source and read false,
except AvatarChatServiceEnabled: AvatarChatService answers a client without
voice or camera chat (ClientFeatures 0, IsEnabled tests a bit of a mask,
EnableVoice false), VideoCaptureService:GetCameraDevices an empty list, and
VoiceChatCore's CoreVoiceManager (which needs the service) no longer errors.
AvatarEditorService:BustAvatarFetchCache (the menu's Respawn) fetches the
signed-in avatar again for the next character. String properties, names,
attributes and string arguments take any Lua bytes, as Roblox does; bytes
that aren't UTF-8 are left out (estimate: rustudio holds UTF-8 text).

Menu and styles (round 2): Foundation (the CoreScripts' UI kit) styles its
objects through StyleSheets with tags (".size-full", ".row ::UIListLayout").
Fixed in gui_input.luau: tags and names with "-" in selectors; pseudo-instance
selectors ("::UIListLayout", "::UIPadding", "::UIFlexItem", "::UICorner"),
which make and style one modifier per matched element; rule data held
strongly (a parented StyleRule's Lua object could be collected and its
properties lost); equal-priority rules applied in sheet order. With those
the settings menu's Leave / Respawn / Resume row draws as in Roblox and
Resume closes the menu. AutomaticSize XY frames lay their content out again
at the width they settle on (ExpChat's translation notice was 300x400; now
its text height). TextService:GetTextSize / GetTextBoundsAsync measure with
rustudio's text layout; GetTextSizeOffsetAsync answers 0 (Medium text,
estimate). rbxthumb:// thumbnails (the People tab's headshot) come from the
thumbnails API. Lua signals and render-step binds run at their connector's
identity.

Performance: style matching ran over every descendant of each StyleLink's
parent every frame, RobloxGui's 45 000 CoreScript modules included. It now
skips scripts and their children; destroyed-instance sweeps run twice a
second. Round 3: matching reads each styled tree in one engine call
(gui_input.rs `style_tree`) and only the elements holding a selector's
first tag, and reruns per StyleLink only when the DataModel's journal shows
its tree changed (an element added / moved / removed, a Name / Tags /
Interactable write), for all links when a sheet, rule, query or the view
changes (and hover / press / focus when a rule has a :state), with no 2 s
timer. A rerun with the settings menu open: 150-250 ms -> 8-20 ms (debug).

Look (round 3, compared with the real client's captures):
- Styles no longer override what an instance sets itself (Roblox's styling
  guide, "Modified properties"): Foundation's icons keep their Builder Icons
  FontFace and TextSize over the "text-defaults" / "x-default-text-size"
  rules, so `microphone`, `arrow-medium-up` and the rest draw as glyphs
  (they drew as Builder Sans 8 text).
- RobloxGui is ZIndexBehavior Sibling: the settings menu's Leave / Respawn /
  Resume row and the Help tab's grid draw over the menu's background, not
  dimmed under it.
- The default chat PrefixText is `<font color="#…">DisplayName:</font>` in
  the team colour, else the CoreScripts' computeNameColor colour;
  RobloxEmoji's colour glyphs (the verified badge U+E000) draw in their COLR
  layers' colours (gui_color_glyphs.rs); rich text `color = "…"` with spaces
  parses.
- Text outlines: a run's own transparency fades its outline (ExpChat's
  invisible copy of the name drew a black outline over it), and the eight
  outline copies share the stroke's alpha (mech GUI_TEXT_STROKE_OVERLAP).
- A row's flex-grown items that grow on Y are as tall as they are at their
  new width (the chat input grew to 182 px while typing).
- An AutomaticSize X object's children measure with the room it can grow
  into (the Help tab's key column, AutomaticSize XY frames 0 wide around
  wrapped "Up arrow", wrapped a letter per line).
- TextFits before the first layout measures TextBounds against the box
  (SettingsHub's version footer turned TextScaled on and drew over itself).
- Setting Font sets FontFace to Font.fromEnum of it, and Font.fromEnum maps
  as Roblox's does (BuilderSansMedium → BuilderSans Medium, Gotham →
  GothamSSm): the settings rows set Font = BuilderSans and drew in
  LegacyArial at its 1.5× size. Section headers are text-heading-small
  (Builder Sans Bold 25, the Foundation token) in content-emphasis white.

Still open: the Respawn confirmation page draws its text tiny.
TextService:GetTextSizeOffsetAsync answers 0 (Medium, estimate; see
above). The Help tab wraps
"Drop tool", "Use tool" and "Mouse lock" (Roblox doesn't) and its Camera
movement / Menu items groups are not on screen; setting FontFace doesn't
set Font back; `<stroke>` rich text tags are read but not drawn.

A game's top bar beside Roblox's (round 4, Legends Battlegrounds with the
CoreScripts on, on a phone):
- The settings menu's Settings page errored at `Utility:1429` ("invalid
  argument #1 to 'pairs'"): GameSettings' updateAudioDevices pcalls
  `SoundService:GetOutputDevices()` / `GetInputDevices()`, and when the
  call fails it hands the error string to UpdateOptions as the option
  list. rustudio had none of SoundService's device members, so the
  CoreScripts' call fell to the generic "lacking capability RobloxScript".
  Now (domains/media.rs) `GetOutputDevices` / `GetInputDevices` answer the
  default device's name as a one-entry list with the empty GUID the page
  labels "Default (...)" and index 1; `GetOutputDevice` / `GetInputDevice`
  the same name and GUID; `SetOutputDevice` is a no-op (one device: the
  mixer plays through cpal's default output); the camera selector is behind
  the VideoCaptureService engine feature, which reads false.
- TopbarPlus's icons drew over the Chrome top bar's menu and chat buttons.
  TopbarPlus is a ScreenGui with ScreenInsets TopbarSafeInsets, placed by
  the engine at this build's own bar (mech GUI_TOPBAR_LEFT_CHAT: 128 px
  from the left), while the Chrome unibar reaches 164 px. The CoreScripts
  say where the free area is: CoreGuiCommon's keep-out-area store (and the
  older KeepOutAreasHandler) takes the unibar's, menu icon's and health
  bar's boxes and calls `GuiService:SetTopbarInset(Rect)` with the widest
  strip between them. GuiService.TopbarInset already returned that rect;
  now it also places TopbarSafeInsets ScreenGuis (World::topbar_inset ->
  the play view -> Gui::topbar_inset in gui_input.rs, runtime_ext.rs's
  AbsolutePosition and GetInsetArea): with the CoreScripts on, TopbarPlus
  starts at 164 px, after the chat button (test
  core_scripts_topbar_inset_places_topbar_safe_insets_guis). The Chrome
  bar mounts a few seconds into Play (React renders it as its modules
  load; 12-15 s on the 2-CPU test machine); until then TopbarPlus sits at
  the first inset (0, the store's before any area), as on Roblox.
- `StarterGui:SetCore("TopbarEnabled" / "ChatActive")` and
  `GetCore("TopbarEnabled")` went to the CoreScripts' SetCore / GetCore
  registry even when no CoreScript had registered the key, which swallowed
  them (GetCore("TopbarEnabled") nil; TopbarPlus's IconController polls
  it, and rustudio's own top bar never hid). Those two keys now fall
  through to the engine when unregistered (domains/services.rs).

## Next (open, by how much they block real games)

1. **Soft spot lights** (render): Studio spreads a SpotLight's light well
   outside its cone in LightingStyle Soft (night calibration, MECHANICS.md
   round 13); the Place A chapel's darkness comes mostly from this.
   **ForceField visual**: Studio draws the spawn bubble at 10 s in demo and
   control; rustudio draws none. Terrain editing from scripts (FillBlock …).
2. **PathfindingService:CreatePath / Path:ComputeAsync / GetWaypoints** —
   most NPC games.
3. **TextService:GetTextSize / GetTextBoundsAsync** — UI layout code.
4. ~~GuiService.MenuOpened / MenuClosed with an in-game Esc menu~~ (round 13).
5. Gamepad input (GamepadConnected / Disconnected, gamepad KeyCodes).
6. The rest of `missing_game_members_by_class` in the coverage report.
7. ~~Message / Hint drew nothing~~ (place 2828's regen scripts show
   "Regenerating …" with one): drawn as Studio 0.747's `Message::render2d` /
   `Hint::render2d` do (MECHANICS.md, GUI).
8. Model / BasePart `MakeJoints` is a no-op (Roblox makes Snap / Weld joints
   from touching parts' SurfaceTypes); joints saved in a model still load as
   instances.
9. ~~Touch for games~~ (Android, docs/ANDROID.md "Fingers in Play"): on a
   touch screen `UserInputService.TouchEnabled` / `TouchScreenEnabled` /
   `PreferredInput == Touch`, one InputObject per finger through
   `InputBegan / Changed / Ended` and `TouchStarted / Moved / Ended`,
   ContextActionService `UserInputType.Touch` bindings, the input action
   system's `TouchPosition / TouchDelta / TouchPinch` with `UIButton` /
   `UIModifier`, GuiObject `InputBegan(Touch)` / `TouchTap`, the gestures
   (`TouchTap`, `TouchTapInWorld`, `TouchLongPress`, `TouchPan`,
   `TouchPinch`, `TouchRotate`, `TouchSwipe`) and the motion sensors
   (`AccelerometerEnabled` / `GyroscopeEnabled`, `GetDeviceAcceleration /
   Gravity / Rotation`, `Device*Changed`). Still open: GuiObject
   `TouchLongPress / Pan / Pinch / Rotate / Swipe` (only
   UserInputService's fire), `UserInputService.TouchDrag`, and the
   gestures on a desktop (no finger reaches the game there: the editor's
   mouse path stays). The soft keyboard over a game's TextBox:
   `OnScreenKeyboardVisible / Position / Size` in the game's
   coordinates and `TextBox.ReturnPressedFromOnScreenKeyboard`
   (src/viewport.rs `keyboard_pan`, crates/rustudio-engine/src/runtime_ext.rs `device_prop`;
   zero and never on a desktop, as in Studio).
