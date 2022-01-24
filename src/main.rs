//! The Brine Minecraft client entrypoint.

use bevy::{
    log::{Level, LogSettings},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::prelude::*;

use brine_proto::ProtocolPlugin;
use brine_proto_backend::ProtocolBackendPlugin;
use brine_voxel::chunk_builder::{ChunkBuilderPlugin, VisibleFacesChunkBuilder};

use brine::{login::LoginPlugin, DEFAULT_LOG_FILTER};

const SERVER: &str = "localhost:25565";
const USERNAME: &str = "user";

fn main() {
    App::new()
        // Default plugins.
        .insert_resource(LogSettings {
            level: Level::INFO,
            filter: String::from(DEFAULT_LOG_FILTER),
        })
        .add_plugins(DefaultPlugins)
        // Debugging, diagnostics, and utility plugins.
        // .add_plugin(WorldInspectorPlugin::new())
        // Brine-specific plugins.
        .add_plugin(ProtocolPlugin)
        .add_plugin(ProtocolBackendPlugin)
        .add_plugin(LoginPlugin::new(SERVER.to_string(), USERNAME.to_string()))
        .add_plugin(MinecraftWorldViewerPlugin)
        .run();
}

#[derive(Default)]
pub struct MinecraftWorldViewerPlugin;

impl Plugin for MinecraftWorldViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .insert_resource(WireframeConfig { global: true })
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(WireframePlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(ChunkBuilderPlugin::<VisibleFacesChunkBuilder>::default())
        .add_startup_system(set_up_camera);
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
