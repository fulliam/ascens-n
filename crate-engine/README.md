# ECS Engine

## Structure

```
crate-engine/          Rust ECS (wasm-pack target)
```

## Build

### 1. WASM (requires wasm-pack and Rust nightly or stable ≥ 1.75)

```bash
cd crate-engine
wasm-pack build --target web --features wasm --out-dir ../runtime/apps/game/wasm
```

`./build.sh --skip-wasm` skips the Rust step if the WASM is already built.

## Architecture

```
Rust WASM (ECS core: archetypes, queries, resources, events, schedule)
  ↓
@engine/runtime  (World, Scheduler, Commands, EventBus, ResourceContainer, Query API)
  ↓
@engine/core     (Position, Rotation, Scale, Parent, GlobalTransform, Lifetime)
  ↓
@engine/gameplay (Velocity, Acceleration, Health, Damage, Movement/Damage/Death systems)
  ↓
apps/game        (PlayerControl, EnemyChase, Collision, Spawn, Canvas renderer)
```

## Add a new game system

```ts
// packages/gameplay/src/my-system.ts
import { Query, type System, type SystemContext } from '@engine/runtime';

export class MySystem implements System {
  readonly name = 'MySystem';
  run(ctx: SystemContext): void {
    Query.queryEach(ctx.world, [SomeComponent], (id, generation) => {
      // read/write without allocations
    });
  }
}
```

Register in a Plugin and `runtime.addPlugin(MyPlugin)` — that's all.
