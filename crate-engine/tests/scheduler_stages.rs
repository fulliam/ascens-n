use ecs_core::*;

struct CounterSystem;
impl System for CounterSystem {
    fn name(&self) -> &str { "counter" }
    fn run(&self, world: &mut World) {
        let _ = world.archetype_count();
    }
}

const PIPELINE_STAGES: [&str; 12] = [
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
];

#[test]
fn standard_game_schedule_has_twelve_stages_in_order() {
    let schedule = Schedule::standard_game_schedule();
    assert_eq!(schedule.stage_names(), PIPELINE_STAGES.to_vec());
}

#[test]
fn add_system_to_named_stage() {
    let mut schedule = Schedule::standard_game_schedule();
    schedule.add_system_to("Gameplay", CounterSystem);
    schedule.add_system_to("Gameplay", CounterSystem);
    schedule.add_system_to("Physics", CounterSystem);
    assert_eq!(schedule.system_count(), 3);
}

#[test]
#[should_panic(expected = "stage 'NoSuchStage' not found")]
fn add_system_to_unknown_stage_panics() {
    let mut schedule = Schedule::standard_game_schedule();
    schedule.add_system_to("NoSuchStage", CounterSystem);
}

#[test]
fn run_stage_only_runs_that_stage() {
    use std::sync::{Arc, Mutex};

    struct RecordingSystem {
        log: Arc<Mutex<Vec<&'static str>>>,
        tag: &'static str,
    }
    impl System for RecordingSystem {
        fn name(&self) -> &str { self.tag }
        fn run(&self, _world: &mut World) {
            self.log.lock().unwrap().push(self.tag);
        }
    }

    let log = Arc::new(Mutex::new(Vec::new()));
    let mut world = World::new();
    world.schedule.add_system_to("Physics", RecordingSystem { log: log.clone(), tag: "physics" });
    world.schedule.add_system_to("Gameplay", RecordingSystem { log: log.clone(), tag: "gameplay" });

    world.run_stage("Physics");
    assert_eq!(*log.lock().unwrap(), vec!["physics"]);

    world.run_stage("Gameplay");
    assert_eq!(*log.lock().unwrap(), vec!["physics", "gameplay"]);
}

#[test]
fn unknown_stage_name_is_a_lenient_no_op() {
    let mut world = World::new();
    world.run_stage("DoesNotExist"); // must not panic
    assert_eq!(world.archetype_count(), 0);
}

#[test]
fn world_starts_with_standard_schedule_already_installed() {
    let world = World::new();
    assert_eq!(world.stage_names(), PIPELINE_STAGES.to_vec());
}
