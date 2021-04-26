use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy_tilemap::event::TilemapChunkEvent;
use bevy_tilemap::point::Point2;
use bevy_tilemap::prelude::*;
use rand::prelude::*;

pub fn load_resources(
    mut cmd: Commands,
    server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    cmd.spawn_bundle(OrthographicCameraBundle::new_2d());

    let tilemap_tex: Handle<Texture> = server.load("textures/tilemap_dense.png");
    let tilemap_atlas = TextureAtlas::from_grid_with_padding(
        tilemap_tex,
        Vec2::new(32., 32.),
        4,
        4,
        Vec2::new(1., 1.),
    );
    let tilemap_atlas = texture_atlases.add(tilemap_atlas);

    let mut tilemap = Tilemap::builder()
        .auto_chunk()
        .auto_spawn(2, 0) // BUG: x & y are reversed
        .texture_atlas(tilemap_atlas)
        .texture_dimensions(32, 32)
        .chunk_dimensions(32, 32, 1)
        .finish()
        .unwrap();

    tilemap.insert_chunk((0, 0)).unwrap();
    tilemap.spawn_chunk((0, 0)).unwrap();

    cmd.spawn_bundle(TilemapBundle {
        tilemap,
        visible: Visible {
            is_transparent: true,
            is_visible: true,
        },
        transform: Default::default(),
        global_transform: Default::default(),
    });
}

pub fn update_new_chunks(
    tilemap_query: Query<&mut Tilemap>,
    mut spawned_chunks: Local<Vec<Point2>>,
    mut tiles: Local<Vec<usize>>,
) {
    tilemap_query.for_each_mut(|mut tilemap| {
        {
            let mut reader = tilemap.chunk_events().get_reader();
            for event in reader.iter(&tilemap.chunk_events()) {
                match event {
                    TilemapChunkEvent::Spawned { point } => {
                        spawned_chunks.push(*point);
                    }
                    _ => {}
                }
            }
        }

        let chunk_size = Point2::new(tilemap.chunk_width() as i32, tilemap.chunk_height() as i32);
        let half_size = chunk_size / Point2::new(2, 2);

        tiles.resize_with((chunk_size.x * chunk_size.y) as usize, || 0);

        fn randomize_tiles(rng: &mut ThreadRng, tiles: &mut Vec<usize>, chunk_size: Point2) {
            let mut chunk_tiles = ChunkTiles {
                rng,
                tiles,
                size: chunk_size,
            };

            chunk_tiles.reset();
            chunk_tiles.add_lava_cliff();
            chunk_tiles.add_lava_fall();
            chunk_tiles.add_full_wall();
        }

        let mut rng = rand::thread_rng();
        for chunk in spawned_chunks.drain(..) {
            randomize_tiles(&mut rng, &mut tiles, chunk_size);

            let map_tiles = tiles.iter().enumerate().map(|(i, tile_index)| Tile {
                point: (chunk * chunk_size
                    + Point2::new(i as i32 % chunk_size.x, i as i32 / chunk_size.x))
                    - half_size,
                sprite_index: *tile_index,
                ..Default::default()
            });

            tilemap.insert_tiles(map_tiles).unwrap();
        }
    });
}

const WALL_LAVA_PROB: f64 = 0.6;
const FULL_WALL_PROB: f64 = 0.2;

const WALL_WIDTH: i32 = 5;
const LAVA_CLIFF_WIDTH: i32 = 5;
const FULL_WALL_HEIGHT: i32 = 5;

struct ChunkTiles<'a> {
    rng: &'a mut ThreadRng,
    size: Point2,
    tiles: &'a mut Vec<usize>,
}

impl<'a> ChunkTiles<'a> {
    pub fn reset(&mut self) {
        self.tiles.fill(0);

        self.fill_area(
            Point2::new(0, 0),
            Point2::new(WALL_WIDTH, self.size.y),
            1
        );

        self.fill_area(
            Point2::new(self.size.x - WALL_WIDTH, 0),
            Point2::new(WALL_WIDTH, self.size.y),
            1
        );
    }

    pub fn add_lava_fall(&mut self) -> bool {
        if !self.rng.gen_bool(FULL_WALL_PROB) {
            return false
        }

        let lava_fall_size = Point2::new(3, 5);
        let fall_start = Point2::new(
            self.rng.gen_range(WALL_WIDTH + 2 .. (self.size.y - WALL_WIDTH + 2)),
            self.rng.gen_range(0..(self.size.y - lava_fall_size.y)),
        );

        self.fill_area(fall_start, lava_fall_size, 3);
        true
    }

    pub fn add_lava_cliff(&mut self) -> bool {
        if !self.rng.gen_bool(WALL_LAVA_PROB) {
            return false;
        }

        let lava_cliff_size = Point2::new(5, 2);
        let cliff_start = Point2::new(
            if self.rng.gen_bool(0.5) { WALL_WIDTH } else { self.size.x - LAVA_CLIFF_WIDTH - WALL_WIDTH },
            self.rng.gen_range(0..(self.size.y - lava_cliff_size.y)),
        );

        self.fill_area(cliff_start, lava_cliff_size, 3);
        true
    }

    pub fn add_full_wall(&mut self) -> bool {
        if !self.rng.gen_bool(FULL_WALL_PROB) {
            return false;
        }

        let wall_start = Point2::new(0, self.rng.gen_range(0..(self.size.y - FULL_WALL_HEIGHT)));
        let wall_size = Point2::new(
            self.size.x,
            FULL_WALL_HEIGHT
        );

        self.fill_area(wall_start, wall_size, 2);

        true
    }

    fn fill_area(&mut self, start: Point2, size: Point2, value: usize) {
        for p in grid_points(size) {
            let p = p + start;
            self.set_tile(p, value);
        }
    }

    fn set_tile(&mut self, pos: Point2, tile: usize) {
        self.tiles[point_i(self.size, pos.x, pos.y)] = tile;
    }
}

pub struct GridPoints {
    i: usize,
    width: usize,
    items: usize,
}

pub fn point_i(grid: Point2, x: i32, y: i32) -> usize {
    (x + grid.x * y) as usize
}

pub fn grid_points(size: Point2) -> GridPoints {
    GridPoints {
        i: 0,
        width: size.x as usize,
        items: (size.x * size.y) as usize,
    }
}

impl Iterator for GridPoints {
    type Item = Point2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.items {
            None
        } else {
            let point = Point2::new((self.i % self.width) as i32, (self.i / self.width) as i32);
            self.i += 1;
            Some(point)
        }
    }
}
