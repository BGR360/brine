use std::collections::hash_map::Entry;
use std::{any::Any, marker::PhantomData};

use bevy::tasks::Task;
use bevy::utils::{HashMap, HashSet};
use bevy::{ecs::event::Events, prelude::*, tasks::AsyncComputeTaskPool};
use futures_lite::future;

use brine_asset::api::BlockFace;
use brine_asset::{BlockStateId, MinecraftAssets};
use brine_chunk::ChunkSection;
use brine_proto::event;

use crate::chunk_builder::component::PendingChunk;
use crate::mesh::VoxelMesh;
use crate::texture::BlockTextures;

use super::component::{ChunkSection as ChunkSectionComponent, PendingMeshAtlas};

use super::{
    component::{BuiltChunkBundle, BuiltChunkSectionBundle},
    ChunkBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum System {
    BuilderTaskSpawn,
    BuilderResultAddToWorld,
}

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
    /// [`Events::drain()`]: bevy::ecs::event::Events::drain
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
            .with_system(Self::receive_built_meshes)
            .with_system(Self::add_built_chunks_to_world.label(System::BuilderResultAddToWorld));

        app.add_system_set(systems);
    }
}

type MesherTask = Task<(brine_chunk::Chunk, Vec<VoxelMesh>)>;

impl<T> ChunkBuilderPlugin<T>
where
    T: ChunkBuilder + Default + Any + Send + Sync + 'static,
{
    fn builder_task_spawn(
        chunk_event: event::clientbound::ChunkData,
        commands: &mut Commands,
        task_pool: &AsyncComputeTaskPool,
    ) {
        let chunk = chunk_event.chunk_data;
        if !chunk.is_full() {
            return;
        }

        let chunk_x = chunk.chunk_x;
        let chunk_z = chunk.chunk_z;

        debug!("Received chunk ({}, {}), spawning task", chunk_x, chunk_z);

        let task: MesherTask = task_pool.spawn(async move {
            let built = T::default().build_chunk(&chunk);
            (chunk, built)
        });

        commands.spawn().insert_bundle((
            task,
            PendingChunk::new(T::TYPE),
            Name::new(format!("Pending Chunk ({}, {})", chunk_x, chunk_z)),
        ));
    }

    fn build_texture_atlas_for_mesh(
        mesh: &VoxelMesh,
        chunk_section: &ChunkSection,
        asset_server: &AssetServer,
        mc_assets: &MinecraftAssets,
        texture_builder: &mut BlockTextures,
    ) -> PendingMeshAtlas {
        // One strong texture handle for each unique texture that will make up
        // the atlas.
        let mut texture_handles: HashSet<Handle<Image>> = Default::default();

        // Weak texture handles, one for each face in the mesh.
        let mut face_textures: Vec<Handle<Image>> = Vec::with_capacity(mesh.faces.len());

        // Cached mapping from block state id to weak texture handle.
        let mut handle_cache: HashMap<(BlockStateId, BlockFace), Handle<Image>> =
            Default::default();

        for face in mesh.faces.iter() {
            let [x, y, z] = face.voxel;

            let face = face.axis.into();

            let block_state_id = chunk_section.get_block((x, y, z)).unwrap();
            let block_state_id = BlockStateId(block_state_id.0 as u16);

            let key = (block_state_id, face);
            let weak_handle = match handle_cache.entry(key) {
                Entry::Vacant(entry) => {
                    let strong_handle =
                        match mc_assets.textures().get_texture_path(block_state_id, face) {
                            Some(path) => asset_server.load(path),
                            None => {
                                debug!("No texture for {:?}:{:?}", block_state_id, face);
                                texture_builder.placeholder_texture.clone()
                            }
                        };

                    if !texture_handles.contains(&strong_handle) {
                        texture_handles.insert(strong_handle.clone());
                    }

                    entry.insert(strong_handle.as_weak()).clone_weak()
                }
                Entry::Occupied(entry) => entry.get().clone_weak(),
            };

            face_textures.push(weak_handle);
        }

        // debug!("texture_handles: {:#?}", &texture_handles);
        // debug!("face_textures: {:#?}", &face_textures);
        // debug!("handle_cache: {:#?}", &handle_cache);

        let atlas = texture_builder
            .create_texture_atlas_with_textures(texture_handles.into_iter(), asset_server);

        PendingMeshAtlas {
            atlas,
            face_textures,
        }
    }

    fn add_built_chunk_to_world(
        chunk_data: brine_chunk::Chunk,
        voxel_meshes: Vec<VoxelMesh>,
        atlases: Vec<&TextureAtlas>,
        face_textures: Vec<Vec<Handle<Image>>>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        commands: &mut Commands,
    ) -> Entity {
        debug!(
            "Adding chunk ({}, {}) to world",
            chunk_data.chunk_x, chunk_data.chunk_z
        );
        commands
            .spawn()
            .insert_bundle(BuiltChunkBundle::new(
                T::TYPE,
                chunk_data.chunk_x,
                chunk_data.chunk_z,
            ))
            .with_children(move |parent| {
                for (((section, mut mesh), atlas), face_textures) in chunk_data
                    .sections
                    .into_iter()
                    .zip(voxel_meshes.into_iter())
                    .zip(atlases.into_iter())
                    .zip(face_textures.into_iter())
                {
                    // debug!("atlas has texture handles: {:#?}", &atlas.texture_handles);
                    // debug!("voxel mesh has face textures: {:#?}", &face_textures[..]);

                    mesh.adjust_tex_coords(atlas, &face_textures);

                    parent
                        .spawn()
                        .insert_bundle(BuiltChunkSectionBundle::new(T::TYPE, section.chunk_y))
                        .insert_bundle(PbrBundle {
                            mesh: meshes.add(mesh.to_render_mesh()),
                            material: materials.add(StandardMaterial {
                                base_color_texture: Some(atlas.texture.clone()),
                                unlit: true,
                                //alpha_mode: AlphaMode::Blend,
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .insert(ChunkSectionComponent(section));
                }
            })
            .id()
    }

    /*
      ____            _
     / ___| _   _ ___| |_ ___ _ __ ___  ___
     \___ \| | | / __| __/ _ \ '_ ` _ \/ __|
      ___) | |_| \__ \ ||  __/ | | | | \__ \
     |____/ \__, |___/\__\___|_| |_| |_|___/
            |___/
    */

    fn builder_task_spawn_unique(
        mut chunk_events: ResMut<Events<event::clientbound::ChunkData>>,
        mut commands: Commands,
        task_pool: Res<AsyncComputeTaskPool>,
    ) {
        for chunk_event in chunk_events.drain() {
            Self::builder_task_spawn(chunk_event, &mut commands, &task_pool);
        }
    }

    fn builder_task_spawn_shared(
        mut chunk_events: EventReader<event::clientbound::ChunkData>,
        mut commands: Commands,
        task_pool: Res<AsyncComputeTaskPool>,
    ) {
        for chunk_event in chunk_events.iter() {
            Self::builder_task_spawn(chunk_event.clone(), &mut commands, &task_pool);
        }
    }

    fn receive_built_meshes(
        asset_server: Res<AssetServer>,
        mc_assets: Res<MinecraftAssets>,
        mut chunks_with_pending_meshes: Query<(Entity, &mut PendingChunk, &mut MesherTask)>,
        mut texture_builder: ResMut<BlockTextures>,
        mut commands: Commands,
    ) {
        const MAX_PER_FRAME: usize = 1;

        for (i, (entity, mut pending_chunk, mut mesher_task)) in
            chunks_with_pending_meshes.iter_mut().enumerate()
        {
            if i >= MAX_PER_FRAME {
                break;
            }

            if pending_chunk.builder != T::TYPE {
                continue;
            }

            if let Some((chunk, voxel_meshes)) =
                future::block_on(future::poll_once(&mut *mesher_task))
            {
                debug!(
                    "Received meshes for Chunk ({}, {})",
                    chunk.chunk_x, chunk.chunk_z
                );

                let texture_atlases = voxel_meshes
                    .iter()
                    .zip(chunk.sections.iter())
                    .map(|(mesh, chunk_section)| {
                        Self::build_texture_atlas_for_mesh(
                            mesh,
                            chunk_section,
                            &*asset_server,
                            &*mc_assets,
                            &mut *texture_builder,
                        )
                    })
                    .collect();

                pending_chunk.chunk_data = Some(chunk);
                pending_chunk.voxel_meshes = Some(voxel_meshes);
                pending_chunk.texture_atlases = Some(texture_atlases);

                commands.entity(entity).remove::<MesherTask>();
            }
        }
    }

    fn add_built_chunks_to_world(
        atlases: Res<Assets<TextureAtlas>>,
        mut chunks_with_pending_atlases: Query<(Entity, &mut PendingChunk), Without<MesherTask>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut commands: Commands,
    ) {
        for (entity, mut pending_chunk) in chunks_with_pending_atlases.iter_mut() {
            if pending_chunk.builder != T::TYPE {
                continue;
            }

            let built_atlases: Vec<Option<&TextureAtlas>> = pending_chunk
                .texture_atlases
                .as_ref()
                .unwrap()
                .iter()
                .map(|pending_atlas| atlases.get(&pending_atlas.atlas))
                .collect();

            if built_atlases.iter().any(|atlas| atlas.is_none()) {
                continue;
            }

            let atlases: Vec<&TextureAtlas> =
                built_atlases.iter().map(|atlas| atlas.unwrap()).collect();

            let face_textures: Vec<Vec<Handle<Image>>> = pending_chunk
                .texture_atlases
                .take()
                .unwrap()
                .into_iter()
                .map(|atlas| atlas.face_textures)
                .collect();

            let chunk = pending_chunk.chunk_data.take().unwrap();
            let voxel_meshes = pending_chunk.voxel_meshes.take().unwrap();

            debug!(
                "Received all texture atlases for Chunk ({}, {})",
                chunk.chunk_x, chunk.chunk_z
            );

            Self::add_built_chunk_to_world(
                chunk,
                voxel_meshes,
                atlases,
                face_textures,
                &mut *meshes,
                &mut *materials,
                &mut commands,
            );

            commands.entity(entity).despawn();
        }
    }
}
