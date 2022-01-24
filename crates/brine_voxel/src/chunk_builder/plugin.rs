use std::{any::Any, marker::PhantomData};

use bevy_app::prelude::*;
use bevy_asset::prelude::*;
use bevy_ecs::{event::Events, prelude::*};
use bevy_log::prelude::*;
use bevy_render::prelude::*;
use bevy_tasks::{prelude::*, Task};
use futures_lite::future;

use brine_proto::event;

use super::component::Chunk;

use super::{AddToWorld, ChunkBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum System {
    BuilderTaskSpawn,
    BuilderResultAddToWorld,
}

//const MAX_CHUNKS: usize = 3000;

type ChunkBuilderTask<T> = Task<(Chunk, <T as ChunkBuilder>::Output)>;

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
    <T as ChunkBuilder>::Output: Send + 'static,
{
    fn build(&self, app: &mut App) {
        if self.shared {
            app.add_system(Self::builder_task_spawn_shared.label(System::BuilderTaskSpawn));
        } else {
            app.add_system(Self::builder_task_spawn_unique.label(System::BuilderTaskSpawn));
        }

        app.add_system(Self::builder_result_add_to_world.label(System::BuilderResultAddToWorld));
    }
}

impl<T> ChunkBuilderPlugin<T>
where
    T: ChunkBuilder + Default + Any + Send + Sync + 'static,
    <T as ChunkBuilder>::Output: Send + 'static,
{
    fn builder_task_spawn(
        current_chunk: &mut usize,
        chunk_event: event::clientbound::ChunkData,
        commands: &mut Commands,
        task_pool: &AsyncComputeTaskPool,
    ) {
        let chunk = chunk_event.chunk_data;
        if let brine_chunk::ChunkData::Delta { .. } = chunk.data {
            return;
        }

        debug!("Chunk time!");

        let task: ChunkBuilderTask<T> = task_pool.spawn(async move {
            let built = T::default().build_chunk(&chunk);
            (Chunk(chunk), built)
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
        mut built_chunks: Query<(Entity, &mut ChunkBuilderTask<T>)>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut commands: Commands,
    ) {
        for (task_entity, mut task) in built_chunks.iter_mut() {
            if let Some((chunk, built_chunk)) = future::block_on(future::poll_once(&mut *task)) {
                debug!("Spawning chunk stuff");

                let chunk_entity = built_chunk.add_to_world(&mut meshes, &mut commands);

                commands.entity(chunk_entity).insert(chunk);

                // Task is complete, so remove task component from entity
                commands.entity(task_entity).remove::<ChunkBuilderTask<T>>();
            }
        }
    }
}
