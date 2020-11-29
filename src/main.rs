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
use bevy::render::texture::{Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat};
use bevy::core::Byteable;

mod vox;

/// This example illustrates how to create a custom material asset and a shader that uses that material
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_asset::<MyMaterial>()
        .add_startup_system(setup)
        .run();
}

#[derive(Default, Clone, Copy, Debug)]
struct VoxelMaterial {
    pub albedo: Vec3,
}
unsafe impl Byteable for VoxelMaterial {}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
struct MyMaterial {
    pub color: Color,
    pub texture: Handle<Texture>,
    #[render_resources(buffer)]
    pub palette: Vec<VoxelMaterial>,
}

const VERTEX_SHADER: &str = r#"
#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_Uv;

layout(location = 0) out vec3 v_Position;
layout(location = 1) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_Position = gl_Position.xyz;
    v_Uv = Vertex_Uv;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 450


layout(location = 0) in vec3 v_Position;
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

void main() {
    uint x = textureLod(
        usampler3D(MyMaterial_texture, MyMaterial_texture_sampler),
        vec3(v_Uv, 0), 0).r;
    o_Target = vec4(Palette[x].albedo, 1);
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
        tex_3d_data.push(i);
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

    let palette = vec![VoxelMaterial { albedo: Vec3::new(1.0, 0.0, 1.0)}; 255];

    // Create a new material
    let material = materials.add(MyMaterial {
        color: Color::rgb(0.0, 0.8, 0.0),
        texture: texture_handle,
        palette: palette,
    });

    // Setup our world
    commands
        // cube
        .spawn(MeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(material)
        // camera
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(3.0, 5.0, -8.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}

// use bevy::prelude::*;
//
// fn main() {
//     App::build()
//         .add_plugins(DefaultPlugins)
//         .add_startup_system(setup)
//         .add_system(animate_sprite_system)
//         .run();
// }
//
// fn animate_sprite_system(
//     time: Res<Time>,
//     texture_atlases: Res<Assets<TextureAtlas>>,
//     mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
// ) {
//     for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
//         timer.tick(time.delta_seconds());
//         if timer.finished() {
//             let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
//             sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
//         }
//     }
// }
//
// fn setup(
//     commands: &mut Commands,
//     asset_server: Res<AssetServer>,
//     mut texture_atlases: ResMut<Assets<TextureAtlas>>,
// ) {
//     let texture_handle = asset_server.load("textures/rpg/chars/gabe/gabe-idle-run.png");
//     let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
//     let texture_atlas_handle = texture_atlases.add(texture_atlas);
//     commands
//         .spawn(Camera2dBundle::default())
//         .spawn(SpriteSheetBundle {
//             texture_atlas: texture_atlas_handle,
//             transform: Transform::from_scale(Vec3::splat(6.0)),
//             ..Default::default()
//         })
//         .with(Timer::from_seconds(0.1, true));
// }
