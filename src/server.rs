use std::{
    any::Any,
    fs,
    path::{Path, PathBuf},
};

use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};

use brine_chunk::Chunk;
use brine_proto::event::clientbound::ChunkData;
use futures_lite::future;

use crate::{
    chunk::{load_chunk, Result},
    error::{exit_on_error, log_error},
};

/// A plugin that acts as a phony server, sending ChunkData events containing
/// data read from a directory of chunk data files.
pub struct ServeChunksFromDirectoryPlugin<P> {
    path: P,
}

impl<P> ServeChunksFromDirectoryPlugin<P> {
    pub fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P> Plugin for ServeChunksFromDirectoryPlugin<P>
where
    P: AsRef<Path> + Any + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        let path = PathBuf::from(self.path.as_ref());
        app.insert_resource(ChunkDirectory { path });
        app.add_startup_system(load_chunks.chain(exit_on_error));
        app.add_system(send_chunks.chain(log_error));
    }
}

#[derive(Debug)]
pub struct ChunkDirectory {
    path: PathBuf,
}

type LoadChunkTask = Task<Result<Chunk>>;

fn load_chunks(
    chunk_directory: Res<ChunkDirectory>,
    task_pool: Res<IoTaskPool>,
    mut commands: Commands,
) -> Result<()> {
    for entry in fs::read_dir(&chunk_directory.path)? {
        let entry = entry?;

        let task: LoadChunkTask = task_pool.spawn(async move { load_chunk(entry.path()) });

        commands.spawn().insert(task);
    }

    Ok(())
}

fn send_chunks(
    mut tasks: Query<(Entity, &mut LoadChunkTask)>,
    mut chunk_events: EventWriter<ChunkData>,
    mut commands: Commands,
) -> Result<()> {
    for (task_entity, mut task) in tasks.iter_mut() {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut *task)) {
            let chunk_data = chunk_data?;
            chunk_events.send(ChunkData { chunk_data });

            commands.entity(task_entity).remove::<LoadChunkTask>();
        }
    }

    Ok(())
}
