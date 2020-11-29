use bevy::{
    core::Byteable,
    input::{mouse::MouseMotion, system::exit_on_esc_system},
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::Camera,
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{AssetRenderResourcesNode, base, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
        texture::{Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat},
    },
};
use bevy::render::camera::PerspectiveProjection;

mod vox;

/// This example illustrates how to create a custom material asset and a shader that uses that material
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_asset::<MyMaterial>()
        .add_startup_system(setup)
        .add_system(exit_on_esc_system)
        .add_system(mouse_camera)
        .run();
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
struct VoxelMaterial {
    pub albedo: Vec3,
    pub roughness: f32,
}

impl VoxelMaterial {
    fn of_mat(mat: vox::Material) -> VoxelMaterial {
        return VoxelMaterial {
            albedo: Vec3::new(mat.color.r(), mat.color.g(), mat.color.b()),
            roughness: 1.0,
        };
    }
}

unsafe impl Byteable for VoxelMaterial {}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
struct MyMaterial {
    pub texture: Handle<Texture>,
    pub blue_noise: Handle<Texture>,
    #[render_resources(buffer)]
    pub palette: Vec<VoxelMaterial>,
    pub camera_object_pos: Vec3,
    pub object_size: Vec3,
}

const VERTEX_SHADER: &str = include_str!("voxel.vert");
const FRAGMENT_SHADER: &str = include_str!("voxel.frag");

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
    mut render_graph: ResMut<RenderGraph>,
    mut windows: ResMut<Windows>,
) {
    for window in windows.iter_mut() {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }

    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to our shader
    render_graph.add_system_node(
        "my_material",
        AssetRenderResourcesNode::<MyMaterial>::new(false),
    );

    // Add a Render Graph edge connecting our new "my_material" node to the main pass node. This ensures "my_material" runs before the main pass
    render_graph
        .add_node_edge("my_material", base::node::MAIN_PASS)
        .unwrap();

    let blue_noise_handle = asset_server.load("LDR_RGBA_0.png");

    for model in vox::load("assets/monu6.vox").unwrap() {
        let (width, height, depth) = model.voxels.dim();

        let tex_3d = Texture {
            data: model.voxels_vec(),
            size: Extent3d {
                width: width as u32,
                height: height as u32,
                depth: depth as u32,
            },
            format: TextureFormat::R8Uint,
            dimension: TextureDimension::D3,
            sampler: SamplerDescriptor {
                min_filter: FilterMode::Nearest,
                ..SamplerDescriptor::default()
            },
        };
        let texture_handle = textures.add(tex_3d);

        let mut palette = Vec::new();
        for i in 0..256 {
            palette.push(VoxelMaterial::of_mat(model.palette[i]));
        }

        // Create a new material
        let material = materials.add(MyMaterial {
            texture: texture_handle,
            blue_noise: blue_noise_handle.clone(),
            palette: palette,
            camera_object_pos: Vec3::new(50.0, 100.0, -40.0),
            object_size: Vec3::new(width as f32, height as f32, depth as f32),
        });

        let shape = shape::Box {
            min_x: 0.0,
            max_x: width as f32,
            min_y: 0.0,
            max_y: height as f32,
            min_z: 0.0,
            max_z: depth as f32,
        };

        commands
            .spawn(MeshBundle {
                mesh: meshes.add(Mesh::from(shape)),
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    pipeline_handle.clone(),
                )]),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                ..Default::default()
            })
            .with(material);
    }

    commands.spawn(Camera3dBundle {
        perspective_projection: PerspectiveProjection {
            fov: 70.0 / 180.0 * std::f32::consts::PI,
            near: 1.0,
            far: 10000.0,
            aspect_ratio: 1.0,
        },
        transform: Transform::from_translation(Vec3::new(50.0, 100.0, -40.0)),
        ..Default::default()
    });
}

struct MouseCameraState {
    reader: EventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

impl Default for MouseCameraState {
    fn default() -> Self {
        MouseCameraState {
            reader: Default::default(),
            pitch: 0.0,
            yaw: 180.0,
        }
    }
}

const MOUSE_SENSITIVITY: f32 = 15.0;

fn mouse_camera(
    mut state: Local<MouseCameraState>,
    time: Res<Time>,
    events: Res<Events<MouseMotion>>,
    mut query: Query<(&Camera, &mut Transform)>,
) {
    let delta = state
        .reader
        .iter(&events)
        .fold(Vec2::zero(), |acc, event| acc + event.delta);
    state.yaw -= delta.x * time.delta_seconds() * MOUSE_SENSITIVITY;
    state.pitch = (state.pitch + delta.y * time.delta_seconds() * MOUSE_SENSITIVITY)
        .min(90.0)
        .max(-90.0);

    for (_, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_axis_angle(Vec3::unit_y(), state.yaw.to_radians())
            * Quat::from_axis_angle(-Vec3::unit_x(), state.pitch.to_radians());
    }
}
