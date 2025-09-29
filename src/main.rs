use crate::{
    camera::{
        configuration::CameraConfiguration, marker::CameraMarkerPrimary, plugin::CameraPlugin,
    },
    gpu_types::{GpuUVec3, GpuVec3},
    voxelization::{VoxelizationPlugin, voxelization_worker::VoxelizationWorker},
};
use bevy::prelude::*;
use bevy_app_compute::prelude::AppComputeWorker;

mod camera;
pub(crate) mod gpu_types;
pub(crate) mod voxelization;

static mut RUN: bool = false;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(CameraPlugin::<CameraMarkerPrimary> {
        configuration: CameraConfiguration::<CameraMarkerPrimary>::default(),
    });

    app.add_plugins(VoxelizationPlugin);

    app.add_systems(Startup, (camera_system, light_system));
    app.add_systems(Startup, sphere);
    app.add_systems(Update, (test).run_if(|| unsafe { !RUN }));

    app.run();
}

fn camera_system(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        CameraMarkerPrimary,
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
            scale: Vec3::ONE,
        },
    ));
}

fn light_system(mut commands: Commands) {
    commands.spawn((PointLight::default(), Transform::from_xyz(0.0, 10.0, 0.0)));
}

fn sphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(
            Sphere::new(1.0),
            //Cuboid::new(2.0, 2.0, 2.0),
        )),
        MeshMaterial3d(
            materials.add(StandardMaterial::from_color(Color::linear_rgba(
                1.0, 0.0, 0.0, 1.0,
            ))),
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn test(
    mesh_handle: Single<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    mut worker: ResMut<AppComputeWorker<VoxelizationWorker>>,
) {
    if !worker.ready() {
        return;
    }

    info!("Uploading mesh to GPU.");
    unsafe { RUN = true };

    let mesh = meshes.get(mesh_handle.0.id()).unwrap();

    let verts = match mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("Mesh has no positions")
    {
        bevy::render::mesh::VertexAttributeValues::Float32x3(verts) => verts
            .clone()
            .iter()
            .map(|s| GpuVec3::from_slice(s))
            .collect::<Vec<_>>(),
        _ => panic!("Unexpected format!"),
    };

    let triangles: Vec<_> = match mesh.indices().unwrap() {
        bevy::render::mesh::Indices::U32(indices) => {
            indices.chunks_exact(3).map(GpuUVec3::from_slice).collect()
        }
        bevy::render::mesh::Indices::U16(indices) => indices
            .chunks_exact(3)
            .map(|tri| GpuUVec3::new(tri[0] as u32, tri[1] as u32, tri[2] as u32))
            .collect(),
    };

    info!(n_verts = verts.len(), n_tris = triangles.len(),);

    worker.write_slice("vertices", &verts);
    worker.write_slice("triangles", &triangles);
}
