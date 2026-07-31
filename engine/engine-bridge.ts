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
