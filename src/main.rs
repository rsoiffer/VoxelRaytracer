use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{AssetRenderResourcesNode, base, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};
use bevy::core::Byteable;
use bevy::render::texture::{Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat};

mod vox;

/// This example illustrates how to create a custom material asset and a shader that uses that material
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_asset::<MyMaterial>()
        .add_startup_system(setup)
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
    #[render_resources(buffer)]
    pub palette: Vec<VoxelMaterial>,
    pub camera_object_pos: Vec3,
    pub object_size: Vec3,
}

const VERTEX_SHADER: &str = include_str!("voxel.vert");
const FRAGMENT_SHADER: &str = include_str!("voxel.frag");

fn setup(
    commands: &mut Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
    mut render_graph: ResMut<RenderGraph>,
) {
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
            palette: palette,
            camera_object_pos: Vec3::new(50.0, 80.0, -200.0),
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

    // Setup our world
    commands
        // camera
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(50.0, 80.0, -200.0))
                .looking_at(Vec3::new(2.0, 100.0, 2.0), Vec3::unit_y()),
            ..Default::default()
        });
}
