#version 450

layout(location = 0) in vec3 v_ObjectPos;
layout(location = 1) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 2) uniform utexture3D MyMaterial_texture;
layout(set = 1, binding = 3) uniform sampler MyMaterial_texture_sampler;

struct VoxelMaterial {
    vec3 albedo;
};
layout(set = 1, binding = 4) buffer MyMaterial_palette {
    VoxelMaterial[] Palette;
};

layout(set = 1, binding = 5) uniform MyMaterial_camera_object_pos {
    vec3 camera_object_pos;
};

layout(set = 1, binding = 6) uniform MyMaterial_object_size {
    vec3 object_size;
};

void main() {
    vec3 rayDir = normalize(v_ObjectPos - camera_object_pos);
    vec3 floatPos = v_ObjectPos + rayDir * .00001;
    vec3 pos = floor(floatPos);
    vec3 step = sign(rayDir);
    // if (step.x == 1 then 1/rayDir.x - floatPos.x/rayDir.x else -floatPos.x / rayDir.x
    vec3 tmax = (1 + step) / (2 * rayDir) - (floatPos - pos) / rayDir;
    vec3 tdelta = 1 / abs(rayDir);

    uint vox = textureLod(
        usampler3D(MyMaterial_texture, MyMaterial_texture_sampler),
        pos / object_size, 0).r;
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
        if (any(lessThan(pos, vec3(0, 0, 0))) || any(greaterThanEqual(pos, object_size))) {
            break;
        }
        vox = textureLod(
            usampler3D(MyMaterial_texture, MyMaterial_texture_sampler),
            pos / object_size, 0).r;
    }

    if (vox == 0) {
        discard;
    } else {
        o_Target = vec4(Palette[vox].albedo, 1);
    }
}
