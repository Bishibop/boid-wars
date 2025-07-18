use bevy::prelude::*;
use std::collections::HashMap;

/// Spatial grid for efficient neighbor queries
#[derive(Resource)]
pub struct SpatialGrid {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<Entity>>,
    #[allow(dead_code)]
    width: f32,
    #[allow(dead_code)]
    height: f32,
}

impl SpatialGrid {
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn insert(&mut self, entity: Entity, position: Vec2) {
        let cell = self.get_cell(position);
        self.grid.entry(cell).or_insert_with(Vec::new).push(entity);
    }

    pub fn get_nearby_entities(&self, position: Vec2, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = self.get_cell(position);

        // Check all cells within radius
        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);
                if let Some(entities) = self.grid.get(&cell) {
                    result.extend(entities.iter().copied());
                }
            }
        }

        result
    }

    fn get_cell(&self, position: Vec2) -> (i32, i32) {
        let x = (position.x / self.cell_size).floor() as i32;
        let y = (position.y / self.cell_size).floor() as i32;
        (x, y)
    }
}

impl Default for SpatialGrid {
    fn default() -> Self {
        let game_config = &*boid_wars_shared::GAME_CONFIG;
        Self::new(game_config.game_width, game_config.game_height, 100.0)
    }
}

/// Update the spatial grid with all entity positions
pub fn update_spatial_grid(
    mut spatial_grid: ResMut<SpatialGrid>,
    entities: Query<(Entity, &boid_wars_shared::Position)>,
) {
    spatial_grid.clear();
    
    for (entity, pos) in entities.iter() {
        spatial_grid.insert(entity, pos.0);
    }
}