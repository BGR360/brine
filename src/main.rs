//! The Brine Minecraft client entrypoint.

use std::path::PathBuf;

use bevy::{
    log::{Level, LogSettings},
    prelude::*,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::prelude::*;
use brine_asset::MinecraftAssets;
use brine_data::MinecraftData;
use clap::Parser;

use brine_proto::{AlwaysSuccessfulLoginPlugin, ProtocolPlugin};
use brine_proto_backend::ProtocolBackendPlugin;
use brine_voxel_v1::{
    chunk_builder::{
        component::BuiltChunkSection, ChunkBuilderPlugin, GreedyQuadsChunkBuilder,
        VisibleFacesChunkBuilder,
    },
    texture::TextureBuilderPlugin,
};

use brine::{
    debug::DebugWireframePlugin, login::LoginPlugin, server::ServeChunksFromDirectoryPlugin,
    DEFAULT_LOG_FILTER,
};

const SERVER: &str = "localhost:25565";
const USERNAME: &str = "user";

/// Brine Minecraft Client
#[derive(Parser)]
struct Args {
    /// Run with additional debug utilities (e.g., egui inspector).
    #[clap(short, long)]
    debug: bool,

    /// Run with a fake server that serves chunks from a directory of chunk files.
    #[clap(name = "chunks", long, value_name = "CHUNK_DIR")]
    chunk_dir: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();

    // Default plugins.

    app.insert_resource(LogSettings {
        level: Level::DEBUG,
        filter: String::from(DEFAULT_LOG_FILTER),
    });
    app.add_plugins(DefaultPlugins);

    // Brine-specific plugins.

    app.add_plugin(ProtocolPlugin);

    if let Some(chunk_dir) = args.chunk_dir {
        app.add_plugin(AlwaysSuccessfulLoginPlugin);
        app.add_plugin(ServeChunksFromDirectoryPlugin::new(chunk_dir));
    } else {
        app.add_plugin(ProtocolBackendPlugin);
        app.add_plugin(
            LoginPlugin::new(SERVER.to_string(), USERNAME.to_string()).exit_on_disconnect(),
        );
    }

    let mc_data = MinecraftData::for_version("1.14.4");
    let mc_assets = MinecraftAssets::new("assets/1.14.4", &mc_data).unwrap();
    app.insert_resource(mc_data);
    app.insert_resource(mc_assets);
    app.add_plugin(TextureBuilderPlugin);

    app.add_plugin(MinecraftWorldViewerPlugin);

    // Debugging, diagnostics, and utility plugins.

    if args.debug {
        app.add_plugin(WorldInspectorPlugin::new());
    }

    app.run();
}

#[derive(Default)]
pub struct MinecraftWorldViewerPlugin;

impl Plugin for MinecraftWorldViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa { samples: 4 })
            .add_plugin(DebugWireframePlugin)
            .add_plugin(FlyCameraPlugin)
            .add_plugin(ChunkBuilderPlugin::<VisibleFacesChunkBuilder>::default())
            // .add_plugin(ChunkBuilderPlugin::<GreedyQuadsChunkBuilder>::default())
            .add_startup_system(set_up_camera)
            .add_system(give_chunk_sections_correct_y_height);
    }
}

fn set_up_camera(mut commands: Commands) {
    // Screenshot coords.
    let camera_start = Transform::from_translation(Vec3::new(-200.0, 87.8, 157.3))
        .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.1338, 0.183, -0.025));

    // let camera_start = Transform::from_translation(Vec3::new(-260.0, 115.0, 200.0))
    //     .looking_at(Vec3::new(-40.0, 100.0, 0.0), Vec3::Y);

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: camera_start,
            ..Default::default()
        })
        .insert(FlyCamera::default());
}

fn give_chunk_sections_correct_y_height(mut query: Query<(&mut Transform, &BuiltChunkSection)>) {
    for (mut transform, chunk_section) in query.iter_mut() {
        let height = (chunk_section.section_y as f32) * 16.0;
        if transform.translation.y != height {
            transform.translation.y = height;
        }
    }
}
