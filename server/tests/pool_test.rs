use bevy::prelude::*;
use boid_wars_server::pool::{BoundedPool, PooledEntity};

#[derive(Component, Clone)]
struct TestComponent {
    value: i32,
}

#[test]
fn test_bounded_pool_basic_operations() {
    let mut app = App::new();
    let mut pool = BoundedPool::new(TestComponent { value: 0 }, 10);

    // Pre-spawn some entities
    pool.pre_spawn(&mut app.world_mut().commands(), 5, |cmds, template| {
        cmds.spawn(template.clone()).id()
    });

    let status = pool.status();
    assert_eq!(status.available, 5);
    assert_eq!(status.active, 0);
    assert_eq!(status.total, 5);

    // Acquire an entity
    let handle1 = pool.acquire().expect("Should get entity from pool");
    assert!(pool.is_valid(handle1));

    let status = pool.status();
    assert_eq!(status.available, 4);
    assert_eq!(status.active, 1);

    // Release it back
    assert!(pool.release(handle1));

    let status = pool.status();
    assert_eq!(status.available, 5);
    assert_eq!(status.active, 0);
}

#[test]
fn test_pool_generation_tracking() {
    let mut app = App::new();
    let mut pool = BoundedPool::new(TestComponent { value: 0 }, 10);

    pool.pre_spawn(&mut app.world_mut().commands(), 1, |cmds, template| {
        cmds.spawn(template.clone()).id()
    });

    // Get entity and store handle
    let handle1 = pool.acquire().unwrap();
    let entity = handle1.entity;
    let gen1 = handle1.generation;

    // Release it
    pool.release(handle1);

    // Get it again - should have new generation
    let handle2 = pool.acquire().unwrap();
    assert_eq!(handle2.entity, entity); // Same entity
    assert_eq!(handle2.generation, gen1 + 1); // New generation

    // Old handle should be invalid
    assert!(!pool.is_valid(handle1));
    assert!(pool.is_valid(handle2));
}

#[test]
fn test_pool_double_release_protection() {
    let mut app = App::new();
    let mut pool = BoundedPool::new(TestComponent { value: 0 }, 10);

    pool.pre_spawn(&mut app.world_mut().commands(), 1, |cmds, template| {
        cmds.spawn(template.clone()).id()
    });

    let handle = pool.acquire().unwrap();

    // First release should succeed
    assert!(pool.release(handle));

    // Second release should fail
    assert!(!pool.release(handle));

    // Stats should show failed return
    let stats = pool.stats();
    assert_eq!(stats.failed_returns, 1);
}

#[test]
fn test_pool_exhaustion() {
    let mut app = App::new();
    let mut pool = BoundedPool::new(TestComponent { value: 0 }, 2);

    pool.pre_spawn(&mut app.world_mut().commands(), 2, |cmds, template| {
        cmds.spawn(template.clone()).id()
    });

    // Acquire all entities
    let handle1 = pool.acquire().unwrap();
    let handle2 = pool.acquire().unwrap();

    // Pool should be exhausted
    assert!(pool.acquire().is_none());

    let stats = pool.stats();
    assert_eq!(stats.pool_exhausted_count, 1);

    // Release one and try again
    pool.release(handle1);
    assert!(pool.acquire().is_some());
}

#[test]
fn test_pool_size_limits() {
    let mut app = App::new();
    let mut pool = BoundedPool::new(TestComponent { value: 0 }, 10);

    // Try to spawn more than max size
    pool.pre_spawn(&mut app.world_mut().commands(), 20, |cmds, template| {
        cmds.spawn(template.clone()).id()
    });

    let status = pool.status();
    assert_eq!(status.total, 10); // Should be capped at max_size
    assert_eq!(status.max_size, 10);
}

#[test]
fn test_invalid_entity_handling() {
    let mut app = App::new();
    let mut pool = BoundedPool::new(TestComponent { value: 0 }, 10);

    // Create a fake handle
    let fake_handle = PooledEntity {
        entity: Entity::from_raw(9999),
        generation: 0,
    };

    // Should not be valid
    assert!(!pool.is_valid(fake_handle));

    // Release should fail
    assert!(!pool.release(fake_handle));

    let stats = pool.stats();
    assert_eq!(stats.failed_returns, 1);
}
