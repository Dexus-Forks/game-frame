/* sgltest.exe: draws a known scene through sgl.dll (wrappers/sgl) without a game, to check
 * the stand-in's drawing under Wine: a window of class "SglTest", the address callback,
 * a device and a viewport with a square camera rectangle as the POD case study makes them,
 * an ambient and a point light, a camera 6 units from a textured cube that turns, a floor, a
 * light volume (a cone, as a car's headlight beam), a translucent pane, fog, and a 2D bar
 * drawn over the marker as a game's dashboard is. Run with SGL_SHOT_DIR set to keep captures.
 *
 *   zig cc -target x86-windows-gnu -O2 wrappers/sgl/test/sgltest.c -o sgltest.exe -luser32
 *   WRAPPER_WINDOW_CLASS=SglTest SGL_SHOT_DIR='C:\shots' SGL_SHOT_EVERY=30 SGL_SCALE=2 \
 *     wine sgltest.exe [FRAMES]
 *
 * Licence: Apache-2.0. */
#include <windows.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>

#define W 640
#define H 480
static unsigned short frame[W * H];

typedef struct { void *mem; unsigned short stride; unsigned char bpp; } __attribute__((packed)) address_params;
typedef struct { char id[4]; int x, y; unsigned char *pixels; } imap;

static int __cdecl next_address(address_params *p) {
    p->mem = frame;
    p->stride = W * 2;
    p->bpp = 16;
    return 0;
}

#define F(ret, name, ...) static ret(__cdecl *name)(__VA_ARGS__);
F(int, sgl_use_address_mode, void *, void *)
F(int, sgl_create_screen_device, int, int, int, int, int)
F(int, sgl_create_viewport, int, int, int, int, int, float, float, float, float)
F(int, sgl_create_list, int, int, int)
F(void, sgl_to_parent, void)
F(int, sgl_create_transform, int)
F(void, sgl_modify_transform, int, int)
F(void, sgl_translate, float, float, float)
F(void, sgl_rotate, float *, float)
F(int, sgl_create_camera, float, float, float)
F(int, sgl_create_ambient_light, int, float *, int)
F(int, sgl_create_point_light, int, float *, float *, float *, int, int, int)
F(int, sgl_create_mesh, int)
F(void, sgl_add_vertices, int, float *, float *, float *)
F(void, sgl_add_face, int, int *)
F(void, sgl_set_diffuse, float *)
F(void, sgl_set_ambient, float *)
F(void, sgl_set_texture_map, int, int, int)
F(void, sgl_set_texture_effect, int, int, int, int)
F(int, sgl_preprocess_texture, int, int, int, int, imap *, void *, imap *)
F(int, sgl_create_texture, int, int, int, int, imap *, void *)
F(void, sgl_set_fog, int, float *, float)
F(void, sgl_set_background_colour, int, float *)
F(void, sgl_render, int, int, int)
F(int, sgl_create_light_volume, int, int, int)
F(void, sgl_add_plane, float *, float *, float *, int, float *, float *, float *, float *, float *, float *)
F(void, sgl_add_simple_plane, float *, float *, int)
F(void, sgl_set_opacity, float)
F(void, sgl_set_glow, float *)
F(void, sgl_new_translucent, void)
F(void, sgl_scale, float, float, float)
F(int, sgl_wrapper_present, void *, void *, unsigned, unsigned, unsigned, unsigned)

static HMODULE lib;
static void *get(const char *n) {
    void *p = GetProcAddress(lib, n);
    if (!p) { fprintf(stderr, "sgl.dll has no %s\n", n); exit(1); }
    return p;
}
#define L(name) *(void **)&name = get(#name)

static LRESULT CALLBACK proc(HWND w, UINT m, WPARAM a, LPARAM b) { return DefWindowProcA(w, m, a, b); }

