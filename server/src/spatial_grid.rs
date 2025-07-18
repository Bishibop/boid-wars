use bevy::prelude::*;
use std::collections::HashMap;

/// Spatial grid for efficient neighbor queries
#[derive(Resource)]
pub struct SpatialGrid {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<Entity>>,
    // Pre-allocated buffer for query results
    query_buffer: Vec<Entity>,
    // Cache for cell calculations
    cells_per_row: i32,
    cells_per_col: i32,
    #[allow(dead_code)]
    width: f32,
    #[allow(dead_code)]
    height: f32,
}

impl SpatialGrid {
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let cells_per_row = (width / cell_size).ceil() as i32;
        let cells_per_col = (height / cell_size).ceil() as i32;
        
        // Pre-size the HashMap based on expected density
        let expected_cells = ((cells_per_row * cells_per_col) as f32 * 0.3) as usize;
        let mut grid = HashMap::with_capacity(expected_cells);
        
        Self {
            cell_size,
            grid,
            query_buffer: Vec::with_capacity(256),
            cells_per_row,
            cells_per_col,
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        // Clear but retain capacity to avoid reallocation
        for (_, entities) in self.grid.iter_mut() {
            entities.clear();
        }
        // Remove empty cells to prevent unbounded growth
        self.grid.retain(|_, entities| !entities.is_empty());
    }

    pub fn insert(&mut self, entity: Entity, position: Vec2) {
        let cell = self.get_cell(position);
        
        // Pre-allocate with reasonable capacity for new cells
        self.grid.entry(cell)
            .or_insert_with(|| Vec::with_capacity(8))
            .push(entity);
    }

    pub fn get_nearby_entities(&self, position: Vec2, radius: f32) -> Vec<Entity> {
        self.get_nearby_entities_filtered(position, radius, None)
    }

    /// Get nearby entities with optional distance filtering and entity type filter
    pub fn get_nearby_entities_filtered(
        &self,
        position: Vec2,
        radius: f32,
        filter: Option<fn(Entity) -> bool>,
    ) -> Vec<Entity> {
        // Use thread-local storage for the result to avoid allocations
        thread_local! {
            static RESULT_BUFFER: std::cell::RefCell<Vec<Entity>> = 
                std::cell::RefCell::new(Vec::with_capacity(256));
        }
        
        RESULT_BUFFER.with(|buffer| {
            let mut result = buffer.borrow_mut();
            result.clear();
            
            let radius_squared = radius * radius;
            let cell_radius = (radius / self.cell_size).ceil() as i32;
            let center_cell = self.get_cell(position);
            
            // Early bounds checking to avoid unnecessary iterations
            let min_x = (center_cell.0 - cell_radius).max(0);
            let max_x = (center_cell.0 + cell_radius).min(self.cells_per_row - 1);
            let min_y = (center_cell.1 - cell_radius).max(0);
            let max_y = (center_cell.1 + cell_radius).min(self.cells_per_col - 1);
            
            // Check all cells within radius
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    // Skip cells that are definitely outside the circle
                    let cell_center_x = (x as f32 + 0.5) * self.cell_size;
                    let cell_center_y = (y as f32 + 0.5) * self.cell_size;
                    let cell_dist_sq = (cell_center_x - position.x).powi(2) + 
                                       (cell_center_y - position.y).powi(2);
                    
                    // Conservative check - if cell center is too far, skip
                    if cell_dist_sq > (radius + self.cell_size * 0.7071).powi(2) {
                        continue;
                    }
                    
                    if let Some(entities) = self.grid.get(&(x, y)) {
                        for &entity in entities {
                            if let Some(filter_fn) = filter {
                                if !filter_fn(entity) {
                                    continue;
                                }
                            }
                            result.push(entity);
                        }
                    }
                }
            }
            
            // Return a clone to avoid holding the RefCell borrow
            result.clone()
        })
    }

    /// Get nearby entities with actual distance checking
    pub fn get_nearby_entities_exact(
        &self,
        position: Vec2,
        radius: f32,
        positions: &Query<&boid_wars_shared::Position>,
    ) -> Vec<Entity> {
        let mut result = Vec::with_capacity(64);
        let radius_squared = radius * radius;
        
        // Get candidates from cells
        let candidates = self.get_nearby_entities(position, radius);
        
        // Filter by actual distance
        for entity in candidates {
            if let Ok(entity_pos) = positions.get(entity) {
                let dist_squared = position.distance_squared(entity_pos.0);
                if dist_squared <= radius_squared {
                    result.push(entity);
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