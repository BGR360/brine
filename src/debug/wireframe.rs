use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};

pub struct DebugWireframePlugin;

impl Plugin for DebugWireframePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .add_plugin(WireframePlugin)
        .register_type::<EnableWireframe>()
        .add_startup_system(spawn_component)
        .add_system(update_wireframe_config);
    }
}

#[derive(Component, Reflect, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub struct EnableWireframe {
    pub enable: bool,
}

fn spawn_component(mut commands: Commands) {
    commands.spawn().insert_bundle((
        Name::new("Debug Wireframe"),
        EnableWireframe { enable: true },
    ));
}

fn update_wireframe_config(
    component: Query<&EnableWireframe>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    let component = component.single();
    wireframe_config.global = component.enable;
}
