use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use boid_wars_server::physics::{Player as PhysicsPlayer, PlayerInput as PhysicsInput};
use boid_wars_server::position_sync::{PositionSyncPlugin, SyncPosition};
use boid_wars_server::spatial_grid::SpatialGridPlugin;
use boid_wars_shared::{Player, Position, Velocity};

#[test]
fn test_physics_to_network_sync() {
    let mut app = create_test_app();

    // Spawn an entity with both physics and network components
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(100.0, 200.0, 0.0),
            Position(Vec2::new(0.0, 0.0)), // Start with different position
            SyncPosition,
            RigidBody::Dynamic,
            bevy_rapier2d::dynamics::Velocity::linear(Vec2::new(10.0, 0.0)),
        ))
        .id();

    // Run one update cycle
    app.update();

    // Check that Position was synced to Transform
    let world = app.world();
    let position = world.get::<Position>(entity).unwrap();
    let transform = world.get::<Transform>(entity).unwrap();

    assert!(
        (position.0 - transform.translation.truncate()).length() < 0.01,
        "Position should be synced to Transform. Position: {:?}, Transform: {:?}",
        position.0,
        transform.translation.truncate()
    );
}

#[test]
fn test_velocity_sync() {
    let mut app = create_test_app();

    // Spawn entity with physics velocity
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(100.0, 100.0, 0.0),
            Position(Vec2::new(100.0, 100.0)),
            Velocity(Vec2::ZERO), // Network velocity starts at zero
            bevy_rapier2d::dynamics::Velocity::linear(Vec2::new(50.0, -30.0)),
            SyncPosition,
        ))
        .id();

    // Run update
    app.update();

    // Check velocity sync
    let world = app.world();
    let net_velocity = world.get::<Velocity>(entity).unwrap();
    let physics_velocity = world
        .get::<bevy_rapier2d::dynamics::Velocity>(entity)
        .unwrap();

    assert!(
        (net_velocity.0 - physics_velocity.linvel).length() < 0.01,
        "Network velocity should match physics velocity. Network: {:?}, Physics: {:?}",
        net_velocity.0,
        physics_velocity.linvel
    );
}

#[test]
fn test_player_movement_physics() {
    let mut app = create_test_app();

    // Spawn a player with physics components
    let player_entity = app
        .world_mut()
        .spawn((
            Player {
                id: 1,
                name: "Test".to_string(),
            },
            PhysicsPlayer {
                player_id: 1,
                ..Default::default()
            },
            PhysicsInput {
                movement: Vec2::new(1.0, 0.0),
                aim_direction: Vec2::new(1.0, 0.0),
                thrust: 1.0,
                shooting: false,
                input_sequence: 0,
            },
            Transform::from_xyz(400.0, 300.0, 0.0),
            Position(Vec2::new(400.0, 300.0)),
            RigidBody::Dynamic,
            Collider::cuboid(5.0, 5.0),
            bevy_rapier2d::dynamics::Velocity::zero(),
            ExternalForce::default(),
            ExternalImpulse::default(),
            bevy_rapier2d::dynamics::GravityScale(0.0),
            bevy_rapier2d::dynamics::AdditionalMassProperties::Mass(1.0),
            bevy_rapier2d::dynamics::Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            SyncPosition,
        ))
        .id();

    // Store initial position
    let initial_pos = app
        .world()
        .get::<Transform>(player_entity)
        .unwrap()
        .translation
        .truncate();

    // Run several physics updates
    for _ in 0..10 {
        app.update();
    }

    // Check that player moved
    let world = app.world();
    let final_transform = world.get::<Transform>(player_entity).unwrap();
    let final_position = world.get::<Position>(player_entity).unwrap();
    let velocity = world
        .get::<bevy_rapier2d::dynamics::Velocity>(player_entity)
        .unwrap();

    // Player should have moved in the positive X direction
    assert!(
        final_transform.translation.x > initial_pos.x,
        "Player should have moved right. Initial: {:.2}, Final: {:.2}",
        initial_pos.x,
        final_transform.translation.x
    );

    // Network position should match physics position
    assert!(
        (final_position.0 - final_transform.translation.truncate()).length() < 0.1,
        "Network position should match physics position"
    );

    // Velocity should be positive X
    assert!(
        velocity.linvel.x > 0.0,
        "Player should have positive X velocity: {:?}",
        velocity.linvel
    );
}

