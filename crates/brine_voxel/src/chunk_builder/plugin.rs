use std::{any::Any, marker::PhantomData};

use bevy::{
    ecs::event::Events,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;

use brine_chunk::Chunk;
use brine_proto::event;

use super::component::{BuiltChunk, BuiltChunkSection, ChunkSection};

use super::{
    component::{BuiltChunkBundle, BuiltChunkSectionBundle},
    ChunkBuilder, ChunkMeshes, SectionMesh,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum System {
    BuilderTaskSpawn,
    BuilderResultAddToWorld,
}

type ChunkBuilderTask<T> = Task<(Chunk, ChunkMeshes<T>)>;

/// Plugin that asynchronously generates renderable entities from chunk data.
///
/// The [`ChunkBuilderPlugin`] listens for [`ChunkData`] events from the backend
/// and spawns a task to run a particular [`ChunkBuilder`]. When the task
/// completes, the plugin adds the result to the game world.
///
/// [`ChunkData`]: brine_proto::event::clientbound::ChunkData
pub struct ChunkBuilderPlugin<T: ChunkBuilder> {
    shared: bool,
    _phantom: PhantomData<T>,
}

impl<T: ChunkBuilder> ChunkBuilderPlugin<T> {
    /// For (potentially premature) performance reasons, the default behavior of
    /// the [`ChunkBuilderPlugin`] is to consume `ChunkData` events (i.e.,
    /// [`Events::drain()`]) so they can be moved into the builder task rather
    /// than cloned.
    ///
    /// [`Events::drain()`]: bevy_ecs::event::Events::drain
    ///
    /// This constructor allows multiple chunk builder plugins to exist
    /// simultaneously without them clobbering each other. It forces the plugin
    /// to use a regular old [`EventReader`] rather than draining the events.
    pub fn shared() -> Self {
        Self {
            shared: true,
            ..Default::default()
        }
    }
}

impl<T: ChunkBuilder> Default for ChunkBuilderPlugin<T> {
    fn default() -> Self {
        Self {
            shared: false,
            _phantom: PhantomData,
        }
    }
}

impl<T> Plugin for ChunkBuilderPlugin<T>
where
    T: ChunkBuilder + Default + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        let mut systems = SystemSet::new();

        systems = if self.shared {
            systems.with_system(Self::builder_task_spawn_shared.label(System::BuilderTaskSpawn))
        } else {
            systems.with_system(Self::builder_task_spawn_unique.label(System::BuilderTaskSpawn))
        };

        systems = systems
            .with_system(Self::builder_result_add_to_world.label(System::BuilderResultAddToWorld))
            .with_system(Self::add_names);

        app.add_system_set(systems);
    }
}

impl<T> ChunkBuilderPlugin<T>
where
    T: ChunkBuilder + Default + Any + Send + Sync + 'static,
{
    fn builder_task_spawn(
        current_chunk: &mut usize,
        chunk_event: event::clientbound::ChunkData,
        commands: &mut Commands,
        task_pool: &AsyncComputeTaskPool,
    ) {
        let chunk = chunk_event.chunk_data;
        if !chunk.is_full() {
            return;
        }

        debug!("Received chunk, spawning task");

        let task: ChunkBuilderTask<T> = task_pool.spawn(async move {
            let built = T::default().build_chunk(&chunk);
            (chunk, built)
        });

        commands.spawn().insert(task);

        *current_chunk += 1;
    }

    fn builder_task_spawn_unique(
        mut current_chunk: Local<usize>,
        mut chunk_events: ResMut<Events<event::clientbound::ChunkData>>,
        mut commands: Commands,
        task_pool: Res<AsyncComputeTaskPool>,
    ) {
        for chunk_event in chunk_events.drain() {
            Self::builder_task_spawn(&mut *current_chunk, chunk_event, &mut commands, &task_pool);
        }
    }

    fn builder_task_spawn_shared(
        mut current_chunk: Local<usize>,
        mut chunk_events: EventReader<event::clientbound::ChunkData>,
        mut commands: Commands,
        task_pool: Res<AsyncComputeTaskPool>,
    ) {
        for chunk_event in chunk_events.iter() {
            Self::builder_task_spawn(
                &mut *current_chunk,
                chunk_event.clone(),
                &mut commands,
                &task_pool,
            );
        }
    }

    fn builder_result_add_to_world(
        mut meshes: ResMut<Assets<Mesh>>,
        mut built_chunks: Query<(Entity, &mut ChunkBuilderTask<T>)>,
        mut commands: Commands,
    ) {
        for (task_entity, mut task) in built_chunks.iter_mut() {
            if let Some((chunk, built_chunk)) = future::block_on(future::poll_once(&mut *task)) {
                debug!("Spawning chunk stuff");

                let meshes = &mut *meshes;
                commands
                    .spawn()
                    .insert_bundle(BuiltChunkBundle::new(T::TYPE, chunk.chunk_x, chunk.chunk_z))
                    .with_children(move |parent| {
                        for (section, SectionMesh { mesh, .. }) in chunk
                            .sections
                            .into_iter()
                            .zip(built_chunk.sections.into_iter())
                        {
                            parent
                                .spawn()
                                .insert_bundle(BuiltChunkSectionBundle::new(
                                    T::TYPE,
                                    section.chunk_y,
                                ))
                                .insert_bundle(PbrBundle {
                                    mesh: meshes.add(mesh.to_render_mesh()),
                                    ..Default::default()
                                })
                                .insert(ChunkSection(section));
                        }
                    });

                // Task is complete, so remove task component from entity
                commands.entity(task_entity).remove::<ChunkBuilderTask<T>>();
            }
        }
    }

    fn add_names(
        built_chunks: Query<(Entity, &BuiltChunk), Added<BuiltChunk>>,
        built_sections: Query<(Entity, &BuiltChunkSection), Added<BuiltChunkSection>>,
        mut commands: Commands,
    ) {
        for (entity, built_chunk) in built_chunks.iter() {
            if built_chunk.builder == T::TYPE {
                commands
                    .entity(entity)
                    .insert(Name::new(built_chunk.to_string()));
            }
        }
        for (entity, built_section) in built_sections.iter() {
            if built_section.builder == T::TYPE {
                commands
                    .entity(entity)
                    .insert(Name::new(built_section.to_string()));
            }
        }
    }
}
