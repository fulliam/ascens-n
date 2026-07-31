use crate::World;

/// A callable system that operates on the world
pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, world: &mut World);
}

/// Systems grouped and ordered for execution
pub struct Schedule {
    stages: Vec<Stage>,
}

pub struct Stage {
    pub name: String,
    pub systems: Vec<Box<dyn System>>,
}

impl Stage {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), systems: Vec::new() }
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }
}

impl Schedule {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Creates a schedule pre-populated with the six standard game stages,
    /// in execution order, all empty. Game systems are added later (Stage 3
    /// of the roadmap) via `add_system_to`, one module at a time, without
    /// ever touching this ordering again.
    /// Creates a schedule pre-populated with the twelve fixed pipeline
    /// stages (Stage 3 roadmap), in execution order, all empty. Replaces
    /// the original six-stage list from Stage 1 — that name set is gone,
    /// not kept alongside this one, since the TS Runtime now drives this
    /// exact pipeline 1:1 (see `packages/runtime/src/stage.ts`).
    pub fn standard_game_schedule() -> Self {
        let mut schedule = Self::new();
        for name in [
            "Startup",
            "PreFrame",
            "Input",
            "PrePhysics",
            "Physics",
            "PostPhysics",
            "AI",
            "Gameplay",
            "Animation",
            "Audio",
            "Render",
            "PostFrame",
        ] {
            schedule.add_stage(Stage::new(name));
        }
        schedule
    }

    pub fn add_stage(&mut self, stage: Stage) {
        self.stages.push(stage);
    }

    /// Add a system to an existing stage, looked up by name.
    ///
    /// Panics if no stage with that name exists yet — call `add_stage` (or
    /// start from `standard_game_schedule`) first. This is intentionally a
    /// hard failure rather than silently creating the stage: stage order
    /// matters, and auto-creating an unordered stage on first use would
    /// hide a real configuration mistake.
    pub fn add_system_to<S: System + 'static>(&mut self, stage_name: &str, system: S) {
        let stage = self
            .stages
            .iter_mut()
            .find(|s| s.name == stage_name)
            .unwrap_or_else(|| panic!("Schedule::add_system_to: stage '{}' not found — call add_stage() first", stage_name));
        stage.add_system(system);
    }

    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|s| s.name.as_str()).collect()
    }

    pub fn run(&self, world: &mut World) {
        for stage in &self.stages {
            for system in &stage.systems {
                system.run(world);
            }
        }
    }

    /// Run only the systems belonging to a single named stage. No-op if the
    /// stage doesn't exist (lenient on purpose — this is the method the
    /// WASM layer calls every host frame/tick, and a typo'd stage name
    /// should not crash the game loop).
    pub fn run_stage(&self, stage_name: &str, world: &mut World) {
        if let Some(stage) = self.stages.iter().find(|s| s.name == stage_name) {
            for system in &stage.systems {
                system.run(world);
            }
        }
    }

    pub fn system_count(&self) -> usize {
        self.stages.iter().map(|s| s.systems.len()).sum()
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: wrap a closure as a system
pub struct FnSystem {
    name: String,
    f: Box<dyn Fn(&mut World) + Send + Sync>,
}

impl FnSystem {
    pub fn new(name: impl Into<String>, f: impl Fn(&mut World) + Send + Sync + 'static) -> Self {
        Self { name: name.into(), f: Box::new(f) }
    }
}

impl System for FnSystem {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, world: &mut World) {
        (self.f)(world);
    }
}
