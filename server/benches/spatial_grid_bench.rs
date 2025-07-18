use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bevy::prelude::*;
// Import from the server crate
extern crate boid_wars_server;
use boid_wars_server::spatial_grid::SpatialGrid;

fn setup_grid_with_entities(count: usize) -> (SpatialGrid, Vec<(Entity, Vec2)>) {
    let mut grid = SpatialGrid::new(2000.0, 1500.0, 100.0);
    let mut entities = Vec::with_capacity(count);
    
    // Distribute entities randomly across the space
    for i in 0..count {
        let entity = Entity::from_raw(i as u32);
        let x = (i as f32 * 17.0) % 2000.0;
        let y = (i as f32 * 23.0) % 1500.0;
        let pos = Vec2::new(x, y);
        
        grid.insert(entity, pos);
        entities.push((entity, pos));
    }
    
    (grid, entities)
}

fn bench_spatial_grid_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_grid");
    
    // Test with different entity counts
    for count in [1000, 5000, 10000].iter() {
        let (grid, entities) = setup_grid_with_entities(*count);
        
        group.bench_function(format!("query_{}_entities", count), |b| {
            let mut i = 0;
            b.iter(|| {
                // Query from different positions
                let pos = entities[i % entities.len()].1;
                let result = black_box(grid.get_nearby_entities(pos, 150.0));
                i += 1;
                result
            });
        });
    }
    
    group.finish();
}

fn bench_spatial_grid_insert(c: &mut Criterion) {
    c.bench_function("spatial_grid_insert_10k", |b| {
        let mut grid = SpatialGrid::new(2000.0, 1500.0, 100.0);
        let mut i = 0u32;
        
        b.iter(|| {
            let entity = Entity::from_raw(i);
            let pos = Vec2::new((i as f32 * 17.0) % 2000.0, (i as f32 * 23.0) % 1500.0);
            black_box(grid.insert(entity, pos));
            i = i.wrapping_add(1);
            
            // Clear periodically to prevent unbounded growth
            if i % 10000 == 0 {
                grid.clear();
            }
        });
    });
}

fn bench_spatial_grid_clear(c: &mut Criterion) {
    c.bench_function("spatial_grid_clear_10k", |b| {
        let (mut grid, _) = setup_grid_with_entities(10000);
        
        b.iter(|| {
            black_box(grid.clear());
            // Re-populate for next iteration
            for i in 0..10000 {
                let entity = Entity::from_raw(i as u32);
                let pos = Vec2::new((i as f32 * 17.0) % 2000.0, (i as f32 * 23.0) % 1500.0);
                grid.insert(entity, pos);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_spatial_grid_queries,
    bench_spatial_grid_insert,
    bench_spatial_grid_clear
);
criterion_main!(benches);