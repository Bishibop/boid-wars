use bevy::prelude::*;
use std::collections::VecDeque;
use tracing::warn;

/// Generation-based entity handle to prevent use-after-free
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PooledEntity {
    pub entity: Entity,
    pub generation: u32,
}

/// Bounded object pool with generation tracking
#[derive(Resource)]
pub struct BoundedPool<T: Component + Clone> {
    /// Available entities ready for reuse
    available: VecDeque<PooledEntity>,
    /// Currently active entities with their generation
    active: std::collections::HashMap<Entity, u32>,
    /// Generation counter for each entity
    generations: std::collections::HashMap<Entity, u32>,
    /// Maximum pool size
    max_size: usize,
    /// Current pool size
    current_size: usize,
    /// Template component for spawning new entities
    template: T,
    /// Pool statistics
    stats: PoolStats,
}

#[derive(Debug, Default)]
pub struct PoolStats {
    pub total_spawned: u64,
    pub total_returned: u64,
    pub failed_returns: u64,
    pub pool_exhausted_count: u64,
    pub max_active: usize,
}

impl<T: Component + Clone> BoundedPool<T> {
    pub fn new(template: T, max_size: usize) -> Self {
        Self {
            available: VecDeque::with_capacity(max_size),
            active: std::collections::HashMap::with_capacity(max_size),
            generations: std::collections::HashMap::with_capacity(max_size),
            max_size,
            current_size: 0,
            template,
            stats: PoolStats::default(),
        }
    }

    /// Get an entity from the pool
    pub fn acquire(&mut self) -> Option<PooledEntity> {
        if let Some(mut pooled) = self.available.pop_front() {
            // Increment generation to invalidate old references
            // This should always exist as we maintain the generation map
            if let Some(gen) = self.generations.get_mut(&pooled.entity) {
                *gen += 1;
                pooled.generation = *gen;
            } else {
                return None;
            }

            self.active.insert(pooled.entity, pooled.generation);
            self.stats.total_spawned += 1;
            self.stats.max_active = self.stats.max_active.max(self.active.len());

            Some(pooled)
        } else {
            self.stats.pool_exhausted_count += 1;
            None
        }
    }

    /// Return an entity to the pool
    pub fn release(&mut self, pooled: PooledEntity) -> bool {
        // Validate generation to prevent double-release
        if let Some(&active_gen) = self.active.get(&pooled.entity) {
            if active_gen == pooled.generation {
                self.active.remove(&pooled.entity);
                self.available.push_back(pooled);
                self.stats.total_returned += 1;
                true
            } else {
                // Generation mismatch - entity was already released
                self.stats.failed_returns += 1;
                false
            }
        } else {
            // Entity not in active set
            self.stats.failed_returns += 1;
            false
        }
    }

    /// Pre-spawn entities into the pool
    pub fn pre_spawn<F>(&mut self, commands: &mut Commands, count: usize, spawn_fn: F)
    where
        F: Fn(&mut Commands, &T) -> Entity,
    {
        let spawn_count = (count.min(self.max_size - self.current_size)).min(100); // Cap at 100 per frame

        for _ in 0..spawn_count {
            let entity = spawn_fn(commands, &self.template);
            let pooled = PooledEntity {
                entity,
                generation: 0,
            };

            self.generations.insert(entity, 0);
            self.available.push_back(pooled);
            self.current_size += 1;
        }
    }

    /// Check if an entity handle is still valid
    pub fn is_valid(&self, pooled: PooledEntity) -> bool {
        self.active
            .get(&pooled.entity)
            .map(|&gen| gen == pooled.generation)
            .unwrap_or(false)
    }

    /// Get pool statistics
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Get current pool status
    pub fn status(&self) -> PoolStatus {
        PoolStatus {
            available: self.available.len(),
            active: self.active.len(),
            total: self.current_size,
            max_size: self.max_size,
        }
    }
}

#[derive(Debug)]
pub struct PoolStatus {
    pub available: usize,
    pub active: usize,
    pub total: usize,
    pub max_size: usize,
}

/// System to monitor pool health and log warnings
pub fn monitor_pool_health<T: Component + Clone>(
    pool: Res<BoundedPool<T>>,
    mut last_warning: Local<f32>,
    time: Res<Time>,
) {
    let status = pool.status();
    let utilization = status.active as f32 / status.max_size as f32;

    // Warn if pool is getting full
    if utilization > 0.8 && time.elapsed_secs() - *last_warning > 5.0 {
        *last_warning = time.elapsed_secs();
    }

    // Log stats periodically
    if time.elapsed_secs() as u32 % 30 == 0 {
        let stats = pool.stats();
        debug!(
            "Pool stats - Spawned: {}, Returned: {}, Failed: {}, Exhausted: {}",
            stats.total_spawned,
            stats.total_returned,
            stats.failed_returns,
            stats.pool_exhausted_count
        );
    }
}
