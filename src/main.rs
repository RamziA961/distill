use crate::{
    bvh::{BvhData, BvhPlugin},
    camera::{
        configuration::CameraConfiguration, marker::CameraMarkerPrimary, plugin::CameraPlugin,
    },
    gpu_types::{GpuBox3, GpuCamera},
    voxelization::{
        VoxelizationPlugin, VoxelizeMarker, VoxelizedMarker,
        raymarch_material::RaymarchMaterial,
        voxelization_worker::{SIZE, VoxelVariables, VoxelizationWorker},
    },
};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        mesh::MeshAabb,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_app_compute::prelude::AppComputeWorker;

#[cfg(feature = "distill-dev")]
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
#[cfg(feature = "distill-dev")]
use iyes_perf_ui::{PerfUiPlugin, prelude::PerfUiDefaultEntries};

pub(crate) mod bvh;
mod camera;
pub(crate) mod gpu_types;
pub(crate) mod voxelization;
mod window;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));

    #[cfg(feature = "distill-dev")]
    app.add_plugins((FrameTimeDiagnosticsPlugin::default(), PerfUiPlugin));
    #[cfg(feature = "distill-dev")]
    app.add_systems(Startup, debug_tools);

    app.add_plugins(CameraPlugin::<CameraMarkerPrimary> {
        configuration: CameraConfiguration::<CameraMarkerPrimary>::default(),
    });
    app.add_plugins((BvhPlugin, VoxelizationPlugin));

    app.add_systems(Startup, (camera_system, light_system));

    // window and cursor controls
    app.add_systems(Startup, (window::grab_cursor, window::hide_cursor));
    app.add_systems(Update, window::toggle_cursor);
    app.add_systems(Startup, spawn_target_mesh);

    app.add_systems(
        Update,
        (
            upload_to_gpu,
            extract_sdf,
            spawn_sdf_test,
            update_raymarch_material,
        )
            .chain(),
    );

    app.run();
}

fn camera_system(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        CameraMarkerPrimary,
        Transform::from_xyz(-5.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn light_system(mut commands: Commands) {
    commands.spawn((PointLight::default(), Transform::from_xyz(0.0, 10.0, 0.0)));
}

#[cfg(feature = "distill-dev")]
fn debug_tools(mut commands: Commands) {
    commands.spawn(PerfUiDefaultEntries::default());
}

fn spawn_target_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        VoxelizeMarker,
        Mesh3d(meshes.add(
            Sphere::new(1.0),
            //Cuboid::new(2.0, 2.0, 2.0),
            //Torus::new(0.5, 1.0),
            //Cone::new(1.0, 3.0),
            //Tetrahedron::default(),
            //Capsule3d::default(),
        )),
        MeshMaterial3d(
            materials.add(StandardMaterial::from_color(Color::linear_rgba(
                1.0, 0.0, 0.0, 1.0,
            ))),
        ),
        Transform::from_xyz(-3.0, 0.0, 0.0),
    ));
}

#[allow(clippy::type_complexity)]
fn upload_to_gpu(
    mut commands: Commands,
    mesh_data: Query<(Entity, &BvhData), (With<VoxelizeMarker>, Without<VoxelizedMarker>)>,
    mut worker: ResMut<AppComputeWorker<VoxelizationWorker>>,
) {
    if mesh_data.iter().count() == 0 {
        return;
    }

    info!("Uploading mesh to GPU.");
    let (entity, bvh_data) = mesh_data.iter().next().unwrap();

    info!(
        n_triangles = bvh_data.triangles.len(),
        n_bvh_nodes = bvh_data.nodes.len(),
    );

    info!(bvh_root = ?bvh_data.nodes[0]);

    worker.write_slice(VoxelVariables::Triangles.as_ref(), &bvh_data.triangles);
    worker.write_slice(VoxelVariables::BvhNodes.as_ref(), &bvh_data.nodes);
    commands.entity(entity).insert(VoxelizedMarker);
    info!("Mesh uploaded to GPU.");
}

fn extract_sdf(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
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

    let grid_size = SIZE;
    let extent = Extent3d {
        width: grid_size,
        height: grid_size,
        depth_or_array_layers: grid_size,
    };

    // Convert to GPU 3D texture
    let mut image = Image::new(
        extent,
        TextureDimension::D3,
        bytemuck::cast_slice(&buff).to_vec(),
        TextureFormat::R32Float, // one channel, 32-bit float
        RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });

    let handle = images.add(image);

    commands.insert_resource(SdfBufferHandle(handle));
}

#[derive(Resource, Clone)]
struct SdfBufferHandle(pub Handle<Image>);

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
) {
    if sdf_buff_handle.is_none() {
        return;
    }

    if render_target_query.iter().count() == 1 {
        return;
    }

    let (transform, projection) = camera_params.into_inner();

    let camera = GpuCamera::from_transform_and_projection(transform, projection);

    let mesh = meshes.get(voxel_mesh_query.iter().next().unwrap()).unwrap();
    let grid_bounds = mesh.compute_aabb().map(GpuBox3::from).unwrap();
    let grid_size = SIZE;

    commands.spawn((
        RenderTargetSingle,
        Mesh3d(meshes.add(Cuboid::from_size(grid_bounds.size().into()))),
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
