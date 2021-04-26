use bevy::prelude::*;
use bevy::render::camera::Camera;

pub struct Player;

pub struct Health(u8);

pub struct Velocity(Vec2);

pub fn init_player(
    mut cmd: Commands,
    server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player_tex: Handle<Texture> = server.load("textures/player.png");

    cmd.spawn()
        .insert(Player)
        .insert(Velocity(Vec2::new(0., -10.)))
        .insert_bundle(SpriteBundle {
        material: materials.add(player_tex.into()),
        transform: Transform {
            translation: Vec3::new(0., 0., 10.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn follow_player(
    cameras: Query<(&Camera, &mut Transform)>,
    players: Query<(&Player, &Transform), Without<Camera>>
) {
    cameras.for_each_mut(|(camera, mut tx)| {
        let (_, player_tx) = players.single().unwrap();
        tx.translation = player_tx.translation;
    });
}

pub fn update_velocity(
    mut query: Query<(&Velocity, &mut Transform)>
) {
    query.for_each_mut(|(velocity, mut tx)| {
        tx.translation += velocity.0.extend(0.);
    });
}

pub fn update_collisions() {

}