#[test]
fn test_projectile_lifecycle() {
    let mut app = create_test_app();

    // Get the projectile pool
    let pool_status_before = app
        .world()
        .resource::<boid_wars_server::physics::ProjectilePool>()
        .status();

    // Spawn a player who will shoot
    let player_entity = app
        .world_mut()
        .spawn((
            Player {
                id: 1,
                name: "Shooter".to_string(),
            },
            PhysicsPlayer {
                player_id: 1,
                weapon_cooldown: Timer::from_seconds(0.0, TimerMode::Once), // Ready to shoot
                ..Default::default()
            },
            PhysicsInput {
                movement: Vec2::ZERO,
                aim_direction: Vec2::new(1.0, 0.0),
                thrust: 0.0,
                shooting: true, // Player is shooting
                input_sequence: 0,
            },
            Transform::from_xyz(400.0, 300.0, 0.0),
            Position(Vec2::new(400.0, 300.0)),
            boid_wars_server::physics::WeaponStats::default(),
            RigidBody::Dynamic,
            Collider::cuboid(5.0, 5.0),
        ))
        .id();

    // Run update to process shooting
    app.update();

    // Check that a projectile was taken from the pool
    let pool_status_after = app
        .world()
        .resource::<boid_wars_server::physics::ProjectilePool>()
        .status();
    assert!(
        pool_status_after.active > pool_status_before.active,
        "A projectile should have been taken from the pool"
    );

    // Find the projectile
    let world = app.world_mut();
    let projectile_count = world
        .query::<&boid_wars_server::physics::Projectile>()
        .iter(world)
        .filter(|p| p.owner == Some(player_entity))
        .count();

    assert_eq!(
        projectile_count, 1,
        "One projectile should have been spawned"
    );
}

#[test]
fn test_collision_detection() {
    let mut app = create_test_app();

    // Spawn two entities that will collide
    let entity1 = app
        .world_mut()
        .spawn((
            Transform::from_xyz(100.0, 100.0, 0.0),
            RigidBody::Dynamic,
            Collider::ball(10.0),
            bevy_rapier2d::dynamics::Velocity::linear(Vec2::new(100.0, 0.0)),
            ActiveEvents::COLLISION_EVENTS,
            Restitution::coefficient(0.5), // Add some bounce
        ))
        .id();

    let _entity2 = app
        .world_mut()
        .spawn((
            Transform::from_xyz(150.0, 100.0, 0.0),
            RigidBody::Fixed,
            Collider::ball(10.0),
            ActiveEvents::COLLISION_EVENTS,
        ))
        .id();

    // Run several updates to allow collision
    for _ in 0..10 {
        app.update();
    }

    // Check that collision occurred by verifying entity1's velocity changed
    let velocity = app
        .world()
        .get::<bevy_rapier2d::dynamics::Velocity>(entity1)
        .unwrap();

    // After collision with fixed body, the dynamic body should have changed velocity
    // Check that velocity has changed from the initial (100, 0)
    let initial_velocity = Vec2::new(100.0, 0.0);
    assert!(
        (velocity.linvel - initial_velocity).length() > 5.0, // Velocity should have changed noticeably
        "Velocity should change after collision. Initial: {:?}, Final: {:?}",
        initial_velocity,
        velocity.linvel
    );
}

#[test]
fn test_drift_detection() {
    let mut app = create_test_app();

    // Spawn entity with deliberate position drift
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(100.0, 100.0, 0.0),
            Position(Vec2::new(200.0, 200.0)), // Large drift
            SyncPosition,
        ))
        .id();

    // Run update
    app.update();

    // Position should be corrected to match Transform
    let position = app.world().get::<Position>(entity).unwrap();
    let transform = app.world().get::<Transform>(entity).unwrap();

    assert!(
        (position.0 - transform.translation.truncate()).length() < 0.1,
        "Large drift should be corrected"
    );
}

fn create_test_app() -> App {
    let mut app = App::new();

    // Add minimal plugins needed for testing
    app.add_plugins(MinimalPlugins);
    app.add_plugins(TransformPlugin);

    // Add our plugins in the correct order
    app.add_plugins(SpatialGridPlugin); // Must be before PhysicsPlugin
    app.add_plugins(boid_wars_server::physics::PhysicsPlugin {
        enable_debug_render: false, // Disable debug rendering for tests
    });
    app.add_plugins(PositionSyncPlugin);

    // Initialize time resource
    app.init_resource::<Time>();

    app
}
