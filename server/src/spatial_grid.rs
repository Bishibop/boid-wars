use bevy::prelude::*;

/// System sets for spatial grid operations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpatialGridSet {
    /// Systems that write to the spatial grid (update_spatial_grid)
    Update,
    /// Systems that read from the spatial grid (flocking, targeting, etc.)
    Read,
}

/// Spatial grid for efficient neighbor queries
#[derive(Resource)]
pub struct SpatialGrid {
    cell_size: f32,
    // Flat array indexed by row * cells_per_row + col for better cache locality
    cells: Vec<Vec<Entity>>,
    // Dimensions for index calculation
    cells_per_row: usize,
    cells_per_col: usize,
    #[allow(dead_code)]
    width: f32,
    #[allow(dead_code)]
    height: f32,
}

impl SpatialGrid {
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let cells_per_row = (width / cell_size).ceil() as usize;
        let cells_per_col = (height / cell_size).ceil() as usize;
        let total_cells = cells_per_row * cells_per_col;
        
        // Pre-allocate all cells with reasonable capacity
        let mut cells = Vec::with_capacity(total_cells);
        for _ in 0..total_cells {
            cells.push(Vec::with_capacity(8)); // Most cells will have <8 entities
        }
        
        Self {
            cell_size,
            cells,
            cells_per_row,
            cells_per_col,
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        // Clear all cells but retain capacity to avoid reallocation
        for cell in self.cells.iter_mut() {
            cell.clear();
        }
    }

    pub fn insert(&mut self, entity: Entity, position: Vec2) {
        if let Some(idx) = self.get_cell_index(position) {
            self.cells[idx].push(entity);
        }
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
            
            let _radius_squared = radius * radius;
            let cell_radius = (radius / self.cell_size).ceil() as i32;
            let center_cell = self.get_cell(position);
            
            // Early bounds checking to avoid unnecessary iterations
            let min_x = (center_cell.0 - cell_radius).max(0);
            let max_x = (center_cell.0 + cell_radius).min(self.cells_per_row as i32 - 1);
            let min_y = (center_cell.1 - cell_radius).max(0);
            let max_y = (center_cell.1 + cell_radius).min(self.cells_per_col as i32 - 1);
            
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
                    
                    // Use flat array index for better cache locality
                    if let Some(idx) = self.get_cell_index_from_coords(x, y) {
                        for &entity in &self.cells[idx] {
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
    
    /// Get flat array index from position, returns None if out of bounds
    fn get_cell_index(&self, position: Vec2) -> Option<usize> {
        let (x, y) = self.get_cell(position);
        if x >= 0 && y >= 0 && x < self.cells_per_row as i32 && y < self.cells_per_col as i32 {
            Some(y as usize * self.cells_per_row + x as usize)
        } else {
            None
        }
    }
    
    /// Get flat array index from cell coordinates, returns None if out of bounds
    fn get_cell_index_from_coords(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && y >= 0 && x < self.cells_per_row as i32 && y < self.cells_per_col as i32 {
            Some(y as usize * self.cells_per_row + x as usize)
        } else {
            None
        }
    }
    
    /// Get statistics about grid usage (useful for debugging)
    #[allow(dead_code)]
    pub fn get_stats(&self) -> SpatialGridStats {
        let total_cells = self.cells.len();
        let occupied_cells = self.cells.iter()
            .filter(|cell| !cell.is_empty())
            .count();
        let total_entities = self.cells.iter()
            .map(|cell| cell.len())
            .sum();
        let max_entities_per_cell = self.cells.iter()
            .map(|cell| cell.len())
            .max()
            .unwrap_or(0);
            
        SpatialGridStats {
            total_cells,
            occupied_cells,
            occupancy_ratio: occupied_cells as f32 / total_cells as f32,
            total_entities,
            avg_entities_per_occupied_cell: if occupied_cells > 0 {
                total_entities as f32 / occupied_cells as f32
            } else {
                0.0
            },
            max_entities_per_cell,
        }
    }
}

/// Statistics about spatial grid usage
#[derive(Debug, Clone)]
pub struct SpatialGridStats {
    pub total_cells: usize,
    pub occupied_cells: usize,
    pub occupancy_ratio: f32,
    pub total_entities: usize,
    pub avg_entities_per_occupied_cell: f32,
    pub max_entities_per_cell: usize,
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

/// Plugin for spatial grid functionality
pub struct SpatialGridPlugin;

impl Plugin for SpatialGridPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize the spatial grid resource
            .init_resource::<SpatialGrid>()
            // Configure system sets for clear dependencies
            .configure_sets(
                FixedUpdate,
                (
                    SpatialGridSet::Update,
                    SpatialGridSet::Read.after(SpatialGridSet::Update),
                ),
            )
            // Add the update system
            .add_systems(
                FixedUpdate,
                update_spatial_grid.in_set(SpatialGridSet::Update),
            );
        
        info!("Spatial grid plugin initialized");
    }
}