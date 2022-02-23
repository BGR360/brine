use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy_inspector_egui::WorldInspectorPlugin;

use brine_asset::MinecraftAssets;
use brine_chunk::{BlockState, BlockStates, ChunkSection, BLOCKS_PER_SECTION};
use brine_data::MinecraftData;
use brine_render::chunk::ChunkBakery;

fn main() {
    let mc_data = MinecraftData::for_version("1.14.4");
    let mc_assets = MinecraftAssets::new("assets/1.14.4", &mc_data).unwrap();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .insert_resource(WireframeConfig { global: true })
        .add_plugin(WireframePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(mc_data)
        .insert_resource(mc_assets)
        .add_startup_system(setup)
        .run();
}

fn random_block_state() -> BlockState {
    let id = fastrand::u32(1..10000);
    BlockState(id)
}

fn random_chunk() -> ChunkSection {
    let mut block_states = [BlockState::AIR; BLOCKS_PER_SECTION];

    let mut block_count = 0;
    for block_state in block_states.iter_mut() {
        if fastrand::f32() >= 0.9 {
            *block_state = random_block_state();
            block_count += 1;
        }
    }

    ChunkSection {
        block_count,
        chunk_y: 0,
        block_states: BlockStates(block_states),
    }
}

fn bake_chunk(chunk: &ChunkSection, mc_data: &MinecraftData, mc_assets: &MinecraftAssets) -> Mesh {
    let chunk_bakery = ChunkBakery::new(mc_data, mc_assets);

    let baked_chunk = chunk_bakery.bake_chunk(chunk);

    baked_chunk.mesh
}

fn setup(
    mc_data: Res<MinecraftData>,
    mc_assets: Res<MinecraftAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let chunk = random_chunk();

    let mesh = bake_chunk(&chunk, &*mc_data, &*mc_assets);

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
