// =========================================================================
// Shader: "Cyberpunk Quantum Core"
// Description: A complex, highly optimized 3D raymarching scene featuring:
// - KIFS (Kaleidoscopic Iterated Function System) Fractal architecture
// - Volumetric glow accumulation
// - Dynamic space warping & non-linear cinematic camera pathing
// - Fake Ambient Occlusion & Physical Post-Processing
// =========================================================================

#define MAX_STEPS 150
#define MAX_DIST 60.0
#define SURF_DIST 0.002

// Global distances used for calculating volumetric glow
float g_helix, g_cubes, g_rings;

// Precomputed rotation matrices for the KIFS fractal (Massive performance boost)
const mat2 r1 = mat2(0.7071, -0.7071, 0.7071, 0.7071);
const mat2 r2 = mat2(0.8253, -0.5646, 0.5646, 0.8253);
const mat2 r3 = mat2(0.9553, -0.2955, 0.2955, 0.9553);

// Global time-based matrices (Set in mainImage)
mat2 g_rotCube1, g_rotCube2;

// Standard 2D rotation matrix
mat2 rot(float a) {
    float s = sin(a), c = cos(a);
    return mat2(c, -s, s, c);
}

// The twisting winding path of the tunnel
vec2 path(float z) {
    return vec2(sin(z * 0.3) * 2.5, cos(z * 0.2) * 2.5);
}

// --- Core Distance Field Map ---
vec2 map(vec3 p) {
    // Save original Z for texturing and pathing later
    float pZ = p.z;

    // Warp space to follow the curve of the path
    p.xy -= path(p.z);

    // Twist the tunnel dynamically down its length
    p.xy *= rot(p.z * 0.03 + sin(iTime * 0.2) * 0.1);

    // 1. KIFS Fractal Walls
    vec3 q = p;
    q.z = mod(q.z, 4.0) - 2.0; // Repeat domain infinitely along Z
    float s = 1.0;

    for(int i = 0; i < 5; i++) {
        // Fold space and rotate repeatedly to create complex sci-fi greebles
        q = abs(q) - vec3(1.2, 0.8, 1.0);
        q.xy *= r1;
        q.xz *= r2;
        q.yz *= r3;
        q *= 1.4;
        s *= 1.4;
    }
    vec3 dbox = abs(q) - vec3(0.5);
    // Exact distance to the resulting fractal matrix
    float fractal = (length(max(dbox, 0.0)) + min(max(dbox.x, max(dbox.y, dbox.z)), 0.0)) / s;

    // Hollow out a cylindrical path through the middle of the fractal
    float tRad = 2.5 + sin(pZ * 1.2) * 0.2;
    float tunnel = tRad - length(p.xy);
    float walls = max(tunnel, fractal); // Boolean intersection

    // 2. Central Energy Double Helix
    vec3 p1 = p;
    p1.xy *= rot(p.z * 1.5 - iTime * 3.0);
    float h1 = length(p1.xy - vec2(0.3, 0.0)) - 0.02;
    float h2 = length(p1.xy - vec2(-0.3, 0.0)) - 0.02;
    g_helix = min(h1, h2);

    // 3. Orbiting Data Cubes
    vec3 pC = p;
    pC.z = mod(pC.z, 2.0) - 1.0;
    pC.xy *= rot(p.z * 0.2 + iTime);

    // Polar fold to create a ring of spinning cubes
    float aC = atan(pC.y, pC.x);
    float rC = length(pC.xy);
    float secC = 6.283185 / 6.0;
    aC = mod(aC, secC) - secC * 0.5;
    pC.x = rC * cos(aC);
    pC.y = rC * sin(aC);

    pC.x -= 1.2; // Distance from center
    pC.xy *= g_rotCube1;
    pC.xz *= g_rotCube2;
    vec3 qb = abs(pC) - vec3(0.04);
    g_cubes = length(max(qb, 0.0)) + min(max(qb.x, max(qb.y, qb.z)), 0.0) - 0.01;

    // 4. Neon Structural Rings
    vec3 pR = p;
    pR.z = mod(pR.z, 8.0) - 4.0;
    float a = atan(pR.y, pR.x);
    float radR = length(pR.xy);

    // Polar fold to chop the rings into 12 tech-like segments
    float sector = 6.283185 / 12.0;
    a = mod(a + iTime * 0.5, sector) - sector * 0.5;
    pR.x = radR * cos(a);
    pR.y = radR * sin(a);

    vec3 qRing = pR - vec3(2.1, 0.0, 0.0);
    vec3 rbox = abs(qRing) - vec3(0.05, 0.15, 0.05);
    g_rings = length(max(rbox, 0.0)) + min(max(rbox.x, max(rbox.y, rbox.z)), 0.0) - 0.01;

    // Combine everything and assign material IDs
    vec2 res = vec2(walls, 0.0); // Material 0: Dark Cyber-Metal
    if(g_helix < res.x) res = vec2(g_helix, 1.0); // Material 1: Cyan Helix
    if(g_cubes < res.x) res = vec2(g_cubes, 2.0); // Material 2: Orange Cubes
    if(g_rings < res.x) res = vec2(g_rings, 3.0); // Material 3: Magenta Rings

    return res;
}

