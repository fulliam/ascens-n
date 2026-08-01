// engine/engine-bridge.ts
//
// Narrow, typed wrapper around the wasm-bindgen-generated `ecs-core` glue.
// See .tools/docs/engineering/WASM_ECS_MIGRATION_PLAN.md (Phase 0/1) for full
// context. This is the ONLY file in the project that talks directly to the
// raw, byte-offset/component-id-indexed wasm API (`get_f32(id, gen, compId,
// byteOffset)` etc.) — every other script, including the classic game script
// in index.html, reaches the engine only through `window.EngineBridge`'s
// named, typed helpers.
//
// This file is compiled standalone by `tsc` (see engine/tsconfig.json) — no
// bundler, no npm packaging. `.tools/build-wasm.sh` inlines the wasm-bindgen
// JS glue (`ecs_core.js`) verbatim into the SAME `<script type="module">`
// block as this file's compiled output, so there is no `import`/module
// resolution at runtime: `JsWorld` and the wasm init entry point are plain
// top-level declarations already in scope by the time this code runs. The
// two `declare`s below describe exactly that shape without an actual
// `import` statement — an `import` would need a real separate file on disk
// at a real relative path, which contradicts the whole point of
// base64-embedding everything into one index.html.
//
// `__engine_wasm_init` is NOT wasm-bindgen's literal generated name (that's
// currently `__wbg_init`, an internal name that could change between
// versions). The build script aliases whatever the glue's actual
// `export default` binding is to this stable name in one generated line
// between the glue and this file's output, so this source file never has to
// know or guess wasm-bindgen's internal naming — only the build script does,
// and only by reading it fresh out of the glue at build time.
declare function __engine_wasm_init(input: Uint8Array): Promise<unknown>;

// Mirrors the subset of `JsWorld`'s real (wasm-bindgen-generated, see
// crate-engine/src/wasm_api.rs) surface this bridge actually uses. Kept
// narrow on purpose — see file header.
declare class JsWorld {
  constructor();
  register_f32x2(name: string): number;
  entity_count(): number;
  spawn_batch_pos_vel(
    posId: number, px: number, py: number,
    velId: number, vx: number, vy: number,
    count: number
  ): Uint32Array;
  step_physics(posId: number, velId: number, dt: number): void;
  get_f32x2_interleaved(componentId: number): Float32Array;
  // Spatial broadphase (crate-engine/src/wasm_api.rs — Phase 2 of the migration
  // plan). Rust-side grid is already written and unit-tested; these three were
  // simply never exposed through this bridge until now.
  init_spatial_grid(componentId: number, worldW: number, worldH: number): void;
  rebuild_spatial_index(componentId: number): void;
  query_near(x: number, y: number, radius: number, componentId: number): Uint32Array;
  query_near_count(x: number, y: number, radius: number, componentId: number): number;
  // Batch spawn (all N entities start at the same x/y — fine here since every
  // position gets overwritten by the first sync_enemy_echo call anyway) and
  // bulk field write (Phase 3 step 2 — "physics results -> ECS" per the
  // doc comment on set_component_x/y_values, repurposed here for "JS enemy
  // positions -> ECS" instead of a physics system).
  spawn_batch_f32x2(componentId: number, x: number, y: number, count: number): Uint32Array;
  set_component_x_values(componentId: number, values: Float32Array): void;
  set_component_y_values(componentId: number, values: Float32Array): void;
  // Needed to translate query_near's [id,generation,...] pairs back to a
  // stable pool slot index — see queryEnemyEchoNear below for why.
  entity_generation(entityId: number): number;
}

// The build script emits `const WASM_B64 = "...";` before inlining this
// file's compiled output — declared here purely so `tsc` type-checks the
// reference; the real value only exists in the assembled <script> block.
declare const WASM_B64: string;

// ---------------------------------------------------------------------
// base64 -> bytes. Local to this file rather than a shared util — this is
// the one and only place in the project that needs it.
function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

