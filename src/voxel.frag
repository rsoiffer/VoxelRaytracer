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

struct RaycastHit {
    uint vox;
    vec3 intPos;
    vec3 pos;
    vec3 normal;
};

RaycastHit raytrace(vec3 start, vec3 dir) {
    vec3 rayDir = normalize(dir);
    vec3 floatPos = start + rayDir * .00001;
    vec3 pos = floor(floatPos);
    vec3 step = sign(rayDir);
    // if (step.x == 1 then 1/rayDir.x - floatPos.x/rayDir.x else -floatPos.x / rayDir.x
    vec3 tmax = (0.5 + 0.5 * step - floatPos + pos) / rayDir;
    vec3 tdelta = 1 / abs(rayDir);

    pos = (pos + .5) / object_size;
    step = step / object_size;
    vec3 normal = vec3(0, 0, 0);
    float t = 0;

    uint vox = textureLod(
        usampler3D(MyMaterial_texture, MyMaterial_texture_sampler),
        pos, 0).r;
    while (vox == 0) {
        if (tmax.x < tmax.y) {
            if (tmax.x < tmax.z) {
                t = tmax.x;
                pos.x += step.x;
                tmax.x += tdelta.x;
                normal = vec3(1, 0, 0);
            } else {
                t = tmax.z;
                pos.z += step.z;
                tmax.z += tdelta.z;
                normal = vec3(0, 0, 1);
            }
        } else {
            if (tmax.y < tmax.z) {
                t = tmax.y;
                pos.y += step.y;
                tmax.y += tdelta.y;
                normal = vec3(0, 1, 0);
            } else {
                t = tmax.z;
                pos.z += step.z;
                tmax.z += tdelta.z;
                normal = vec3(0, 0, 1);
            }
        }
        if (any(lessThan(pos, vec3(0, 0, 0))) || any(greaterThanEqual(pos, vec3(1, 1, 1)))) {
            break;
        }
        vox = textureLod(
            usampler3D(MyMaterial_texture, MyMaterial_texture_sampler),
            pos, 0).r;
    }
    return RaycastHit(vox, object_size * pos, floatPos + t * rayDir, -normal * sign(step));
}

void main() {
    RaycastHit r = raytrace(v_ObjectPos, v_ObjectPos - camera_object_pos);
    if (r.vox == 0) {
        discard;
    }
    o_Target = vec4(Palette[r.vox].albedo, 1);
}
