use bevy::prelude::*;

fn main() {
    println!("🚀 Boid Wars Server Starting...");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, log_status)
        .run();
}

fn setup(mut commands: Commands) {
    println!("✅ Server initialized");

    // Create a timer to log status every 5 seconds
    commands.spawn(StatusTimer {
        timer: Timer::from_seconds(5.0, TimerMode::Repeating),
    });
}

fn log_status(time: Res<Time>, mut query: Query<&mut StatusTimer>) {
    for mut status in query.iter_mut() {
        if status.timer.tick(time.delta()).just_finished() {
            println!("📊 Server running - Uptime: {:.1}s", time.elapsed_seconds());
        }
    }
}

#[derive(Component)]
struct StatusTimer {
    timer: Timer,
}
