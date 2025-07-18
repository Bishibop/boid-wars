use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::flocking::FlockingConfig;
#[cfg(debug_assertions)]
use crate::config::PhysicsConfig;
#[cfg(debug_assertions)]
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

#[cfg(debug_assertions)]
const DEBUG_PANEL_WIDTH: f32 = 350.0;

pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(debug_assertions)]
        {
            app.add_plugins(EguiPlugin::default())
                .add_systems(EguiPrimaryContextPass, debug_ui_system)
                .add_systems(Startup, setup_debug_camera);
        }

        #[cfg(not(debug_assertions))]
        {
            // Debug UI disabled in release build
        }
    }
}

#[cfg(debug_assertions)]
fn setup_debug_camera(mut commands: Commands) {
    let game_config = &*boid_wars_shared::GAME_CONFIG;

    // Simple camera setup
    commands.spawn((
        Camera2d,
        Transform::from_xyz(
            game_config.game_width / 2.0,
            game_config.game_height / 2.0,
            999.0,
        ),
    ));
}

#[cfg(debug_assertions)]
fn debug_ui_system(
    mut contexts: EguiContexts,
    mut flocking_config: ResMut<FlockingConfig>,
    mut physics_config: ResMut<PhysicsConfig>,
    time: Res<Time>,
    boids: Query<&boid_wars_shared::Velocity, With<boid_wars_shared::Boid>>,
    _spatial_grid: Res<crate::spatial_grid::SpatialGrid>,
    mut clipboard_feedback: Local<Option<std::time::Instant>>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    // Show clipboard feedback if active
    let show_clipboard_feedback = clipboard_feedback
        .as_ref()
        .map(|time| time.elapsed().as_secs_f32() < 1.0)
        .unwrap_or(false);
    
    if show_clipboard_feedback {
        egui::Area::new(egui::Id::from("clipboard_feedback"))
            .fixed_pos(egui::pos2(ctx.screen_rect().center().x - 50.0, 20.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(0, 255, 0));
                ui.label("✅ Copied!");
            });
    }

    egui::SidePanel::left("debug_panel")
        .default_width(DEBUG_PANEL_WIDTH)
        .resizable(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Flocking Configuration Section
                ui.heading("🐦 Flocking Configuration");
                
                // Detection Radii
                ui.separator();
                render_config_section(
                    ui,
                    "Detection Radii",
                    &mut *clipboard_feedback,
                    || format!(
                        "separation_radius: {:.1}\nalignment_radius: {:.1}\ncohesion_radius: {:.1}",
                        flocking_config.separation_radius,
                        flocking_config.alignment_radius,
                        flocking_config.cohesion_radius
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut flocking_config.separation_radius, 10.0..=200.0)
                        .text("Separation Radius"),
                )
                .on_hover_text("Distance at which boids avoid each other");

                ui.add(
                    egui::Slider::new(&mut flocking_config.alignment_radius, 20.0..=300.0)
                        .text("Alignment Radius"),
                )
                .on_hover_text("Distance at which boids align their velocities");

                ui.add(
                    egui::Slider::new(&mut flocking_config.cohesion_radius, 30.0..=400.0)
                        .text("Cohesion Radius"),
                )
                .on_hover_text("Distance at which boids move toward each other");

                // Force Weights
                ui.separator();
                render_config_section(
                    ui,
                    "Force Weights",
                    &mut *clipboard_feedback,
                    || format!(
                        "separation_weight: {:.1}\nalignment_weight: {:.1}\ncohesion_weight: {:.1}",
                        flocking_config.separation_weight,
                        flocking_config.alignment_weight,
                        flocking_config.cohesion_weight
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut flocking_config.separation_weight, 0.0..=5.0)
                        .text("Separation Weight"),
                )
                .on_hover_text("How strongly boids avoid each other");

                ui.add(
                    egui::Slider::new(&mut flocking_config.alignment_weight, 0.0..=5.0)
                        .text("Alignment Weight"),
                )
                .on_hover_text("How strongly boids match velocities");

                ui.add(
                    egui::Slider::new(&mut flocking_config.cohesion_weight, 0.0..=5.0)
                        .text("Cohesion Weight"),
                )
                .on_hover_text("How strongly boids group together");

                // Movement Parameters
                ui.separator();
                render_config_section(
                    ui,
                    "Movement Parameters",
                    &mut *clipboard_feedback,
                    || format!(
                        "max_speed: {:.1}\nmax_force: {:.1}",
                        flocking_config.max_speed,
                        flocking_config.max_force
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut flocking_config.max_speed, 50.0..=500.0).text("Max Speed"),
                )
                .on_hover_text("Maximum boid velocity");

                ui.add(
                    egui::Slider::new(&mut flocking_config.max_force, 100.0..=1000.0).text("Max Force"),
                )
                .on_hover_text("Maximum steering force per frame");

                // Boundary Behavior
                ui.separator();
                render_config_section(
                    ui,
                    "Boundary Behavior",
                    &mut *clipboard_feedback,
                    || format!(
                        "boundary_margin: {:.1}\nboundary_turn_force: {:.1}",
                        flocking_config.boundary_margin,
                        flocking_config.boundary_turn_force
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut flocking_config.boundary_margin, 10.0..=200.0)
                        .text("Boundary Margin"),
                )
                .on_hover_text("Distance from edge to start turning");

                ui.add(
                    egui::Slider::new(&mut flocking_config.boundary_turn_force, 0.5..=10.0)
                        .text("Boundary Turn Force"),
                )
                .on_hover_text("How strongly to turn away from boundaries");

                ui.add(
                    egui::Slider::new(&mut flocking_config.wall_avoidance_weight, 0.0..=10.0)
                        .text("Wall Avoidance Weight"),
                )
                .on_hover_text("Strength of wall avoidance (uses predictive steering)");

                // Collision Configuration Section
                ui.separator();
                ui.separator();
                ui.heading("💥 Collision Configuration");

                // Player Collision
                ui.separator();
                render_config_section(
                    ui,
                    "Player Collision",
                    &mut *clipboard_feedback,
                    || format!(
                        "player_collider_size: {:.1}",
                        physics_config.player_collider_size
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut physics_config.player_collider_size, 1.0..=20.0)
                        .text("Player Collider Size"),
                )
                .on_hover_text("Size of player collision box");

                // Boid Collision
                ui.separator();
                render_config_section(
                    ui,
                    "Boid Collision",
                    &mut *clipboard_feedback,
                    || format!(
                        "boid_radius: {:.1}",
                        physics_config.boid_radius
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut physics_config.boid_radius, 1.0..=15.0)
                        .text("Boid Radius"),
                )
                .on_hover_text("Radius of boid collision circle");

                // Projectile Collision
                ui.separator();
                render_config_section(
                    ui,
                    "Projectile Collision",
                    &mut *clipboard_feedback,
                    || format!(
                        "projectile_collider_radius: {:.1}",
                        physics_config.projectile_collider_radius
                    ),
                );

                // Ensure projectile radius doesn't exceed entity sizes
                let max_projectile_radius = physics_config.boid_radius.min(physics_config.player_collider_size / 2.0);
                ui.add(
                    egui::Slider::new(&mut physics_config.projectile_collider_radius, 0.5..=max_projectile_radius)
                        .text("Projectile Radius"),
                )
                .on_hover_text(format!("Radius of projectile collision circle (max: {:.1})", max_projectile_radius));

                // Wall Collision
                ui.separator();
                render_config_section(
                    ui,
                    "Wall Collision",
                    &mut *clipboard_feedback,
                    || format!(
                        "arena_wall_thickness: {:.1}",
                        physics_config.arena_wall_thickness
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut physics_config.arena_wall_thickness, 5.0..=100.0)
                        .text("Wall Thickness"),
                )
                .on_hover_text("Thickness of arena walls");

                // Copy All Configurations
                ui.separator();
                ui.separator();
                if ui.button("📋 Copy All Configurations").clicked() {
                    let all_config = format!(
                        "// Flocking Configuration\n\
                        separation_radius: {:.1}\n\
                        alignment_radius: {:.1}\n\
                        cohesion_radius: {:.1}\n\
                        separation_weight: {:.1}\n\
                        alignment_weight: {:.1}\n\
                        cohesion_weight: {:.1}\n\
                        max_speed: {:.1}\n\
                        max_force: {:.1}\n\
                        boundary_margin: {:.1}\n\
                        boundary_turn_force: {:.1}\n\
                        wall_avoidance_weight: {:.1}\n\n\
                        // Obstacle Avoidance\n\
                        obstacle_avoidance_radius: {:.1}\n\
                        obstacle_avoidance_weight: {:.1}\n\
                        obstacle_prediction_time: {:.1}\n\
                        player_avoidance_radius: {:.1}\n\
                        player_avoidance_weight: {:.1}\n\n\
                        // Collision Configuration\n\
                        player_collider_size: {:.1}\n\
                        boid_radius: {:.1}\n\
                        projectile_collider_radius: {:.1}\n\
                        arena_wall_thickness: {:.1}",
                        flocking_config.separation_radius,
                        flocking_config.alignment_radius,
                        flocking_config.cohesion_radius,
                        flocking_config.separation_weight,
                        flocking_config.alignment_weight,
                        flocking_config.cohesion_weight,
                        flocking_config.max_speed,
                        flocking_config.max_force,
                        flocking_config.boundary_margin,
                        flocking_config.boundary_turn_force,
                        flocking_config.wall_avoidance_weight,
                        flocking_config.obstacle_avoidance_radius,
                        flocking_config.obstacle_avoidance_weight,
                        flocking_config.obstacle_prediction_time,
                        flocking_config.player_avoidance_radius,
                        flocking_config.player_avoidance_weight,
                        physics_config.player_collider_size,
                        physics_config.boid_radius,
                        physics_config.projectile_collider_radius,
                        physics_config.arena_wall_thickness
                    );
                    ui.ctx().copy_text(all_config);
                    *clipboard_feedback = Some(std::time::Instant::now());
                }

                // Stats
                ui.separator();
                ui.heading("📊 Statistics");

                let boid_count = boids.iter().count();
                ui.label(format!("Boid Count: {}", boid_count));
                ui.label(format!("FPS: {:.1}", 1.0 / time.delta_secs()));

                // Calculate average speed
                if boid_count > 0 {
                    let avg_speed: f32 =
                        boids.iter().map(|vel| vel.0.length()).sum::<f32>() / boid_count as f32;
                    ui.label(format!("Avg Speed: {:.1}", avg_speed));
                }

                // Show spatial grid is active
                ui.label("Spatial Grid: Active");

                // Reset buttons
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Reset Flocking").clicked() {
                        *flocking_config = FlockingConfig::default();
                    }
                    if ui.button("Reset Physics").clicked() {
                        *physics_config = PhysicsConfig::default();
                    }
                });

                // Preset configurations
                ui.separator();
                ui.label("Flocking Presets:");

                if ui.button("Tight Flocking").clicked() {
                    flocking_config.separation_radius = 30.0;
                    flocking_config.alignment_radius = 100.0;
                    flocking_config.cohesion_radius = 120.0;
                    flocking_config.separation_weight = 1.0;
                    flocking_config.alignment_weight = 1.5;
                    flocking_config.cohesion_weight = 1.2;
                }

                if ui.button("Loose Swarm").clicked() {
                    flocking_config.separation_radius = 80.0;
                    flocking_config.alignment_radius = 150.0;
                    flocking_config.cohesion_radius = 200.0;
                    flocking_config.separation_weight = 2.0;
                    flocking_config.alignment_weight = 0.8;
                    flocking_config.cohesion_weight = 0.5;
                }

                if ui.button("Fish School").clicked() {
                    flocking_config.separation_radius = 40.0;
                    flocking_config.alignment_radius = 80.0;
                    flocking_config.cohesion_radius = 100.0;
                    flocking_config.separation_weight = 1.2;
                    flocking_config.alignment_weight = 2.0;
                    flocking_config.cohesion_weight = 1.0;
                    flocking_config.max_speed = 150.0;
                }

                // Obstacle Avoidance Configuration
                ui.separator();
                ui.separator();
                ui.heading("🚧 Obstacle Avoidance");

                // Obstacle Avoidance Parameters
                ui.separator();
                render_config_section(
                    ui,
                    "Obstacle Avoidance",
                    &mut *clipboard_feedback,
                    || format!(
                        "obstacle_avoidance_radius: {:.1}\nobstacle_avoidance_weight: {:.1}\nobstacle_prediction_time: {:.1}",
                        flocking_config.obstacle_avoidance_radius,
                        flocking_config.obstacle_avoidance_weight,
                        flocking_config.obstacle_prediction_time
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut flocking_config.obstacle_avoidance_radius, 20.0..=200.0)
                        .text("Obstacle Detection Radius"),
                )
                .on_hover_text("Distance at which boids detect obstacles");

                ui.add(
                    egui::Slider::new(&mut flocking_config.obstacle_avoidance_weight, 0.0..=10.0)
                        .text("Obstacle Avoidance Weight"),
                )
                .on_hover_text("How strongly boids avoid obstacles");

                ui.add(
                    egui::Slider::new(&mut flocking_config.obstacle_prediction_time, 0.1..=2.0)
                        .text("Obstacle Prediction Time"),
                )
                .on_hover_text("How far ahead boids predict collisions (seconds)");

                // Player Avoidance Parameters
                ui.separator();
                render_config_section(
                    ui,
                    "Player Avoidance",
                    &mut *clipboard_feedback,
                    || format!(
                        "player_avoidance_radius: {:.1}\nplayer_avoidance_weight: {:.1}",
                        flocking_config.player_avoidance_radius,
                        flocking_config.player_avoidance_weight
                    ),
                );

                ui.add(
                    egui::Slider::new(&mut flocking_config.player_avoidance_radius, 20.0..=300.0)
                        .text("Player Detection Radius"),
                )
                .on_hover_text("Distance at which boids detect players");

                ui.add(
                    egui::Slider::new(&mut flocking_config.player_avoidance_weight, 0.0..=10.0)
                        .text("Player Avoidance Weight"),
                )
                .on_hover_text("How strongly boids avoid players");

                // Avoidance Presets
                ui.separator();
                ui.label("Avoidance Presets:");

                if ui.button("Timid Boids").clicked() {
                    flocking_config.obstacle_avoidance_radius = 120.0;
                    flocking_config.obstacle_avoidance_weight = 5.0;
                    flocking_config.player_avoidance_radius = 200.0;
                    flocking_config.player_avoidance_weight = 4.0;
                }

                if ui.button("Balanced Avoidance").clicked() {
                    flocking_config.obstacle_avoidance_radius = 80.0;
                    flocking_config.obstacle_avoidance_weight = 3.0;
                    flocking_config.player_avoidance_radius = 100.0;
                    flocking_config.player_avoidance_weight = 2.5;
                }

                if ui.button("Aggressive Boids").clicked() {
                    flocking_config.obstacle_avoidance_radius = 40.0;
                    flocking_config.obstacle_avoidance_weight = 1.5;
                    flocking_config.player_avoidance_radius = 50.0;
                    flocking_config.player_avoidance_weight = 1.0;
                }
            });
        });

    // Fill remaining space
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |_ui| {});
}

#[cfg(debug_assertions)]
fn render_config_section(
    ui: &mut egui::Ui,
    label: &str,
    clipboard_feedback: &mut Option<std::time::Instant>,
    config_string: impl FnOnce() -> String,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.small_button("📋").clicked() {
            ui.ctx().copy_text(config_string());
            *clipboard_feedback = Some(std::time::Instant::now());
        }
    });
}
