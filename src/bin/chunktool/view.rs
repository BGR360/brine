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
    ChunkBuilderPlugin, GreedyQuadsChunkBuilder, NaiveBlocksChunkBuilder, VisibleFacesChunkBuilder,
};

use brine::{
    chunk::{load_chunk, Result},
    error::log_error,
};
use clap::ArgEnum;

/// Loads a chunk from a file and views it in 3D.
#[derive(clap::Args)]
pub struct Args {
    /// Path to a chunk data file to load.
    file: PathBuf,

    /// Which chunk builder to test.
    #[clap(arg_enum, short, long, default_value = "visible_faces")]
    builder: ChunkBuilder,
}

#[derive(Clone, ArgEnum)]
#[clap(rename_all = "snake_case")]
enum ChunkBuilder {
    VisibleFaces,
    GreedyQuads,
}

pub fn main(args: Args) {
    println!("{}", args.file.to_string_lossy());

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WireframeConfig { global: true })
        .add_plugin(WireframePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ProtocolPlugin);

    app.add_plugin(ChunkBuilderPlugin::<NaiveBlocksChunkBuilder>::shared())
        .add_system(center_section_at_bottom_of_chunk::<NaiveBlocksChunkBuilder>)
        .add_system(put_chunk_on_left::<NaiveBlocksChunkBuilder>);

    match args.builder {
        ChunkBuilder::VisibleFaces => {
            app.add_plugin(ChunkBuilderPlugin::<VisibleFacesChunkBuilder>::shared())
                .add_system(center_section_at_bottom_of_chunk::<VisibleFacesChunkBuilder>)
                .add_system(put_chunk_on_right::<VisibleFacesChunkBuilder>);
        }
        ChunkBuilder::GreedyQuads => {
            app.add_plugin(ChunkBuilderPlugin::<GreedyQuadsChunkBuilder>::shared())
                .add_system(center_section_at_bottom_of_chunk::<GreedyQuadsChunkBuilder>)
                .add_system(put_chunk_on_right::<GreedyQuadsChunkBuilder>);
        }
    }

    app.add_startup_system(load_chunk_system.chain(log_error))
        .add_startup_system(set_up_camera)
        .add_system(rotate_chunk);

    app.insert_resource(args);
    app.run();
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

fn set_up_camera(mut commands: Commands) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 8.0, 38.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn put_chunk_on_left<T: Send + Sync + 'static>(
    query: Query<(Entity, &mut Transform), Added<BuiltChunk<T>>>,
    commands: Commands,
) {
    move_and_name(query, commands, Vec3::X * -13.0);
}

fn put_chunk_on_right<T: Send + Sync + 'static>(
    query: Query<(Entity, &mut Transform), Added<BuiltChunk<T>>>,
    commands: Commands,
) {
    move_and_name(query, commands, Vec3::X * 13.0);
}

fn move_and_name<T: Send + Sync + 'static>(
    mut query: Query<(Entity, &mut Transform), Added<BuiltChunk<T>>>,
    mut commands: Commands,
    position: Vec3,
) {
    for (entity, mut transform) in query.iter_mut() {
        transform.translation = position;
        transform.rotate(Quat::from_rotation_y(PI / 4.0));

        let name = std::any::type_name::<T>();
        commands.entity(entity).insert(Name::new(name));
    }
}

fn center_section_at_bottom_of_chunk<T: Send + Sync + 'static>(
    mut query: Query<&mut Transform, Added<BuiltChunkSection<T>>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::new(-8.0, -8.0, -8.0);
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
