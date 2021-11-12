#version 450

layout (location = 0) in vec2 v_uv;

layout (location = 0) out vec4 color;

layout (set = 0, binding = 0) readonly buffer Block {
    ivec3 position;
    uint data[];
} block;

layout (set = 0, binding = 1) uniform View {
    vec4 position;
    vec4 look_dir;
    float aspect_ratio;
    float fov;
} view;

const float PI = 3.1415926;
const vec3 UP = vec3(0.0, 1.0, 0.0);
const uint BLOCK_SIZE = 16;
const uint BLOCK_VOLUME = BLOCK_SIZE * BLOCK_SIZE * BLOCK_SIZE;
const uint BLOCK_MAX_STEPS = BLOCK_SIZE * 3;

struct Ray {
    vec3 origin;
    vec3 dir;
    vec3 inv_dir;
};

struct Box {
    vec3 center;
    vec3 radius;
    vec3 inv_radius;
};

// https://www.shadertoy.com/view/ld23DV
// The MIT License
// Copyright © 2014 Inigo Quilez
float sBox(in Ray ray, in vec3 center, in vec3 radius) {
    ray.origin -= center;
    vec3 m = 1.0/ray.dir;
    vec3 n = m*ray.origin;
    vec3 k = abs(m)*radius;

    vec3 t1 = -n - k;
    vec3 t2 = -n + k;

    float tN = max(max(t1.x, t1.y), t1.z);
    float tF = min(min(t2.x, t2.y), t2.z);
    if(tN > tF || tF < 0.0)
        return -1.0;

    return tN;
}

float degrees_to_radians(in float degrees) {
    return degrees / 180.0 * PI;
}

Ray generate_ray() {
    vec3 origin = view.position.xyz;
    vec3 look_dir = normalize(view.look_dir.xyz);

    float scale = 2 * tan(degrees_to_radians(view.fov) / 2);
    vec3 u = scale * normalize(cross(look_dir, UP)) * view.aspect_ratio;
    vec3 v = scale * normalize(cross(u, look_dir));

    vec3 dir = normalize(look_dir - (v_uv.x - 0.5) * u + (v_uv.y - 0.5) * v);
    vec3 inv_dir = 1.0 / dir;

    return Ray(origin, dir, inv_dir);
}

vec3 shade(vec3 albedo, vec3 normal) {
    return albedo * (0.8 + 0.2 * dot(normal, normalize(vec3(0.9, 0.5, 0.1))));
}

struct DDA {
    ivec3 voxel_pos;
    vec3 d_dist;
    ivec3 ray_step;
    vec3 dist;
    bvec3 mask;
};

void dda_init(out DDA dda, in Ray ray) {
    dda.voxel_pos = ivec3(floor(ray.origin));
    dda.d_dist = abs(vec3(length(ray.dir)) * ray.inv_dir);
    vec3 s = sign(ray.dir);
    dda.ray_step = ivec3(s);
    dda.dist = (s * (vec3(dda.voxel_pos) - ray.origin) + (s * 0.5) + 0.5) * dda.d_dist;
}

void dda_step(inout DDA dda) {
    bvec3 lt = lessThan(dda.dist.xxy, dda.dist.yzz);
    if (lt.x && lt.y) {
        dda.dist.x += dda.d_dist.x;
        dda.voxel_pos.x += dda.ray_step.x;
        dda.mask = bvec3(true, false, false);
    } else if (!lt.x && lt.z) {
        dda.dist.y += dda.d_dist.y;
        dda.voxel_pos.y += dda.ray_step.y;
        dda.mask = bvec3(false, true, false);
    } else {
        dda.dist.z += dda.d_dist.z;
        dda.voxel_pos.z += dda.ray_step.z;
        dda.mask = bvec3(false, false, true);
    }
}

void dda_end(in DDA dda, in Ray ray, out float distance, out vec3 normal) {
    vec3 mini = (dda.voxel_pos - ray.origin + 0.5 - 0.5 * vec3(dda.ray_step)) * ray.inv_dir;
    distance = max(mini.x, max(mini.y, mini.z));
    normal = vec3(dda.mask) * -sign(dda.ray_step);
}

uint fetch_voxel(ivec3 pos) {
    bool in_bounds = all(lessThan(pos, vec3(BLOCK_SIZE))) && all(greaterThanEqual(pos, vec3(0)));
    return in_bounds ? block.data[pos.x + pos.y * BLOCK_SIZE + pos.z * BLOCK_SIZE * BLOCK_SIZE] : 0;
}

bool block_dda(in Ray ray, in ivec3 block_pos, out float distance, out vec3 normal, out uint voxel) {
    ray.origin -= block_pos;
    bool intersects = false;

    DDA dda;
    dda_init(dda, ray);

    for (int i = 0; i < BLOCK_MAX_STEPS; i++) {
        dda_step(dda);

        voxel = fetch_voxel(dda.voxel_pos);
        if (voxel != 0) {
            intersects = true;
            break;
        }
    }

    dda_end(dda, ray, distance, normal);

    return intersects;
}

vec3 voxel_color(in uint voxel) {
    return vec3(
        float((voxel & 0xFF000000) >> 24),
        float((voxel & 0x00FF0000) >> 16),
        float((voxel & 0x0000FF00) >> 8)
    ) / 255;
}

void main() {
    float distance;
    vec3 normal;
    Ray ray = generate_ray();

    float d = sBox(ray, vec3(block.position * 16) + vec3(8), vec3(8));
    if (d < 0) {
        // discard;
    }

    ray.origin += ray.dir * (d - 0.1);

    uint voxel;
    bool intersects = block_dda(ray, block.position * 16, distance, normal, voxel);
    if (intersects) {
        color = vec4(shade(voxel_color(voxel), normal), 1.0);
        gl_FragDepth = (d + distance) / 10000.0;
    } else {
        discard;
    }
}
