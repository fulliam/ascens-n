use ecs_core::ComponentMask;

#[test]
fn set_and_contains() {
    let mut mask = ComponentMask::new();
    mask.set(0);
    mask.set(63);
    mask.set(64);
    mask.set(127);
    mask.set(255);

    assert!(mask.contains(0));
    assert!(mask.contains(63));
    assert!(mask.contains(64));
    assert!(mask.contains(127));
    assert!(mask.contains(255));
    assert!(!mask.contains(1));
    assert!(!mask.contains(128));
}

#[test]
fn clear_bit() {
    let mut mask = ComponentMask::new();
    mask.set(5);
    assert!(mask.contains(5));
    mask.clear(5);
    assert!(!mask.contains(5));
}

#[test]
fn matches_superset() {
    let mut archetype = ComponentMask::new();
    archetype.set(0);
    archetype.set(1);
    archetype.set(2);

    let mut query = ComponentMask::new();
    query.set(0);
    query.set(1);

    assert!(archetype.matches(&query));
    assert!(!query.matches(&archetype));
}

#[test]
fn is_disjoint() {
    let mut a = ComponentMask::new();
    a.set(0);
    a.set(2);

    let mut b = ComponentMask::new();
    b.set(1);
    b.set(3);

    assert!(a.is_disjoint(&b));

    let mut c = ComponentMask::new();
    c.set(0);
    assert!(!a.is_disjoint(&c));
}

#[test]
fn count_ones() {
    let mut mask = ComponentMask::new();
    mask.set(0);
    mask.set(10);
    mask.set(100);
    assert_eq!(mask.count_ones(), 3);
}

#[test]
fn equality() {
    let mut a = ComponentMask::new();
    let mut b = ComponentMask::new();
    a.set(5); a.set(10);
    b.set(5); b.set(10);
    assert_eq!(a, b);
    b.set(1);
    assert_ne!(a, b);
}