// ---------------------------------------------------------------------
interface EngineBridgeApi {
  isReady(): boolean;
  entityCount(): number;
  spawnTestEntity(x: number, y: number, vx: number, vy: number): number;
  tickPhysics(dt: number): void;
  readAllPositions(): Float32Array;
  // Spatial broadphase, scoped to the Position component this bridge already
  // tracks (posComponentId is baked in below, never exposed to the caller —
  // there is only ever one spatial grid this bridge manages, so surfacing a
  // component id parameter here would just be a chance to pass the wrong one).
  initSpatialGrid(worldW: number, worldH: number): void;
  rebuildSpatialIndex(): void;
  queryNear(x: number, y: number, radius: number): Uint32Array;
  queryNearCount(x: number, y: number, radius: number): number;
  // Phase 3 step 2 (WASM_ECS_MIGRATION_PLAN.md) — a *parallel, read-only*
  // mirror of real JS enemy positions, for validating the Rust spatial index
  // against real gameplay data before anything is allowed to depend on it.
  // Deliberately does NOT touch real enemies/forEachEnemyNear/dealDamage —
  // see the comment above initEnemyEcho below for the full design rationale.
  initEnemyEcho(worldW: number, worldH: number): void;
  syncEnemyEcho(xs: Float32Array, ys: Float32Array, liveCount: number): void;
  queryEnemyEchoNearCount(x: number, y: number, radius: number): number;
  // Phase 3 step 3, real call-site cut (WASM_ECS_MIGRATION_PLAN.md) — unlike
  // queryEnemyEchoNearCount above (diagnostic count-only, compared against a
  // JS brute-force ground truth), this returns the actual candidate slot
  // indices so a caller can map them back to real enemy objects and act on
  // them. Only safe to use where the caller already re-checks exact hit
  // distance/radius downstream (see index.html's bulletBroadphaseNear) —
  // this performs EXACT-circle filtering internally (see wasm_api.rs), which
  // is stricter than forEachEnemyNear's cell-bounding-box approximation, so
  // it must never replace a call site that relies on that looser semantics
  // without its own distance check.
  queryEnemyEchoNear(x: number, y: number, radius: number): Uint32Array;
}

// No top-level import/export in this file (see header) — tsc treats it as
// a global script, so this ambient interface merges directly with lib.dom's
// `Window` without needing a `declare global` wrapper (that form is only
// required inside actual modules).
interface Window {
  EngineBridge?: EngineBridgeApi;
}

let world: JsWorld | null = null;
let posComponentId = -1;
let velComponentId = -1;
let ready = false;

function spawnTestEntity(x: number, y: number, vx: number, vy: number): number {
  if (!world) return -1;
  const ids = world.spawn_batch_pos_vel(posComponentId, x, y, velComponentId, vx, vy, 1);
  return ids.length > 0 ? ids[0] : -1;
}

function tickPhysics(dt: number): void {
  if (!world) return;
  world.step_physics(posComponentId, velComponentId, dt);
}

function readAllPositions(): Float32Array {
  if (!world) return new Float32Array(0);
  return world.get_f32x2_interleaved(posComponentId);
}

function entityCount(): number {
  return world ? world.entity_count() : 0;
}

function initSpatialGrid(worldW: number, worldH: number): void {
  if (!world) return;
  world.init_spatial_grid(posComponentId, worldW, worldH);
}

function rebuildSpatialIndex(): void {
  if (!world) return;
  world.rebuild_spatial_index(posComponentId);
}

function queryNear(x: number, y: number, radius: number): Uint32Array {
  if (!world) return new Uint32Array(0);
  return world.query_near(x, y, radius, posComponentId);
}

function queryNearCount(x: number, y: number, radius: number): number {
  if (!world) return 0;
  return world.query_near_count(x, y, radius, posComponentId);
}

// Phase 3 step 2 — validating the Rust spatial index against real enemy
// positions, without letting anything in the classic game script actually
// depend on the result yet (that's the whole point of the diff-validation
// gate the migration plan asks for before any real call site is replaced).
//
// Design: rather than spawn/despawn one Rust entity per JS enemy every frame
// (churny, and entity ids would need a JS<->Rust id-mapping table just to
// keep them in sync across kills/spawns), spawn a FIXED pool once, sized to
// MAX_ENEMIES (index.html), and every frame bulk-overwrite its positions via
// set_component_x/y_values — this is exactly the "physics results -> ECS"
// bulk-write pattern those two methods were already built for, just fed by
// JS's enemy array instead of a physics system. Unused pool slots (when
// enemies.length < pool size, the common case) get parked at a sentinel
// coordinate far outside the world so they can never spuriously match a
// proximity query.
const ENEMY_ECHO_POOL_SIZE = 1600; // matches index.html's MAX_ENEMIES
const ENEMY_ECHO_SENTINEL = -1e6; // for both x and y — outside any real query radius
let enemyEchoComponentId = -1;
let enemyEchoInited = false;
// query_near returns raw (entity id, generation) pairs, not the pool's
// insertion-order slot index a JS caller actually wants (to index back into
// its own parallel "live enemy" list — see index.html's
// syncEnemyEchoLive/bulletBroadphaseNear). Built once at pool-creation time:
// the pool's entities are only ever repositioned via set_component_x/y_values
// afterward, never despawned/respawned, so id->slot/generation stays valid
// for the pool's entire lifetime — no per-frame rebuild needed.
let enemyEchoIdToSlot: Map<number, { slot: number; gen: number }> | null = null;