// Surface Normal generation via central differences
vec3 getNormal(vec3 p) {
    vec2 e = vec2(0.002, 0.0);
    return normalize(vec3(
        map(p + e.xyy).x - map(p - e.xyy).x,
        map(p + e.yxy).x - map(p - e.yxy).x,
        map(p + e.yyx).x - map(p - e.yyx).x
    ));
}

// Fake Ambient Occlusion for deep shadows inside the fractal corners
float calcAO(vec3 p, vec3 n) {
    float occ = 0.0;
    float sca = 1.0;
    for(int i = 0; i < 5; i++) {
        float h = 0.02 + 0.15 * float(i) / 4.0;
        float d = map(p + h * n).x;
        occ += (h - d) * sca;
        sca *= 0.75;
    }
    return clamp(1.0 - 2.5 * occ, 0.0, 1.0);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    // Normalize coordinates
    vec2 uv = (fragCoord - 0.5 * iResolution.xy) / iResolution.y;
    vec2 m = iMouse.xy / iResolution.xy;

    // Initialize global time matrices
    g_rotCube1 = rot(iTime * 3.0);
    g_rotCube2 = rot(iTime * 1.5);

    // Camera Setup
    float tCam = iTime * 4.0;
    vec3 ro = vec3(path(tCam), tCam);
    vec3 lookAt = vec3(path(tCam + 1.0), tCam + 1.0);

    // Add natural cinematic drift to the camera
    ro.x += sin(iTime * 0.5) * 0.3;
    ro.y += cos(iTime * 0.4) * 0.3;
    lookAt.x += sin(iTime * 0.5) * 0.3;
    lookAt.y += cos(iTime * 0.4) * 0.3;

    // Frame of reference
    vec3 f_cam = normalize(lookAt - ro);

    // Interactive mouse look (simulates FPS view)
    if(iMouse.z > 0.0) {
        f_cam.yz *= rot((m.y - 0.5) * 2.0);
        f_cam.xz *= rot(-(m.x - 0.5) * 4.0);
    }

    // Camera Roll banking naturally with the curve
    float roll = sin(ro.z * 0.1) * 0.3 + sin(iTime * 0.2) * 0.1;
    vec3 globalUp = vec3(sin(roll), cos(roll), 0.0);

    vec3 r_cam = normalize(cross(globalUp, f_cam));
    vec3 u_cam = cross(f_cam, r_cam);

    // Ray direction with FOV
    vec3 rd = normalize(uv.x * r_cam + uv.y * u_cam + 0.8 * f_cam);

    // Raymarching variables
    float t = 0.0;
    float mat = 0.0;
    vec3 glow = vec3(0.0);

    // --- Raymarch Loop ---
    for(int i = 0; i < MAX_STEPS; i++) {
        vec3 p = ro + rd * t;
        vec2 res = map(p);

        // Volumetric Glow Calculation (Accumulates based on inverse distance)
        float atten = exp(-0.015 * t); // Distance attenuation (Beer's Law)
        float pulseH = 1.0 + 0.2 * sin(iTime * 5.0 + p.z * 2.0); // Travel pulses
        float pulseR = 1.0 + 0.2 * cos(iTime * 3.0 - p.z * 1.0);

        glow += vec3(0.0, 0.8, 1.0) * (0.0004 / (0.005 + g_helix * g_helix)) * atten * pulseH;
        glow += vec3(1.0, 0.6, 0.0) * (0.0005 / (0.005 + g_cubes * g_cubes)) * atten;
        glow += vec3(1.0, 0.0, 0.6) * (0.0004 / (0.005 + g_rings * g_rings)) * atten * pulseR;

        if(res.x < SURF_DIST || t > MAX_DIST) {
            if(res.x < SURF_DIST) mat = res.y;
            break;
        }

        t += res.x * 0.65; // Step multiplier for safety through folded non-Euclidean space
    }

    vec3 col = vec3(0.0);

    // --- Material Shading Engine ---
    if(t < MAX_DIST) {
        vec3 p = ro + rd * t;
        vec3 n = getNormal(p);
        vec3 v = -rd;

        if(mat == 0.0) { // Dark Cyber-Metal Walls
            vec3 base = vec3(0.03, 0.04, 0.06);

            // Recompute local un-warped coordinates for emissive grid texturing
            vec3 lp = p;
            lp.xy -= path(lp.z);
            lp.xy *= rot(lp.z * 0.03 + sin(iTime * 0.2) * 0.1);

            // Procedural grid lines wrapping over the fractal
            float lineX = smoothstep(0.95, 1.0, sin(lp.x * 20.0));
            float lineY = smoothstep(0.95, 1.0, sin(lp.y * 20.0));
            float lineZ = smoothstep(0.95, 1.0, sin(lp.z * 20.0));
            float grid = max(lineX, max(lineY, lineZ));
            base += vec3(0.0, 0.4, 0.8) * grid * 0.3;

            // Lighting properties
            float ao = calcAO(p, n);
            vec3 l = normalize(vec3(0.5, 0.8, -0.5)); // Directional key light
            float dif = max(dot(n, l), 0.0);
            float amb = 0.5 + 0.5 * n.y; // Hemisphere ambient
            float fresnel = pow(1.0 - max(dot(n, v), 0.0), 4.0);
            vec3 h = normalize(l + v);
            float spec = pow(max(dot(n, h), 0.0), 32.0); // Specular reflection

            // Final material combination
            col = base * (dif + amb * 0.2) * ao;
            col += vec3(0.2, 0.5, 1.0) * spec * ao; // Blue-tinted specular
            col += vec3(0.0, 0.8, 1.0) * fresnel * ao * 0.5; // Cyan rim light

        } else if(mat == 1.0) {
            col = vec3(0.0, 1.0, 0.8) * 2.0; // Helix Emissive
        } else if(mat == 2.0) {
            col = vec3(1.0, 0.6, 0.0) * 2.0; // Cubes Emissive
        } else if(mat == 3.0) {
            col = vec3(1.0, 0.0, 0.6) * 2.0; // Rings Emissive
        }
    }

    // Atmospheric Distance Fog
    float fog = 1.0 - exp(-t * t * 0.002);
    col = mix(col, vec3(0.01, 0.01, 0.03), fog);

    // Add accumulated volumetrics AFTER fog so light pierces the atmosphere
    col += glow;

    // --- Post-Processing ---

    // ACES Film Tone Mapping (Simulates physical camera dynamic range)
    col = clamp((col * (2.51 * col + 0.03)) / (col * (2.43 * col + 0.59) + 0.14), 0.0, 1.0);

    // Cinematic Vignette
    vec2 uvQ = fragCoord.xy / iResolution.xy;
    col *= 0.5 + 0.5 * pow(16.0 * uvQ.x * uvQ.y * (1.0 - uvQ.x) * (1.0 - uvQ.y), 0.2);

    // sRGB Gamma Correction
    col = pow(col, vec3(1.0 / 2.2));

    fragColor = vec4(col, 1.0);
}