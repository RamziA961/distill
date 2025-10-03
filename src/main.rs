use crate::{
    camera::{
        configuration::CameraConfiguration, marker::CameraMarkerPrimary, plugin::CameraPlugin,
    },
    gpu_types::{GpuBox3, GpuCamera, GpuUVec3, GpuVec2, GpuVec3},
    voxelization::{
        VoxelizationPlugin, VoxelizeMarker,
        raymarch_material::RaymarchMaterial,
        voxelization_worker::{SIZE, VoxelVariables, VoxelizationWorker},
    },
};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{mesh::MeshAabb, storage::ShaderStorageBuffer},
};
use bevy_app_compute::prelude::AppComputeWorker;

mod camera;
pub(crate) mod gpu_types;
pub(crate) mod voxelization;
mod window;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(CameraPlugin::<CameraMarkerPrimary> {
        configuration: CameraConfiguration::<CameraMarkerPrimary>::default(),
    });
    app.add_plugins(VoxelizationPlugin);

    app.add_systems(Startup, (camera_system, light_system));

    // window and cursor controls
    app.add_systems(Startup, (window::grab_cursor, window::hide_cursor));
    app.add_systems(Update, window::toggle_cursor);

    app.add_systems(Startup, (sphere, test).chain());
    app.add_systems(
        Update,
        (extract_sdf, spawn_sdf_test, update_raymarch_material).chain(),
    );

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
        VoxelizeMarker,
        Mesh3d(meshes.add(
            Sphere::new(1.0),
            //Cuboid::new(2.0, 2.0, 2.0),
        )),
        MeshMaterial3d(
            materials.add(StandardMaterial::from_color(Color::linear_rgba(
                1.0, 0.0, 0.0, 0.0,
            ))),
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn test(
    mesh_handle: Single<&Mesh3d, With<VoxelizeMarker>>,
    meshes: Res<Assets<Mesh>>,
    mut worker: ResMut<AppComputeWorker<VoxelizationWorker>>,
) {
    info!("Uploading mesh to GPU.");

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

    info!(n_verts = verts.len(), n_tris = triangles.len());

    worker.write_slice("vertices", &verts);
    worker.write_slice("triangles", &triangles);
}

fn extract_sdf(
    mut commands: Commands,
    mut shader_storage: ResMut<Assets<ShaderStorageBuffer>>,
    worker: Res<AppComputeWorker<VoxelizationWorker>>,
) {
    if !worker.ready() {
        warn!("Worker is not ready!");
        return;
    }

    if !worker.is_changed() {
        trace!("Worker has not changed. Skipping read.");
        return;
    }

    let buff = worker
        .read_raw(VoxelVariables::VoxelTexture.as_ref())
        .to_vec();

    let handle = shader_storage.add(ShaderStorageBuffer::new(
        &buff,
        RenderAssetUsages::RENDER_WORLD,
    ));

    commands.insert_resource(SdfBufferHandle(handle));
}

#[derive(Resource, Clone)]
struct SdfBufferHandle(pub Handle<ShaderStorageBuffer>);

#[derive(Component, Clone)]
struct RenderTargetSingle;

#[allow(clippy::too_many_arguments)]
fn spawn_sdf_test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<RaymarchMaterial>>,
    voxel_mesh_query: Query<&Mesh3d, With<VoxelizeMarker>>,
    camera_params: Single<(&Transform, &Projection), With<CameraMarkerPrimary>>,
    render_target_query: Query<(), With<RenderTargetSingle>>,
    sdf_buff_handle: Option<Res<SdfBufferHandle>>,
    window: Query<&Window>,
) {
    if sdf_buff_handle.is_none() {
        return;
    }

    if render_target_query.iter().count() == 1 {
        return;
    }

    let screen_resolution = window
        .single()
        .map(|w| GpuVec2::new(w.width(), w.height()))
        .unwrap_or(GpuVec2::new(800.0, 600.0));

    let (transform, projection) = camera_params.into_inner();

    let camera = GpuCamera::from_transform_and_projection(transform, projection);

    let mesh = meshes.get(voxel_mesh_query.iter().next().unwrap()).unwrap();
    let grid_bounds = mesh.compute_aabb().map(GpuBox3::from).unwrap();
    let grid_size = SIZE;

    info!(grid_bounds=?grid_bounds, camera=?camera, screen_resolution=?screen_resolution);

    //let bbox = Aabb::from(grid_bounds);
    commands.spawn((
        RenderTargetSingle,
        Mesh3d(meshes.add(
            Cuboid::from_size(grid_bounds.size().into()),
            /*Rectangle::from_corners(bbox.min().xy(), bbox.max().xy()),*/
        )),
        MeshMaterial3d(materials.add(RaymarchMaterial {
            voxel_texture: sdf_buff_handle.unwrap().0.clone(),
            camera,
            grid_bounds,
            grid_size,
        })),
        Transform::from_translation(grid_bounds.center().into()),
    ));
}

fn update_raymarch_material(
    mut material: ResMut<Assets<RaymarchMaterial>>,
    material_handles: Query<&MeshMaterial3d<RaymarchMaterial>>,
    camera_params: Single<(&Transform, &Projection), With<CameraMarkerPrimary>>,
) {
    let (transform, projection) = camera_params.into_inner();
    let camera = GpuCamera::from_transform_and_projection(transform, projection);
    for handle in material_handles {
        let mat = material.get_mut(handle).unwrap();
        mat.camera = camera;
    }
}
