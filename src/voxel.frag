#version 450

layout(location = 0) in vec3 v_ObjectPos;
layout(location = 1) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 1) uniform utexture3D MyMaterial_texture;
layout(set = 1, binding = 2) uniform sampler MyMaterial_texture_sampler;

layout(set = 1, binding = 3) uniform texture2D MyMaterial_blue_noise;
layout(set = 1, binding = 4) uniform sampler MyMaterial_blue_noise_sampler;

struct VoxelMaterial {
    vec3 albedo;
};
layout(set = 1, binding = 5) buffer MyMaterial_palette {
    VoxelMaterial[] Palette;
};

layout(set = 1, binding = 6) uniform MyMaterial_camera_object_pos {
    vec3 camera_object_pos;
};

layout(set = 1, binding = 7) uniform MyMaterial_object_size {
    vec3 object_size;
};

struct RaycastHit {
    uint vox;
    vec3 intPos;
    vec3 pos;
    vec3 normal;
};

bool outsideUnitCube(vec3 pos) {
    return any(lessThan(pos, vec3(0, 0, 0))) || any(greaterThan(pos, vec3(1, 1, 1)));
}

RaycastHit raytrace(vec3 start, vec3 dir) {
    vec3 rayDir = normalize(dir);
    vec3 floatPos = start + rayDir * 1e-4;
    vec3 pos = floor(floatPos);
    vec3 step = sign(rayDir);
    // if (step.x == 1 then 1/rayDir.x - floatPos.x/rayDir.x else -floatPos.x / rayDir.x
    vec3 tmax = (0.5 + 0.5 * step - floatPos + pos) / rayDir;
    vec3 tdelta = 1 / abs(rayDir);

    pos = (pos + .5) / object_size;
    step = step / object_size;
    vec3 normal = vec3(0, 0, 0);
    float t = 0;

    uint vox = 0;
    for (uint i = 0; i < 1000; i++) {
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
        if (outsideUnitCube(pos)) {
            break;
        }
        vox = textureLod(
            usampler3D(MyMaterial_texture, MyMaterial_texture_sampler),
            pos, 0).r;
        if (vox != 0) {
            break;
        }
    }
    return RaycastHit(vox, object_size * pos, floatPos + t * rayDir, -normal * sign(step));
}

float rand(vec2 co) {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453) - .5;
}

vec4 blueNoise(vec2 screenPos) {
    return texture(
        sampler2D(MyMaterial_blue_noise, MyMaterial_blue_noise_sampler),
        mod(screenPos / 1024, 1));
}

float rayBoxIntersection(vec3 start, vec3 dir, vec3 boxMax) {
    vec3 t1 = -start / dir;
    vec3 t2 = (boxMax - start) / dir;
    vec3 tMin = min(t1, t2);
    vec3 tMax = max(t1, t2);
    float tFirst = max(tMin.x, max(tMin.y, tMin.z));
    float tSecond = min(tMax.x, min(tMax.y, tMax.z));
    return clamp(0, tFirst, tSecond);
}

void main() {
    vec3 dir = normalize(v_ObjectPos - camera_object_pos);
    float time = rayBoxIntersection(camera_object_pos, dir, object_size);

    RaycastHit r = raytrace(camera_object_pos + (time - 2e-4) * dir, dir);
    if (r.vox == 0) {
        discard;
    }
    vec3 albedo = pow(Palette[r.vox].albedo, vec3(2.2));

    vec3 light = vec3(.02);

    vec2 screenPos = gl_FragCoord.xy;
    for (int i = 0; i < 16; i++) {
        vec3 newDir = blueNoise(screenPos + vec2(57 * i, 139 * i)).rgb - .5;
//        vec3 newDir = vec3(
//            rand(screenPos + vec2(.03 * i, 0)),
//            rand(screenPos + vec2(.03 * i, .1)),
//            rand(screenPos + vec2(.03 * i, .2)));
        if (dot(newDir, r.normal) < 0) {
            newDir = -1 * newDir;
        }
        RaycastHit r2 = raytrace(r.pos, newDir);
        if (r2.vox == 0) {
            light += vec3(1, 1, 1) * (.7 + .4 * newDir.y) / 16;
        }
    }

    vec3 sunDir = vec3(-.2, 1, .3);
    if (dot(sunDir, r.normal) > 0) {
        RaycastHit r3 = raytrace(r.pos, sunDir);
        if (r3.vox == 0) {
            light += vec3(1, .9, .8);
        }
    }

    vec3 color = clamp(albedo * light * .5, 0, 1);
    o_Target = vec4(pow(color, vec3(1/2.2)), 1);
}
