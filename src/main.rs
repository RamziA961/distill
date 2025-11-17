use crate::{
    bvh::{BvhPlugin, BvhTargetMarker},
    camera::{
        configuration::CameraConfiguration, marker::CameraMarkerPrimary, plugin::CameraPlugin,
    },
    voxelizer::{VoxelizationPlugin, VoxelizeTargetMarker},
};
use bevy::{pbr::wireframe::Wireframe, prelude::*};
use bevy_obj::ObjPlugin;

pub(crate) mod bvh;
mod camera;
pub(crate) mod gpu_types;
pub(crate) mod utils;
pub(crate) mod voxelizer;
mod window;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, ObjPlugin));

    #[cfg(feature = "distill-dev")]
    {
        use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
        use bevy_dev_tools::{DevToolsPlugin, fps_overlay::FpsOverlayPlugin};
        app.add_plugins((
            WireframePlugin::default(),
            DevToolsPlugin,
            FpsOverlayPlugin::default(),
        ));

        app.insert_resource(WireframeConfig {
            global: false,
            default_color: Color::BLACK,
        });

        app.add_systems(Update, debug_gyzmos);
    }

    app.add_plugins(CameraPlugin::<CameraMarkerPrimary> {
        configuration: CameraConfiguration::<CameraMarkerPrimary>::default(),
    });
    app.add_plugins((BvhPlugin, VoxelizationPlugin));

    app.add_systems(Startup, (camera_system, light_system));

    // window and cursor controls
    app.add_systems(Startup, (window::grab_cursor, window::hide_cursor));
    app.add_systems(Update, window::toggle_cursor);

    app.add_systems(PostStartup, spawn_target_mesh_obj);

    app.run();
}

fn camera_system(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        CameraMarkerPrimary,
        Transform::from_xyz(-10.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn light_system(mut commands: Commands) {
    commands.spawn((PointLight::default(), Transform::from_xyz(0.0, 10.0, 0.0)));
}

fn debug_gyzmos(mut gizmos: Gizmos) {
    gizmos.axes(Transform::IDENTITY, 3.0);
}

fn spawn_target_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        VoxelizeTargetMarker,
        BvhTargetMarker,
        Mesh3d(meshes.add(
            Sphere::new(1.0),
            //Cuboid::new(2.0, 2.0, 2.0),
            //Torus::new(0.5, 1.0),
            //Cone::new(1.0, 3.0),
            //Tetrahedron::default(),
            //Capsule3d::default(),
        )),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgba(1.0, 0.0, 0.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(-3.0, 0.0, 0.0),
    ));
}

fn spawn_target_mesh_obj(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mesh_handle = asset_server.load::<Mesh>("models/cow.obj");

    commands.spawn((
        VoxelizeTargetMarker,
        BvhTargetMarker,
        Wireframe,
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgba(1.0, 0.0, 0.0, 1.0),
            ..default()
        })),
        Transform::from_matrix(Mat4::from_scale_rotation_translation(
            Vec3::splat(1.0),
            Quat::IDENTITY,
            Vec3::new(0.0, 0.0, -3.0),
        )),
    ));
}
