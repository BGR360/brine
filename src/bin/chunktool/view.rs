use std::{f32::consts::PI, path::PathBuf};

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy_inspector_egui::WorldInspectorPlugin;

use brine_proto::{event, ProtocolPlugin};
use brine_voxel::chunk_builder::{
    component::{BuiltChunk, BuiltChunkSection, Chunk},
    ChunkBuilderPlugin, NaiveBlocksChunkBuilder, VisibleFacesChunkBuilder,
};

use brine::{
    chunk::{load_chunk, Result},
    error::log_error,
};

/// Loads a chunk from a file and views it in 3D.
#[derive(clap::Args)]
pub struct Args {
    /// Path to a chunk data file to load.
    file: PathBuf,
}

pub fn main(args: Args) {
    println!("{}", args.file.to_string_lossy());

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugin(WireframePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ProtocolPlugin)
        .insert_resource(args)
        .add_plugin(ChunkBuilderPlugin::<NaiveBlocksChunkBuilder>::shared())
        .add_plugin(ChunkBuilderPlugin::<VisibleFacesChunkBuilder>::shared())
        //.add_plugin(ChunkViewerPlugin)
        .add_startup_system(load_chunk_system.chain(log_error))
        .add_startup_system(setup_point_light_and_camera)
        //.add_system(add_material)
        .add_system(put_naive_block_chunk_on_left)
        .add_system(put_visible_faces_chunk_on_right)
        .add_system(center_section_at_bottom_of_chunk::<NaiveBlocksChunkBuilder>)
        .add_system(center_section_at_bottom_of_chunk::<VisibleFacesChunkBuilder>)
        .add_system(rotate_chunk)
        .run();
}

#[derive(Component)]
struct LoadChunk(PathBuf);

fn load_chunk_system(
    args: Res<Args>,
    mut chunk_events: EventWriter<event::clientbound::ChunkData>,
) -> Result<()> {
    let chunk = load_chunk(&args.file)?;
    debug!("loaded chunk: {:#?}", chunk);

    let sections = chunk.data.sections();
    let section = sections[sections.len() - 1].clone();

    let single_section_chunk = brine_chunk::Chunk {
        data: brine_chunk::ChunkData::Full {
            sections: vec![section],
            biomes: Default::default(),
        },
        ..chunk
    };

    chunk_events.send(event::clientbound::ChunkData {
        chunk_data: single_section_chunk,
    });

    Ok(())
}

fn setup_point_light_and_camera(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = true;

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(25.0, 25.0, 25.0)),
        point_light: PointLight {
            range: 200.0,
            intensity: 8000.0,
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 8.0, 38.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn _add_material(
    mut material: Local<Option<Handle<StandardMaterial>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<Entity, Added<Handle<Mesh>>>,
    mut commands: Commands,
) {
    if material.is_none() {
        let mut new_mat = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
        new_mat.perceptual_roughness = 0.9;

        let handle = materials.add(new_mat);
        *material = Some(handle);
    }

    for entity in query.iter() {
        debug!("Adding material");
        commands.entity(entity).insert(material.clone().unwrap());
    }
}

fn put_naive_block_chunk_on_left(
    mut query: Query<&mut Transform, Added<BuiltChunk<NaiveBlocksChunkBuilder>>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::X * -12.0;
        transform.rotate(Quat::from_rotation_y(PI / 4.0));
    }
}

fn put_visible_faces_chunk_on_right(
    mut query: Query<&mut Transform, Added<BuiltChunk<VisibleFacesChunkBuilder>>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::X * 12.0;
        transform.rotate(Quat::from_rotation_y(PI / 4.0));
    }
}

fn center_section_at_bottom_of_chunk<T: Send + Sync + 'static>(
    mut query: Query<&mut Transform, Added<BuiltChunkSection<T>>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::new(-8.0, -8.0, -8.0);
        //transform.translation.y = 0.0;
    }
}

fn rotate_chunk(input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Chunk>>) {
    for mut transform in query.iter_mut() {
        for keypress in input.get_just_pressed() {
            match keypress {
                KeyCode::Left => transform.rotate(Quat::from_rotation_y(-PI / 4.0)),
                KeyCode::Right => transform.rotate(Quat::from_rotation_y(PI / 4.0)),
                KeyCode::Down => transform.rotate(Quat::from_rotation_x(PI / 4.0)),
                KeyCode::Up => transform.rotate(Quat::from_rotation_x(-PI / 4.0)),
                KeyCode::Escape => transform.rotation = Quat::from_rotation_y(PI / 4.0),
                _ => {}
            }
        }
    }
}