/* A cube of side 2: six faces, four vertices each, with normals and texture coordinates. */
static void cube(int tex) {
    static const float n[6][3] = {{0, 0, -1}, {0, 0, 1}, {-1, 0, 0}, {1, 0, 0}, {0, 1, 0}, {0, -1, 0}};
    float v[24][3], nv[24][3], uv[24][2];
    for (int f = 0; f < 6; f++) {
        float a[3], b[3], c[3];
        for (int k = 0; k < 3; k++) c[k] = n[f][k];
        /* Two axes across the face. */
        int i = fabsf(c[0]) > 0.5f ? 1 : 0, j = fabsf(c[2]) > 0.5f ? 1 : 2;
        if (i == j) j = 2;
        for (int k = 0; k < 3; k++) { a[k] = k == i; b[k] = k == j; }
        static const float s[4][2] = {{-1, -1}, {1, -1}, {1, 1}, {-1, 1}};
        for (int q = 0; q < 4; q++) {
            for (int k = 0; k < 3; k++) {
                v[f * 4 + q][k] = c[k] + s[q][0] * a[k] + s[q][1] * b[k];
                nv[f * 4 + q][k] = c[k];
            }
            uv[f * 4 + q][0] = (s[q][0] + 1) / 2;
            uv[f * 4 + q][1] = (s[q][1] + 1) / 2;
        }
    }
    float white[3] = {1, 1, 1}, dim[3] = {0.6f, 0.6f, 0.6f};
    sgl_create_mesh(0);
    sgl_set_diffuse(white);
    sgl_set_ambient(dim);
    sgl_set_texture_effect(1, 1, 1, 1);
    sgl_set_texture_map(tex, 0, 0);
    sgl_add_vertices(24, &v[0][0], &nv[0][0], &uv[0][0]);
    for (int f = 0; f < 6; f++) {
        int ids[4] = {f * 4, f * 4 + 1, f * 4 + 2, f * 4 + 3};
        sgl_add_face(4, ids);
    }
}

