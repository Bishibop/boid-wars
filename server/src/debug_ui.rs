use bevy::prelude::*;

#[cfg(debug_assertions)]
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
#[cfg(debug_assertions)]
use crate::flocking::FlockingConfig;

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
    time: Res<Time>,
    boids: Query<&boid_wars_shared::Velocity, With<boid_wars_shared::Boid>>,
    _spatial_grid: Res<crate::spatial_grid::SpatialGrid>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };
    
    egui::SidePanel::left("flocking_debug_panel")
        .default_width(300.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("🐦 Flocking Configuration");
            
            // Detection Radii
            ui.separator();
            ui.label("Detection Radii");
            
            ui.add(egui::Slider::new(&mut flocking_config.separation_radius, 10.0..=200.0)
                .text("Separation Radius"))
                .on_hover_text("Distance at which boids avoid each other");
                
            ui.add(egui::Slider::new(&mut flocking_config.alignment_radius, 20.0..=300.0)
                .text("Alignment Radius"))
                .on_hover_text("Distance at which boids align their velocities");
                
            ui.add(egui::Slider::new(&mut flocking_config.cohesion_radius, 30.0..=400.0)
                .text("Cohesion Radius"))
                .on_hover_text("Distance at which boids move toward each other");
            
            // Force Weights
            ui.separator();
            ui.label("Force Weights");
            
            ui.add(egui::Slider::new(&mut flocking_config.separation_weight, 0.0..=5.0)
                .text("Separation Weight"))
                .on_hover_text("How strongly boids avoid each other");
                
            ui.add(egui::Slider::new(&mut flocking_config.alignment_weight, 0.0..=5.0)
                .text("Alignment Weight"))
                .on_hover_text("How strongly boids match velocities");
                
            ui.add(egui::Slider::new(&mut flocking_config.cohesion_weight, 0.0..=5.0)
                .text("Cohesion Weight"))
                .on_hover_text("How strongly boids group together");
            
            // Movement Parameters
            ui.separator();
            ui.label("Movement Parameters");
            
            ui.add(egui::Slider::new(&mut flocking_config.max_speed, 50.0..=500.0)
                .text("Max Speed"))
                .on_hover_text("Maximum boid velocity");
                
            ui.add(egui::Slider::new(&mut flocking_config.max_force, 100.0..=1000.0)
                .text("Max Force"))
                .on_hover_text("Maximum steering force per frame");
            
            // Boundary Behavior
            ui.separator();
            ui.label("Boundary Behavior");
            
            ui.add(egui::Slider::new(&mut flocking_config.boundary_margin, 10.0..=200.0)
                .text("Boundary Margin"))
                .on_hover_text("Distance from edge to start turning");
                
            ui.add(egui::Slider::new(&mut flocking_config.boundary_turn_force, 0.5..=10.0)
                .text("Boundary Turn Force"))
                .on_hover_text("How strongly to turn away from boundaries");
            
            // Stats
            ui.separator();
            ui.heading("Statistics");
            
            let boid_count = boids.iter().count();
            ui.label(format!("Boid Count: {}", boid_count));
            ui.label(format!("FPS: {:.1}", 1.0 / time.delta_secs()));
            
            // Calculate average speed
            if boid_count > 0 {
                let avg_speed: f32 = boids.iter()
                    .map(|vel| vel.0.length())
                    .sum::<f32>() / boid_count as f32;
                ui.label(format!("Avg Speed: {:.1}", avg_speed));
            }
            
            // Show spatial grid is active
            ui.label("Spatial Grid: Active");
            
            // Reset button
            ui.separator();
            if ui.button("Reset to Defaults").clicked() {
                *flocking_config = FlockingConfig::default();
            }
            
            // Preset configurations
            ui.separator();
            ui.label("Presets:");
            
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
        });
    
    // Fill remaining space
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |_ui| {});
}