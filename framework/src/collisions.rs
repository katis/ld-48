use bevy::prelude::*;
use enumflags2::{bitflags, BitFlags};
use flat_spatial::SparseGrid;

use crate::extensions::*;
use bevy::tasks::TaskPool;
use bevy::utils::HashMap;
use flat_spatial::grid::GridHandle;

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CollisionMask {
    Players,
    Enemies,
}

pub struct CollisionLayers {
    list: Vec<CollisionLayer>,
}

impl CollisionLayers {
    pub fn new(list: Vec<CollisionLayer>) -> Self {
        CollisionLayers { list }
    }

    pub fn remove(&mut self, entity: Entity) {
        for layer in self.list.iter_mut() {
            if layer.remove(entity) {
                return;
            }
        }
    }

    pub fn mask_layer(&mut self, mask: CollisionMask) -> Option<&mut CollisionLayer> {
        self.list.iter_mut().find(|l| l.mask == mask)
    }

    pub fn colliding_layers(&self, mask: CollisionMask) -> impl Iterator<Item = &CollisionLayer> {
        self.list
            .iter()
            .filter(move |l| l.collides_with.contains(mask))
    }

    pub fn commit_changes(&mut self) {
        self.list.iter_mut().for_each(|layer| layer.grid.maintain());
    }
}

pub struct CollisionLayer {
    mask: CollisionMask,
    collides_with: BitFlags<CollisionMask>,
    entity_handles: HashMap<Entity, GridHandle>,
    grid: SparseGrid<(Entity, ColliderShape)>,
}

impl CollisionLayer {
    pub fn new(
        mask: CollisionMask,
        collides_with: BitFlags<CollisionMask>,
        cell_size: i32,
    ) -> Self {
        CollisionLayer {
            mask,
            collides_with,
            entity_handles: HashMap::default(),
            grid: SparseGrid::new(cell_size),
        }
    }

    pub fn update(&mut self, handle: GridHandle, pos: Vec2) {
        self.grid.set_position(handle, [pos.x, pos.y]);
    }

    pub fn set_shape(&mut self, handle: GridHandle, shape: ColliderShape) {
        if let Some((_, entry)) = self.grid.get_mut(handle) {
            entry.1 = shape;
        }
    }

    pub fn insert(&mut self, entity: Entity, shape: ColliderShape, pos: Vec2) -> GridHandle {
        let handle = self.grid.insert([pos.x, pos.y], (entity, shape));
        self.entity_handles.insert(entity, handle);
        handle
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        if let Some(handle) = self.entity_handles.remove(&entity) {
            self.grid.remove(handle);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColliderShape {
    pub radius: f32,
}

fn overlaps(a_pos: Vec2, a_shape: ColliderShape, b_pos: Vec2, b_shape: ColliderShape) -> bool {
    a_pos.distance_squared(b_pos) < (a_shape.radius + b_shape.radius).powi(2)
}

pub struct Collider {
    handle: Option<GridHandle>,
    mask: CollisionMask,
    collisions: Vec<Collision>,
}

impl Collider {
    pub fn new(mask: CollisionMask) -> Collider {
        Collider {
            handle: None,
            mask,
            collisions: Vec::new(),
        }
    }
}

pub struct Collision {
    pub target: Entity,
    pub mask: CollisionMask,
    pub target_pos: Vec2,
}

pub fn check_collisions(
    layers: Res<CollisionLayers>,
    pool: Res<TaskPool>,
    mut query: Query<(Entity, &Transform, &ColliderShape, &mut Collider)>,
) {
    query.par_for_each_mut(&pool, 64, |(entity, tx, shape, mut collider)| {
        for layer in layers.colliding_layers(collider.mask) {
            let pos = tx.position();

            for (handle, _) in layer.grid.query_around([pos.x, pos.y], 256.0) {
                let (other_pos, &(other, other_shape)) = layer.grid.get(handle).unwrap();
                if entity != other {
                    let other_pos = Vec2::new(other_pos.x, other_pos.y);

                    if overlaps(pos, *shape, other_pos, other_shape) {
                        collider.collisions.push(Collision {
                            mask: layer.mask,
                            target: other,
                            target_pos: other_pos,
                        });
                    }
                }
            }
        }
    });
}

pub fn update_collision_grids(
    mut layers: ResMut<CollisionLayers>,
    query: Query<(Entity, &Transform, &ColliderShape, &mut Collider)>,
) {
    query.for_each_mut(|(entity, transform, shape, mut collider)| {
        if let Some(layer) = layers.mask_layer(collider.mask) {
            let pos = transform.position();
            if let Some(handle) = collider.handle {
                layer.update(handle, pos);
            } else {
                let handle = layer.insert(entity, *shape, pos);
                collider.handle = Some(handle);
            }
        }
    });
    layers.commit_changes();
}

pub fn reset_collisions(pool: Res<TaskPool>, mut query: Query<&mut Collider>) {
    query.par_for_each_mut(&pool, 128, |mut collider| collider.collisions.clear());
}

pub fn update_collider_shapes(
    mut layers: ResMut<CollisionLayers>,
    query: Query<(&ColliderShape, &mut Collider), Changed<ColliderShape>>,
) {
    query.for_each_mut(|(shape, collider)| {
        if let Some(handle) = collider.handle {
            if let Some(layer) = layers.mask_layer(collider.mask) {
                layer.set_shape(handle, *shape);
            }
        }
    });
}

pub fn remove_grid_colliders(
    mut layers: ResMut<CollisionLayers>,
    removed: RemovedComponents<Collider>,
) {
    for entity in removed.iter() {
        layers.remove(entity);
    }
}
