use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};

use brine_chunk::{BlockState, BlockStates, ChunkSection, BLOCKS_PER_SECTION};
use brine_data::MinecraftData;
use brine_render::chunk::ChunkBakery;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .insert_resource(WireframeConfig { global: true })
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .run();
}

fn random_chunk() -> ChunkSection {
    let mut block_states = [BlockState::AIR; BLOCKS_PER_SECTION];

    let mut block_count = 0;
    for block_state in block_states.iter_mut() {
        if fastrand::f32() >= 0.8 {
            *block_state = BlockState(1);
            block_count += 1;
        }
    }

    ChunkSection {
        block_count,
        chunk_y: 0,
        block_states: BlockStates(block_states),
    }
}

fn bake_chunk(chunk: &ChunkSection) -> Mesh {
    let mc_data = MinecraftData::for_version("1.14.4");

    let chunk_bakery = ChunkBakery::new(&mc_data);

    let baked_chunk = chunk_bakery.bake_chunk(chunk);

    baked_chunk.mesh
}

fn setup(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    let chunk = random_chunk();

    let mesh = bake_chunk(&chunk);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        ..Default::default()
    });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(30.0, 24.0, 30.0))
            .looking_at(Vec3::ONE * 8.0, Vec3::Y),
        ..Default::default()
    });
}
