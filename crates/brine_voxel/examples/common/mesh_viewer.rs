use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        mesh::Indices,
        options::WgpuOptions,
        render_resource::{PrimitiveTopology, WgpuFeatures},
    },
};

use brine_voxel::Mesh as VoxelMesh;

use super::CHUNK_SIDE;

#[derive(Component)]
struct Root;

pub struct MeshViewerPlugin {
    mesh: VoxelMesh,
}

impl MeshViewerPlugin {
    pub fn new(mesh: VoxelMesh) -> Self {
        Self { mesh }
    }
}

impl Plugin for MeshViewerPlugin {
    fn build(&self, app: &mut App) {
        let mesh = build_bevy_mesh(&self.mesh);

        let mut meshes = app.world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let handle = meshes.add(mesh);

        app.world.insert_resource(handle);

        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(WgpuOptions {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..Default::default()
            })
            .insert_resource(WireframeConfig { global: true })
            .add_plugin(WireframePlugin)
            .add_startup_system(setup)
            .add_system(rotate);
    }
}

fn setup(
    mesh: Res<Handle<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let offset = CHUNK_SIDE as f32 / 2.0;

    commands
        .spawn_bundle((Transform::default(), GlobalTransform::default(), Root))
        .with_children(|parent| {
            parent.spawn().insert_bundle(PbrBundle {
                transform: Transform::from_translation(Vec3::new(-offset, -offset, -offset)),
                mesh: mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load("placeholder.png")),
                    unlit: true,
                    ..Default::default()
                }),
                ..Default::default()
            });
        });

    // let mut camera = OrthographicCameraBundle::new_3d();
    // camera.transform =
    //     Transform::from_translation(Vec3::new(5.0, 5.0, 5.0)).looking_at(Vec3::ZERO, Vec3::Y);
    // camera.orthographic_projection.scale = 5.0;

    let camera = PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(5.0, 5.0, 5.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    };

    commands.spawn_bundle(camera);
}

fn rotate(input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Root>>) {
    if let Ok(mut transform) = query.get_single_mut() {
        if input.just_pressed(KeyCode::Right) {
            transform.rotate(Quat::from_rotation_y(90.0_f32.to_radians()));
        }
        if input.just_pressed(KeyCode::Left) {
            transform.rotate(Quat::from_rotation_y(-90.0_f32.to_radians()));
        }
    }
}

pub fn build_bevy_mesh(voxel_mesh: &VoxelMesh) -> Mesh {
    let num_vertices = voxel_mesh.quads.len() * 4;
    let num_indices = voxel_mesh.quads.len() * 6;
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    let mut indices = Vec::with_capacity(num_indices);

    for quad in voxel_mesh.quads.iter() {
        indices.extend_from_slice(
            &quad
                .get_indices()
                .map(|i| positions.len() as u32 + i as u32),
        );

        positions.extend_from_slice(&quad.positions);
        normals.extend_from_slice(&quad.get_normals());
        tex_coords.extend_from_slice(&quad.get_tex_coords());
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}
