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

unsafe impl Byteable for VoxelMaterial {}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
struct MyMaterial {
    pub color: Color,
    pub texture: Handle<Texture>,
    #[render_resources(buffer)]
    pub palette: Vec<VoxelMaterial>,
    pub cameraObjectPos: Vec3,
    pub objectSize: Vec3,
}

const VERTEX_SHADER: &str = r#"
#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_Uv;

layout(location = 0) out vec3 v_ObjectPos;
layout(location = 1) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_ObjectPos = Vertex_Position;
    v_Uv = Vertex_Uv;
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 450


layout(location = 0) in vec3 v_ObjectPos;
layout(location = 1) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 1) uniform MyMaterial_color {
    vec4 color;
};

layout(set = 1, binding = 2) uniform utexture3D MyMaterial_texture;
layout(set = 1, binding = 3) uniform sampler MyMaterial_texture_sampler;

struct VoxelMaterial {
    vec3 albedo;
};
layout(set = 1, binding = 4) buffer MyMaterial_palette {
    VoxelMaterial[] Palette;
};

layout(set = 1, binding = 5) uniform MyMaterial_cameraObjectPos {
    vec3 cameraObjectPos;
};

layout(set = 1, binding = 6) uniform MyMaterial_objectSize {
    vec3 objectSize;
};

void main() {
    vec3 rayDir = normalize(v_ObjectPos - cameraObjectPos);
    vec3 floatPos = v_ObjectPos + rayDir * .00001;
    vec3 pos = floor(floatPos);
    vec3 step = sign(rayDir);
    // if (step.x == 1 then 1/rayDir.x - floatPos.x/rayDir.x else -floatPos.x / rayDir.x
    vec3 tmax = (1 + step) / (2 * rayDir) - (floatPos - pos) / rayDir;
    vec3 tdelta = 1 / abs(rayDir);

    uint vox = textureLod(
        usampler3D(MyMaterial_texture, MyMaterial_texture_sampler),
        pos / objectSize, 0).r;
    while (vox == 0) {
        if (tmax.x < tmax.y) {
            if (tmax.x < tmax.z) {
                pos.x += step.x;
                tmax.x += tdelta.x;
            } else {
                pos.z += step.z;
                tmax.z += tdelta.z;
            }
        } else {
            if (tmax.y < tmax.z) {
                pos.y += step.y;
                tmax.y += tdelta.y;
            } else {
                pos.z += step.z;
                tmax.z += tdelta.z;
            }
        }
        if (any(lessThan(pos, vec3(0,0,0))) || any(greaterThan(pos, objectSize-1))) {
            break;
        }
        vox = textureLod(
            usampler3D(MyMaterial_texture, MyMaterial_texture_sampler),
            pos / objectSize, 0).r;
    }

    if (vox == 0) {
        discard;
    } else {
        o_Target = vec4(Palette[vox - 1].albedo, 1);
    }
}
"#;

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


    let mut tex_3d_data = Vec::new();
    for i in 0..64 {
        if i % 3 != 0 {
            tex_3d_data.push(0);
        } else {
            tex_3d_data.push(i * 3 + 5);
        }
    }
    let tex_3d = Texture {
        data: tex_3d_data,
        size: Extent3d {
            width: 4,
            height: 4,
            depth: 4,
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
    for i in 0..255 {
        palette.push(VoxelMaterial {
            albedo: Vec3::new(i as f32 / 255.0, 0.0, 1.0),
            roughness: 1.0,
        })
    }

    // Create a new material
    let material = materials.add(MyMaterial {
        color: Color::rgb(0.0, 0.8, 0.0),
        texture: texture_handle,
        palette: palette,
        cameraObjectPos: Vec3::new(5.0, 8.0, -10.0),
        objectSize: Vec3::new(4.0, 4.0, 4.0),
    });

    let shape = shape::Box {
        min_x: 0.0,
        max_x: 4.0,
        min_y: 0.0,
        max_y: 4.0,
        min_z: 0.0,
        max_z: 4.0,
    };

    // Setup our world
    commands
        // cube
        .spawn(MeshBundle {
            mesh: meshes.add(Mesh::from(shape)),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(material)
        // camera
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(5.0, 8.0, -10.0))
                .looking_at(Vec3::new(2.0, 2.0, 2.0), Vec3::unit_y()),
            ..Default::default()
        });
}
