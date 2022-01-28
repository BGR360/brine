use std::{
    f32::consts::PI,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use bevy::{
    log::{Level, LogSettings},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy_inspector_egui::WorldInspectorPlugin;

use brine_chunk::{Chunk, ChunkSection};
use brine_proto::{event, ProtocolPlugin};
use brine_voxel::chunk_builder::{
    component::{BuiltChunk, BuiltChunkSection, Chunk as ChunkComponent},
    ChunkBuilder, ChunkBuilderPlugin, GreedyQuadsChunkBuilder, NaiveBlocksChunkBuilder,
    VisibleFacesChunkBuilder,
};

use brine::{
    chunk::{load_chunk, Result},
    error::log_error,
    DEFAULT_LOG_FILTER,
};
use clap::ArgEnum;

/// Loads a chunk from a file and views it in 3D.
#[derive(clap::Args)]
pub struct Args {
    /// Paths to one or more chunk data files to load.
    files: Vec<PathBuf>,

    /// Which chunk builder to test.
    #[clap(arg_enum, short, long, default_value = "visible_faces")]
    builder: ChunkBuilderType,
}

#[derive(Clone, ArgEnum)]
#[clap(rename_all = "snake_case")]
enum ChunkBuilderType {
    VisibleFaces,
    GreedyQuads,
}

struct Chunks {
    files: Vec<PathBuf>,
    next_file: usize,

    chunk: Option<Chunk>,
    next_section: usize,
}

impl Chunks {
    fn new(files: Vec<PathBuf>) -> Self {
        Self {
            files,
            next_file: 0,

            chunk: None,
            next_section: 0,
        }
    }

    fn chunk(&self) -> &Chunk {
        self.chunk.as_ref().unwrap()
    }

    fn next_file(&mut self) -> &Path {
        let path = &self.files[self.next_file];
        self.next_file = (self.next_file + 1) % self.files.len();
        path
    }

    fn load_next_file(&mut self) -> Result<()> {
        let path = self.next_file();
        let chunk = load_chunk(path)?;
        self.next_section = chunk.sections.len() - 1;
        self.chunk = Some(chunk);
        Ok(())
    }

    fn next_section(&mut self) -> ChunkSection {
        let sections = &self.chunk().sections;
        let section = sections[self.next_section].clone();
        self.next_section = if self.next_section == 0 {
            sections.len() - 1
        } else {
            self.next_section - 1
        };
        section
    }

    fn send_next_section(&mut self, chunk_events: &mut EventWriter<event::clientbound::ChunkData>) {
        let section = self.next_section();

        let single_section_chunk = Chunk {
            sections: vec![section],
            ..Chunk::empty(self.chunk().chunk_x, self.chunk().chunk_z)
        };

        chunk_events.send(event::clientbound::ChunkData {
            chunk_data: single_section_chunk,
        });
    }
}

const DISTANCE_FROM_ORIGIN: f32 = 13.0;

pub fn main(args: Args) {
    let mut app = App::new();

    app.insert_resource(LogSettings {
        level: Level::DEBUG,
        filter: String::from(DEFAULT_LOG_FILTER),
    })
    .insert_resource(WgpuOptions {
        features: WgpuFeatures::POLYGON_MODE_LINE,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .insert_resource(Msaa { samples: 4 })
    .insert_resource(WireframeConfig { global: true })
    .add_plugin(WireframePlugin)
    .add_plugin(WorldInspectorPlugin::new())
    .add_plugin(ProtocolPlugin);

    app.add_plugin(ChunkViewerPlugin::<NaiveBlocksChunkBuilder>::on_left());

    match args.builder {
        ChunkBuilderType::VisibleFaces => {
            app.add_plugin(ChunkViewerPlugin::<VisibleFacesChunkBuilder>::on_right());
        }
        ChunkBuilderType::GreedyQuads => {
            app.add_plugin(ChunkViewerPlugin::<GreedyQuadsChunkBuilder>::on_right());
        }
    }

    app.add_startup_system(load_first_chunk.chain(log_error))
        .add_startup_system(set_up_camera)
        .add_system(load_next_chunk.chain(log_error));

    app.insert_resource(Chunks::new(args.files));
    app.run();
}

fn load_first_chunk(
    mut chunks: ResMut<Chunks>,
    mut chunk_events: EventWriter<event::clientbound::ChunkData>,
) -> Result<()> {
    chunks.load_next_file()?;
    chunks.send_next_section(&mut chunk_events);
    Ok(())
}

fn load_next_chunk(
    input: Res<Input<KeyCode>>,
    mut chunks: ResMut<Chunks>,
    mut chunk_events: EventWriter<event::clientbound::ChunkData>,
    query: Query<Entity, With<ChunkComponent>>,
    mut commands: Commands,
) -> Result<()> {
    let should_show_next =
        input.just_pressed(KeyCode::Return) || input.just_pressed(KeyCode::Space);
    let should_load_next_file = input.just_pressed(KeyCode::Return);

    if should_load_next_file {
        chunks.load_next_file()?;
    }

    if should_show_next {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        chunks.send_next_section(&mut chunk_events);
    }

    Ok(())
}

fn set_up_camera(mut commands: Commands) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 8.0, 38.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

struct ChunkViewerPlugin<T> {
    position: Vec3,

    _phantom: PhantomData<T>,
}

impl<T> Plugin for ChunkViewerPlugin<T>
where
    T: ChunkBuilder + Default + Send + Sync + 'static,
    <T as ChunkBuilder>::Output: Send,
{
    fn build(&self, app: &mut App) {
        app.add_plugin(ChunkBuilderPlugin::<T>::shared());
        app.add_system(Self::center_section_at_bottom_of_chunk);
        app.add_system(Self::rename_chunks);
        app.add_system(Self::add_material);

        let position = self.position;
        app.add_system(move |query: Query<&mut Transform, Added<BuiltChunk<T>>>| {
            Self::move_and_rotate(query, position)
        });

        app.add_system(Self::rotate_chunk);
    }
}

impl<T> ChunkViewerPlugin<T>
where
    T: ChunkBuilder + Send + Sync + 'static,
{
    pub fn on_left() -> Self {
        Self::at_position(-Vec3::X * DISTANCE_FROM_ORIGIN)
    }

    pub fn on_right() -> Self {
        Self::at_position(Vec3::X * DISTANCE_FROM_ORIGIN)
    }

    pub fn at_position(position: Vec3) -> Self {
        Self {
            position,
            _phantom: PhantomData,
        }
    }

    fn move_and_rotate(mut query: Query<&mut Transform, Added<BuiltChunk<T>>>, position: Vec3) {
        for mut transform in query.iter_mut() {
            transform.translation = position;
            transform.rotate(Quat::from_rotation_y(PI / 4.0));
        }
    }

    fn rename_chunks(mut query: Query<&mut Name, (With<BuiltChunk<T>>, Added<Name>)>) {
        let builder_name = std::any::type_name::<T>().split("::").last().unwrap();
        for mut name in query.iter_mut() {
            let new_name = format!("{} ({})", **name, builder_name);
            name.set(new_name);
        }
    }

    fn center_section_at_bottom_of_chunk(
        mut query: Query<&mut Transform, Added<BuiltChunkSection<T>>>,
    ) {
        for mut transform in query.iter_mut() {
            transform.translation = Vec3::new(-8.0, -8.0, -8.0);
        }
    }

    fn add_material(
        assets: Res<AssetServer>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        query: Query<Entity, Added<Handle<Mesh>>>,
        mut commands: Commands,
    ) {
        let texture: Handle<Image> = assets.load("minecraft-texturesheet.png");
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            ..Default::default()
        });
        for entity in query.iter() {
            commands.entity(entity).insert(material.clone());
        }
    }

    fn rotate_chunk(
        input: Res<Input<KeyCode>>,
        mut query: Query<&mut Transform, With<BuiltChunk<T>>>,
    ) {
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
}
