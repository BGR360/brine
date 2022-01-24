use std::{f32::consts::PI, fmt, fs, io, path::PathBuf};

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy_inspector_egui::WorldInspectorPlugin;

use brine_chunk::Error as ChunkError;
use brine_proto::{event, ProtocolPlugin};
use brine_proto_backend::{
    backend_stevenarella::{
        chunks::get_chunk_from_packet,
        codec::{Direction, Error as PacketError, MinecraftCodec},
    },
    codec::MinecraftProtocolState,
};
use brine_voxel::chunk_builder::{
    component::{BuiltChunk, BuiltChunkSection, Chunk},
    ChunkBuilderPlugin, NaiveBlocksChunkBuilder, VisibleFacesChunkBuilder,
};

/// Loads a chunk from a file and views it in 3D.
#[derive(clap::Args)]
pub struct Args {
    /// Path to a chunk data file to load.
    file: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the loaded file does not contain valid chunk data")]
    NotAChunk,

    #[error(transparent)]
    Packet(#[from] PacketError),

    #[error(transparent)]
    Chunk(#[from] ChunkError),

    #[error(transparent)]
    Io(#[from] io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

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
        .add_startup_system(load_chunk.chain(handle_error))
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

fn handle_error<E: fmt::Display>(In(result): In<Result<(), E>>) {
    if let Err(e) = result {
        error!("{}", e);
    }
}

fn load_chunk(
    args: Res<Args>,
    mut chunk_events: EventWriter<event::clientbound::ChunkData>,
) -> Result<()> {
    let path = &args.file;
    let file_bytes = fs::read(path)?;

    let chunk = chunk_from_bytes(&file_bytes)?;
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

fn chunk_from_bytes(mut reader: &[u8]) -> Result<brine_chunk::Chunk> {
    const CHUNK_DATA_PACKET_ID: i32 = 0x21;

    let packet = MinecraftCodec::decode_packet_with_id(
        498,
        MinecraftProtocolState::Play,
        Direction::Clientbound,
        CHUNK_DATA_PACKET_ID,
        &mut reader,
    )?;

    let chunk = get_chunk_from_packet(&packet)?;

    match chunk {
        Some(chunk) => Ok(chunk),
        None => Err(Error::NotAChunk),
    }
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
