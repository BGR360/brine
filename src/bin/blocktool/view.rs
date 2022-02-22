use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::Rect,
};
use bevy_inspector_egui::WorldInspectorPlugin;

use brine::debug::DebugWireframePlugin;
use brine_asset::{
    bakery_v2::models::{BakedModel, BakedQuad},
    MinecraftAssets,
};
use brine_data::{BlockStateId, MinecraftData};
use brine_render::texture::{
    MinecraftTexturesPlugin, MinecraftTexturesState, TextureAtlas, TextureManager,
    TextureManagerPlugin,
};

/// Displays a block.
#[derive(clap::Args)]
pub struct Args {
    /// Block state id.
    #[clap(short, long)]
    state_id: u16,
}

pub(crate) fn main(args: Args) {
    display_block(BlockStateId(args.state_id));
}

fn display_block(block_state_id: BlockStateId) {
    let mc_data = MinecraftData::for_version("1.14.4");
    let mc_assets = MinecraftAssets::new("assets/1.14.4", &mc_data).unwrap();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugWireframePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(mc_data)
        .insert_resource(mc_assets)
        .add_plugin(TextureManagerPlugin)
        .add_plugin(MinecraftTexturesPlugin)
        .insert_resource(TheBlock { block_state_id })
        .add_system_set(SystemSet::on_enter(MinecraftTexturesState::Loaded).with_system(setup))
        .run();
}

#[derive(Debug)]
struct TheBlock {
    block_state_id: BlockStateId,
}

fn setup(
    the_block: Res<TheBlock>,
    mc_assets: Res<MinecraftAssets>,
    texture_manager: Res<TextureManager>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(3.0, 3.0, 3.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let cube = Mesh::from(shape::Cube::new(1.0));
    // debug!("{:#?}", cube);

    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(cube),
    //     material: materials.add(StandardMaterial::default()),
    //     ..Default::default()
    // });

    let baked_block_state = mc_assets
        .baked_assets()
        .block_states
        .get_by_key(the_block.block_state_id)
        .unwrap();

    for grab_bag in baked_block_state.models.iter() {
        let model_key = grab_bag.choices.first().unwrap();
        let baked_model = mc_assets
            .baked_assets()
            .models
            .get_by_key(*model_key)
            .unwrap();

        let texture_key = baked_model.quads.first().unwrap().texture;
        let atlas_handle = texture_manager.get_atlas(texture_key).unwrap();
        let atlas = texture_atlases.get(&atlas_handle).unwrap();

        let mesh = baked_model_to_mesh(baked_model, atlas);

        debug!("{:#?}", mesh);

        let material = StandardMaterial {
            //base_color_texture: Some(atlas.texture.clone()),
            ..Default::default()
        };

        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(material),
            ..Default::default()
        });
    }
}

fn baked_model_to_mesh(baked_model: &BakedModel, texture_atlas: &TextureAtlas) -> Mesh {
    let num_quads = baked_model.quads.len();
    let num_vertices = num_quads * 4;
    let num_indices = num_quads * 6;

    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    let mut indices = Vec::with_capacity(num_indices);

    for quad in baked_model.quads.iter() {
        positions.extend_from_slice(&quad.positions);
        normals.extend_from_slice(&[quad.normal; 4]);

        let uvs_within_atlas = texture_atlas.get_uv(quad.texture);
        tex_coords.extend_from_slice(&adjust_tex_coords(quad.tex_coords, uvs_within_atlas));

        indices
            .extend_from_slice(&BakedQuad::INDICES.map(|index| (positions.len() + index) as u16));
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.set_indices(Some(Indices::U16(indices)));

    mesh
}

fn adjust_tex_coords(tex_coords: [f32; 4], uvs_within_atlas: Rect) -> [[f32; 2]; 4] {
    let [u0, v0, u1, v1] = tex_coords;

    [[u0, v0], [u1, v0], [u0, v1], [u1, v1]]
}
