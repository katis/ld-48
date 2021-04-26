use bevy::input::system::exit_on_esc_system;
use bevy::prelude::*;
use bevy::tasks::TaskPool;
use bevy_tilemap::event::TilemapChunkEvent;
use bevy_tilemap::point::Point2;
use bevy_tilemap::prelude::*;
use bevy_tilemap::prelude::*;
use framework::collisions::{
    check_collisions, remove_grid_colliders, reset_collisions, update_collider_shapes,
    update_collision_grids, CollisionLayer, CollisionLayers, CollisionMask,
};
use framework::movement::apply_movement;
use rand::prelude::*;
use bevy::render::camera::{Camera, camera_system, ScalingMode};
use bevy::sprite::TextureAtlasBuilder;

use logic::{map, player};

fn main() {
    let pool = TaskPool::new();

    let layers = CollisionLayers::new(vec![
        CollisionLayer::new(CollisionMask::Enemies, CollisionMask::Players.into(), 256),
        CollisionLayer::new(CollisionMask::Players, CollisionMask::Enemies.into(), 256),
    ]);

    App::build()
        .insert_resource(WindowDescriptor {
            title: "DgDig".into(),
            height: 32. * 32.,
            width: 32. * 32.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapDefaultPlugins)
        .insert_resource(pool)
        .insert_resource(layers)
        /*
        .add_system(apply_movement.system().label("apply_movement"))
        .add_system(check_collisions.system().after("apply_movement"))
        .add_system(
            update_collision_grids
                .system()
                .label("update_collision_grids"),
        )
        .add_system(
            update_collider_shapes
                .system()
                .after("update_collision_grids"),
        )
        .add_system(reset_collisions.system())
        .add_system(remove_grid_colliders.system())
         */
        .add_startup_system(player::init_player.system())
        .add_startup_system(map::load_resources.system())
        .add_system(player::update_velocity.system())
        .add_system(player::follow_player.system())
        .add_system(map::update_new_chunks.system())
        .add_system(exit_on_esc_system.system())
        .run();
}