function initEnemyEcho(worldW: number, worldH: number): void {
  if (!world || enemyEchoInited) return;
  enemyEchoComponentId = world.register_f32x2('EnemyEcho');
  const ids = world.spawn_batch_f32x2(enemyEchoComponentId, ENEMY_ECHO_SENTINEL, ENEMY_ECHO_SENTINEL, ENEMY_ECHO_POOL_SIZE);
  enemyEchoIdToSlot = new Map();
  for (let i = 0; i < ids.length; i++) {
    enemyEchoIdToSlot.set(ids[i], { slot: i, gen: world.entity_generation(ids[i]) });
  }
  world.init_spatial_grid(enemyEchoComponentId, worldW, worldH);
  enemyEchoInited = true;
}

function syncEnemyEcho(xs: Float32Array, ys: Float32Array, liveCount: number): void {
  if (!world || !enemyEchoInited) return;
  const n = Math.min(liveCount, ENEMY_ECHO_POOL_SIZE, xs.length, ys.length);
  const fullX = new Float32Array(ENEMY_ECHO_POOL_SIZE).fill(ENEMY_ECHO_SENTINEL);
  const fullY = new Float32Array(ENEMY_ECHO_POOL_SIZE).fill(ENEMY_ECHO_SENTINEL);
  fullX.set(xs.subarray(0, n));
  fullY.set(ys.subarray(0, n));
  world.set_component_x_values(enemyEchoComponentId, fullX);
  world.set_component_y_values(enemyEchoComponentId, fullY);
  world.rebuild_spatial_index(enemyEchoComponentId);
}

function queryEnemyEchoNearCount(x: number, y: number, radius: number): number {
  if (!world || !enemyEchoInited) return 0;
  return world.query_near_count(x, y, radius, enemyEchoComponentId);
}

function queryEnemyEchoNear(x: number, y: number, radius: number): Uint32Array {
  if (!world || !enemyEchoInited || !enemyEchoIdToSlot) return new Uint32Array(0);
  const raw = world.query_near(x, y, radius, enemyEchoComponentId); // [id0,gen0,id1,gen1,...]
  const slots: number[] = [];
  for (let i = 0; i + 1 < raw.length; i += 2) {
    const entry = enemyEchoIdToSlot.get(raw[i]);
    if (entry && entry.gen === raw[i + 1]) slots.push(entry.slot);
  }
  return Uint32Array.from(slots);
}

async function boot(): Promise<void> {
  try {
    await __engine_wasm_init(base64ToBytes(WASM_B64));
    world = new JsWorld();
    posComponentId = world.register_f32x2('Position');
    velComponentId = world.register_f32x2('Velocity');
    ready = true;

    // Exposed only once init has actually succeeded — the classic game
    // script and the console must never observe a half-initialized bridge.
    window.EngineBridge = {
      isReady: () => ready,
      entityCount,
      spawnTestEntity,
      tickPhysics,
      readAllPositions,
      initSpatialGrid,
      rebuildSpatialIndex,
      queryNear,
      queryNearCount,
      initEnemyEcho,
      syncEnemyEcho,
      queryEnemyEchoNearCount,
      queryEnemyEchoNear,
    };
    window.dispatchEvent(new Event('engine-ready'));
  } catch (err) {
    // Strictly additive capability — a failed wasm load must never break
    // the classic game script, so this is a console error, never a throw.
    ready = false;
    // eslint-disable-next-line no-console
    console.error('[engine-bridge] WASM init failed — engine features disabled, game continues without them:', err);
  }
}

boot();
