use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use boid_wars_server::physics::{PhysicsPlugin, ProjectilePool};
use boid_wars_server::position_sync::PositionSyncPlugin;
use boid_wars_shared::{Boid, Player, Position, Velocity};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn spawn_entities_benchmark(c: &mut Criterion) {
    c.bench_function("spawn_1000_entities", |b| {
        b.iter(|| {
            let mut app = create_test_app();

            // Spawn entities
            for i in 0..1000 {
                app.world_mut().spawn((
                    Position(Vec2::new(i as f32, i as f32)),
                    Velocity(Vec2::new(1.0, 0.0)),
                    Transform::from_xyz(i as f32, i as f32, 0.0),
                    RigidBody::Dynamic,
                    Collider::ball(4.0),
                ));
            }

            black_box(&app);
        });
    });

    c.bench_function("spawn_10000_entities", |b| {
        b.iter(|| {
            let mut app = create_test_app();

            // Spawn entities
            for i in 0..10000 {
                app.world_mut().spawn((
                    Position(Vec2::new(i as f32 % 800.0, i as f32 % 600.0)),
                    Velocity(Vec2::new(1.0, 0.0)),
                    Transform::from_xyz(i as f32 % 800.0, i as f32 % 600.0, 0.0),
                    RigidBody::Dynamic,
                    Collider::ball(4.0),
                ));
            }

            black_box(&app);
        });
    });
}

fn physics_update_benchmark(c: &mut Criterion) {
    c.bench_function("update_1000_physics_entities", |b| {
        let mut app = create_test_app();

        // Pre-spawn entities
        for i in 0..1000 {
            app.world_mut().spawn((
                Position(Vec2::new(i as f32 % 800.0, i as f32 % 600.0)),
                Velocity(Vec2::new(1.0, 0.0)),
                Transform::from_xyz(i as f32 % 800.0, i as f32 % 600.0, 0.0),
                RigidBody::Dynamic,
                Collider::ball(4.0),
                bevy_rapier2d::dynamics::Velocity::linear(Vec2::new(1.0, 0.0)),
            ));
        }

        b.iter(|| {
            app.update();
            black_box(&app);
        });
    });

    c.bench_function("update_10000_physics_entities", |b| {
        let mut app = create_test_app();

        // Pre-spawn entities
        for i in 0..10000 {
            app.world_mut().spawn((
                Position(Vec2::new(i as f32 % 800.0, i as f32 % 600.0)),
                Velocity(Vec2::new(1.0, 0.0)),
                Transform::from_xyz(i as f32 % 800.0, i as f32 % 600.0, 0.0),
                RigidBody::Dynamic,
                Collider::ball(4.0),
                bevy_rapier2d::dynamics::Velocity::linear(Vec2::new(1.0, 0.0)),
            ));
        }

        b.iter(|| {
            app.update();
            black_box(&app);
        });
    });
}

fn position_sync_benchmark(c: &mut Criterion) {
    c.bench_function("sync_1000_positions", |b| {
        let mut app = create_test_app();

        // Pre-spawn entities with both Transform and Position
        for i in 0..1000 {
            app.world_mut().spawn((
                Position(Vec2::new(i as f32 % 800.0, i as f32 % 600.0)),
                Transform::from_xyz(i as f32 % 800.0 + 0.1, i as f32 % 600.0 + 0.1, 0.0),
                boid_wars_server::position_sync::SyncPosition,
            ));
        }

        b.iter(|| {
            app.update();
            black_box(&app);
        });
    });
}

fn projectile_pool_benchmark(c: &mut Criterion) {
    c.bench_function("projectile_pool_spawn_despawn", |b| {
        let mut app = create_test_app();
        let pool_resource = app.world().resource::<ProjectilePool>().clone();

        b.iter(|| {
            // Simulate spawning and despawning 100 projectiles
            let mut spawned = Vec::new();

            // Spawn from pool
            for _ in 0..100 {
                if let Some(entity) = pool_resource.available.last() {
                    spawned.push(entity);
                }
            }

            // Return to pool
            for entity in spawned {
                // Simulate returning to pool
                black_box(entity);
            }
        });
    });
}

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(TransformPlugin);
    app.add_plugins(PhysicsPlugin);
    app.add_plugins(PositionSyncPlugin);
    app.init_resource::<ProjectilePool>();
    app
}

criterion_group!(
    benches,
    spawn_entities_benchmark,
    physics_update_benchmark,
    position_sync_benchmark,
    projectile_pool_benchmark
);
criterion_main!(benches);