int main(int argc, char **argv) {
    int frames = argc > 1 ? atoi(argv[1]) : 120;
    lib = LoadLibraryA("sgl.dll");
    if (!lib) { fprintf(stderr, "no sgl.dll\n"); return 1; }
    L(sgl_use_address_mode); L(sgl_create_screen_device); L(sgl_create_viewport); L(sgl_create_list);
    L(sgl_to_parent); L(sgl_create_transform); L(sgl_modify_transform); L(sgl_translate); L(sgl_rotate);
    L(sgl_create_camera); L(sgl_create_ambient_light); L(sgl_create_point_light); L(sgl_create_mesh);
    L(sgl_add_vertices); L(sgl_add_face); L(sgl_set_diffuse); L(sgl_set_ambient); L(sgl_set_texture_map);
    L(sgl_set_texture_effect); L(sgl_preprocess_texture); L(sgl_create_texture); L(sgl_set_fog);
    L(sgl_set_background_colour); L(sgl_render); L(sgl_wrapper_present);
    L(sgl_create_light_volume); L(sgl_add_plane); L(sgl_add_simple_plane); L(sgl_set_opacity); L(sgl_set_glow);
    L(sgl_new_translucent); L(sgl_scale);

    WNDCLASSA wc = {0};
    wc.lpfnWndProc = proc;
    wc.hInstance = GetModuleHandleA(0);
    wc.lpszClassName = "SglTest";
    RegisterClassA(&wc);
    RECT r = {0, 0, W * 2, H * 2};
    AdjustWindowRect(&r, WS_OVERLAPPEDWINDOW, 0);
    HWND hwnd = CreateWindowA("SglTest", "sgltest", WS_OVERLAPPEDWINDOW | WS_VISIBLE, 0, 0, r.right - r.left, r.bottom - r.top, 0, 0, wc.hInstance, 0);

    void *status;
    sgl_use_address_mode((void *)next_address, &status);
    int dev = sgl_create_screen_device(0, W, H, 0, 1);
    /* As FUN_0049dbe0: the camera's square window over the middle of the device. */
    int vp = sgl_create_viewport(dev, 0, 0, W - 1, H - 1, (W - H) / 2.0f, 0, W - (W - H) / 2.0f, H);

    /* A 64 x 64 checker, red and yellow, through preprocess and create, as a game does. */
    static unsigned char px[64 * 64 * 4];
    for (int y = 0; y < 64; y++)
        for (int x = 0; x < 64; x++) {
            unsigned char *p = px + (y * 64 + x) * 4;
            int on = ((x >> 3) ^ (y >> 3)) & 1;
            p[0] = 255; p[1] = on ? 220 : 40; p[2] = on ? 40 : 30; p[3] = 0;
        }
    imap src = {{'I', 'M', 'A', 'P'}, 64, 64, px}, done;
    sgl_preprocess_texture(0, 1, 0, 1, &src, 0, &done);
    int tex = sgl_create_texture(0, 1, 0, 1, &done, 0);

    float amb[3] = {0.35f, 0.35f, 0.35f}, lc[3] = {0.9f, 0.9f, 0.8f}, dir[3] = {0, 0, 1}, pos[3] = {-6, 6, -4};
    sgl_create_ambient_light(0, amb, 0);
    sgl_create_point_light(0, lc, dir, pos, 0, 0, 0);
    sgl_create_list(0, 1, 0);
    sgl_create_transform(1);
    int cam = sgl_create_camera(1.0f, 0.5f, 0.0f);
    sgl_to_parent();
    float fog[3] = {0.5f, 0.55f, 0.6f}, sky[3] = {0.2f, 0.3f, 0.6f};
    sgl_set_fog(cam, fog, 0.02f);
    sgl_set_background_colour(cam, sky);
    sgl_create_list(1, 1, 0);
    int t = sgl_create_transform(1);
    cube(tex);
    sgl_to_parent();

    /* A grey floor below the cube. */
    {
        float grey[3] = {0.5f, 0.5f, 0.5f};
        float v[4][3] = {{-12, -1.8f, 0}, {12, -1.8f, 0}, {12, -1.8f, 30}, {-12, -1.8f, 30}};
        float nv[4][3] = {{0, 1, 0}, {0, 1, 0}, {0, 1, 0}, {0, 1, 0}};
        int ids[4] = {0, 1, 2, 3};
        sgl_create_list(0, 1, 0);
        sgl_create_mesh(0);
        sgl_set_diffuse(grey);
        sgl_set_ambient(grey);
        sgl_add_vertices(4, &v[0][0], &nv[0][0], 0);
        sgl_add_face(4, ids);
        sgl_to_parent();
    }
    /* A light volume as POD's FUN_0049b770 makes it (the case study): 40 planes through an apex and the rim of
     * a circle of radius 2 at distance 12, and a plane across the end; placed above and left,
     * pointing down onto the floor. */
    {
        float white[3] = {1, 1, 1}, warm[3] = {1, 1, 0.9f}, d[3] = {0, 0, 1}, o[3] = {0, 0, 0}, apex[3] = {0, 0, 0};
        float zn[3] = {0, 0, 1}, uv[2] = {0, 0}, r = 2.5f, len = 12;
        float xa[3] = {1, 0, 0};
        sgl_create_list(1, 1, 0);
        int light = sgl_create_point_light(1, white, d, o, 0, 0, 0);
        /* The transform first: a light volume sits in the list where it is made. */
        sgl_create_transform(1);
        sgl_translate(-3.0f, 3.0f, 3.0f);
        sgl_rotate(xa, 0.9f);
        sgl_set_diffuse(warm);
        sgl_set_glow(warm);
        sgl_set_opacity(0.35f);
        sgl_create_light_volume(0, 1, light);
        sgl_create_mesh(0);
        float a = 0, p1[3], p3[3], n1[3], n3[3];
        for (int k = 0; k < 40; k++) {
            float a1 = a - 0.157075f, a2 = a1 - 0.157075f;
            a = a1;
            p1[0] = cosf(a1) * r; p1[1] = sinf(a1) * r; p1[2] = len;
            p3[0] = cosf(a2) * r; p3[1] = sinf(a2) * r; p3[2] = len;
            n1[0] = cosf(a1); n1[1] = sinf(a1); n1[2] = 0;
            n3[0] = cosf(a2); n3[1] = sinf(a2); n3[2] = 0;
            sgl_add_plane(p1, apex, p3, 0, n1, zn, n3, uv, uv, uv);
        }
        sgl_add_simple_plane(p1, zn, 1);
        sgl_to_parent();
    }
    /* A blue pane at half opacity in front of the cube's right side, in a translucent pass. */
    {
        float blue[3] = {0.2f, 0.4f, 1.0f};
        float v[4][3] = {{0.5f, -1.5f, 3.5f}, {3, -1.5f, 3.5f}, {3, 1.5f, 3.5f}, {0.5f, 1.5f, 3.5f}};
        int ids[4] = {0, 1, 2, 3};
        sgl_new_translucent();
        sgl_create_list(0, 1, 0);
        sgl_create_mesh(0);
        sgl_set_diffuse(blue);
        sgl_set_ambient(blue);
        sgl_set_opacity(0.5f);
        sgl_add_vertices(4, &v[0][0], 0, 0);
        sgl_add_face(4, ids);
        sgl_to_parent();
    }

    float yaxis[3] = {0, 1, 0}, xaxis[3] = {1, 0, 0};
    for (int i = 0; i < frames; i++) {
        float a = i * 0.05f;
        sgl_modify_transform(t, 1);
        sgl_translate(0, 0, 6);
        sgl_rotate(yaxis, a);
        sgl_rotate(xaxis, a * 0.7f);
        sgl_render(vp, cam, 1);
        /* The "dashboard": a bar over the top rows, as the game draws over the render. */
        for (int y = 0; y < 24; y++)
            for (int x = 0; x < W; x++) frame[y * W + x] = (unsigned short)((x * 31 / W) << 11 | 0x07e0);
        sgl_wrapper_present(hwnd, frame, W, H, W * 2, 16);
        MSG m;
        while (PeekMessageA(&m, 0, 0, 0, PM_REMOVE)) { TranslateMessage(&m); DispatchMessageA(&m); }
        Sleep(16);
    }
    Sleep(500);
    return 0;
}
