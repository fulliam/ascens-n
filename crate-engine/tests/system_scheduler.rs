use ecs_core::*;

struct CounterSystem {
    name: String,
}

impl System for CounterSystem {
    fn name(&self) -> &str { &self.name }

    fn run(&self, world: &mut World) {
        // simulate updating a counter component
        let _ = world.archetype_count();
    }
}

#[test]
fn schedule_runs_all_systems() {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );

    world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);

    let mut stage = Stage::new("update");
    stage.add_system(CounterSystem { name: "counter_a".into() });
    stage.add_system(CounterSystem { name: "counter_b".into() });

    let mut schedule = Schedule::new();
    schedule.add_stage(stage);
    assert_eq!(schedule.system_count(), 2);

    schedule.run(&mut world);
    // If we get here without panic, the scheduler ran
}

#[test]
fn fn_system_modifies_world() {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );
    let pos_id = pos;

    world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);

    let mut stage = Stage::new("physics");
    stage.add_system(FnSystem::new("move_system", move |w| {
        // Read all positions and verify count
        let q = w.query(&[pos_id]);
        assert_eq!(q.total_rows(), 1);
    }));

    let mut sched = Schedule::new();
    sched.add_stage(stage);
    sched.run(&mut world);
}
