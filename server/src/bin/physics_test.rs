use bevy::prelude::*;
use boid_wars_server::physics::*;

fn main() {
    println!("Starting physics test...");
    
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin)
        .add_systems(Startup, setup_test)
        .add_systems(Update, debug_system)
        .run();
}

fn setup_test(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
    
    // Spawn player
    commands.spawn((
        Player::default(),
        PlayerInput::default(),
        Ship::default(),
        WeaponStats::default(),
        Velocity::zero(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        RigidBody::Dynamic,
        Collider::cuboid(24.0, 32.0),
        GameCollisionGroups::player(),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    // Spawn AI players
    for i in 0..4 {
        let ai_type = match i {
            0 => AIType::Circler,
            1 => AIType::Bouncer,
            2 => AIType::Shooter,
            _ => AIType::Chaser,
        };
        
        let x = (i as f32 - 1.5) * 200.0;
        
        commands.spawn((
            Player::default(),
            AIPlayer::new(ai_type),
            PlayerInput::default(),
            Ship::default(),
            WeaponStats::default(),
            Velocity::zero(),
            ExternalForce::default(),
            ExternalImpulse::default(),
            RigidBody::Dynamic,
            Collider::cuboid(24.0, 32.0),
            GameCollisionGroups::player(),
            Transform::from_xyz(x, 200.0, 0.0),
        ));
    }
}

fn debug_system(
    time: Res<Time>,
    players: Query<(&Transform, Option<&AIPlayer>), With<Player>>,
) {
    if time.elapsed_secs() % 2.0 < 0.016 {
        println!("\n--- Physics Debug ---");
        for (transform, ai) in players.iter() {
            let ai_type = ai.map(|a| format!("{:?}", a.ai_type)).unwrap_or("Player".to_string());
            println!("{}: pos({:.1}, {:.1})", ai_type, transform.translation.x, transform.translation.y);
        }
    }
}