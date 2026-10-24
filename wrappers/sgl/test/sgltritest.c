/* sgltritest.exe: draws a known scene through sgl.dll's immediate mode (sgltri_*, wrappers/sgl)
 * without a game. A window of class "SglTest", a screen device, then each frame:
 * sgltri_startofframe, a fogged floor of quads going into the distance, a box-shaped shadow
 * volume darkening the floor in the middle and a light volume lighting it on the right
 * (sgltri_shadow), a Gouraud triangle,
 * a textured quad that turns (perspective-correct, a red and yellow checker), a flat
 * triangle with a specular highlight, a translucent triangle with vertex alpha in front, and
 * sgltri_render. A 2D bar goes over the top rows, as a game's dashboard does: through the
 * address callback and sgl_wrapper_present, or with "ddraw", through sgl_use_ddraw_mode's 2D
 * callback. At the end it prints what the information calls return.
 *
 *   zig cc -target x86-windows-gnu -O2 wrappers/sgl/test/sgltritest.c -o sgltritest.exe -luser32
 *   WRAPPER_WINDOW_CLASS=SglTest SGL_SHOT_DIR='C:\shots' SGL_SHOT_EVERY=30 SGL_SCALE=2 \
 *     wine sgltritest.exe [FRAMES] [ddraw]
 *
 * Licence: Apache-2.0. */
#include <windows.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define W 640
#define H 480
static unsigned short frame[W * H];

/* sgl.h and sglwin32.h, packed. */
#pragma pack(push, 1)
typedef struct {
    int bFogOn;
    float fFogR, fFogG, fFogB;
    unsigned u32FogDensity;
    int bCullBackfacing;
    unsigned u32Flags;
    int nTextureName;
    float RESERVED;
    int bDoClipping;
    float cBackgroundColour[3];
    int eShadowLightVolMode;
    union { float fShadowBrightness; unsigned u32LightVolColour; } u;
    int bFlipU, bFlipV;
    int bDoUVTimesInvW;
    int FirstXRegion, LastXRegion, FirstYRegion, LastYRegion;
    float invRegX, invRegY;
    unsigned u32GlobalTrans;
    int n32MipmapOffset;
    int RenderRegions;
    unsigned u32Reserve9[9];
} SGLCONTEXT;
typedef struct { float fX, fY, fZ, fInvW; unsigned u32Colour, u32Specular; float fUOverW, fVOverW; } SGLVERTEX;
typedef struct { void *mem; unsigned short stride; unsigned char bpp; } address_params;
typedef struct { void *dd, *surface, *mem; unsigned short bpp; unsigned stride; unsigned short w, h; } surface_params;
typedef struct { char id[4]; int x, y; unsigned char *pixels; } imap;
typedef struct { char *library, *required_header; } sgl_versions;
typedef struct { char *s[13]; } sgl_win_versions;
#pragma pack(pop)
typedef char context_is_0x90_bytes[sizeof(SGLCONTEXT) == 0x90 ? 1 : -1];

#define SGLTT_GOURAUD 0x1
#define SGLTT_TEXTURE 0x2
#define SGLTT_HIGHLIGHT 0x4
#define SGLTT_VERTEXTRANS 0x200

#define F(ret, name, ...) static ret(__cdecl *name)(__VA_ARGS__);
F(int, sgl_use_address_mode, void *, void *)
F(int, sgl_use_ddraw_mode, HWND, void *)
F(int, sgl_use_eor_callback, void *)
F(int, sgl_create_screen_device, int, int, int, int, int)
F(int, sgl_preprocess_texture, int, int, int, int, imap *, void *, imap *)
F(int, sgl_create_texture, int, int, int, int, imap *, void *)
F(void, sgltri_startofframe, SGLCONTEXT *)
F(void, sgltri_triangles, SGLCONTEXT *, int, int (*)[3], SGLVERTEX *)
F(void, sgltri_quads, SGLCONTEXT *, int, int (*)[4], SGLVERTEX *)
F(void, sgltri_shadow, SGLCONTEXT *, int, int (*)[3], SGLVERTEX *, float (*)[2])
F(void, sgltri_render, SGLCONTEXT *)
F(int, sgltri_isrendercomplete, SGLCONTEXT *, unsigned)
F(void, sgl_get_errors, int *, int *)
F(sgl_versions *, sgl_get_versions, void)
F(sgl_win_versions *, sgl_get_win_versions, void)
F(unsigned long, sgl_get_free_texture_mem, void)
F(void, sgl_delete_all_textures, void)
F(int, sgl_wrapper_present, void *, void *, unsigned, unsigned, unsigned, unsigned)

static HMODULE lib;
static void *get(const char *n) {
    void *p = GetProcAddress(lib, n);
    if (!p) { fprintf(stderr, "sgl.dll has no %s\n", n); exit(1); }
    return p;
}
#define L(name) *(void **)&name = get(#name)

static LRESULT CALLBACK proc(HWND w, UINT m, WPARAM a, LPARAM b) { return DefWindowProcA(w, m, a, b); }

static int __cdecl next_address(address_params *p) {
    p->mem = frame;
    p->stride = W * 2;
    p->bpp = 16;
    return 0;
}

/* The "dashboard": a bar over the top rows. */
static void bar(unsigned short *mem, unsigned stride) {
    for (int y = 0; y < 24; y++)
        for (int x = 0; x < W; x++) mem[y * (stride / 2) + x] = (unsigned short)((x * 31 / W) << 11 | 0x07e0);
}

static int pictures;
static int __cdecl draw_2d(surface_params *p) {
    if (p->mem && p->bpp == 16) bar(p->mem, p->stride);
    pictures++;
    return 0;
}

static int renders;
static int __cdecl end_of_render(void) {
    renders++;
    return 0;
}

/* A point in the camera's space, projected as Direct3D's transform stage would: the screen
 * centre, 1 / z as 1 / w, and texture coordinates divided by w. */
static SGLVERTEX project(float x, float y, float z, unsigned colour, float u, float v) {
    SGLVERTEX o;
    float k = 400.0f / z;
    o.fX = W / 2 + x * k;
    o.fY = H / 2 - y * k;
    o.fZ = 1.0f - 1.0f / z;
    o.fInvW = 1.0f / z;
    o.u32Colour = colour;
    o.u32Specular = 0xff000000;
    o.fUOverW = u / z;
    o.fVOverW = v / z;
    return o;
}

/* A box from lo to hi as twelve triangles for sgltri_shadow. Each is wound so that its normal
 * (v1 - v0) x (v2 - v0) in screen (x, y, z) points out of the box, as the driver's planes need
 * (wrappers/sgl/src/scene/shadow.rs). */
static void box_volume(const float lo[3], const float hi[3], SGLVERTEX bv[8], int faces[12][3]) {
    static const int quads[6][4] = {{0, 1, 3, 2}, {4, 5, 7, 6}, {0, 1, 5, 4}, {2, 3, 7, 6}, {0, 2, 6, 4}, {1, 3, 7, 5}};
    float centre[3] = {0, 0, 0};
    for (int i = 0; i < 8; i++) {
        bv[i] = project(i & 1 ? hi[0] : lo[0], i & 2 ? hi[1] : lo[1], i & 4 ? hi[2] : lo[2], 0, 0, 0);
        centre[0] += bv[i].fX / 8; centre[1] += bv[i].fY / 8; centre[2] += bv[i].fZ / 8;
    }
    for (int q = 0; q < 6; q++)
        for (int t = 0; t < 2; t++) {
            int *f = faces[q * 2 + t];
            f[0] = quads[q][0]; f[1] = quads[q][t + 1]; f[2] = quads[q][t + 2];
            SGLVERTEX *a = &bv[f[0]], *b = &bv[f[1]], *c = &bv[f[2]];
            float e1[3] = {b->fX - a->fX, b->fY - a->fY, b->fZ - a->fZ};
            float e2[3] = {c->fX - a->fX, c->fY - a->fY, c->fZ - a->fZ};
            float n[3] = {e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]};
            float m[3] = {(a->fX + b->fX + c->fX) / 3 - centre[0], (a->fY + b->fY + c->fY) / 3 - centre[1], (a->fZ + b->fZ + c->fZ) / 3 - centre[2]};
            if (n[0] * m[0] + n[1] * m[1] + n[2] * m[2] < 0) { int k = f[1]; f[1] = f[2]; f[2] = k; }
        }
}

int main(int argc, char **argv) {
    int frames = argc > 1 ? atoi(argv[1]) : 120;
    int ddraw = argc > 2 && !strcmp(argv[2], "ddraw");
    lib = LoadLibraryA("sgl.dll");
    if (!lib) { fprintf(stderr, "no sgl.dll\n"); return 1; }
    L(sgl_use_address_mode); L(sgl_use_ddraw_mode); L(sgl_use_eor_callback); L(sgl_create_screen_device);
    L(sgl_preprocess_texture); L(sgl_create_texture); L(sgltri_startofframe); L(sgltri_triangles);
    L(sgltri_quads); L(sgltri_shadow); L(sgltri_render); L(sgltri_isrendercomplete); L(sgl_get_errors); L(sgl_get_versions);
    L(sgl_get_win_versions); L(sgl_get_free_texture_mem); L(sgl_delete_all_textures); L(sgl_wrapper_present);

    WNDCLASSA wc = {0};
    wc.lpfnWndProc = proc;
    wc.hInstance = GetModuleHandleA(0);
    wc.lpszClassName = "SglTest";
    RegisterClassA(&wc);
    RECT r = {0, 0, W * 2, H * 2};
    AdjustWindowRect(&r, WS_OVERLAPPEDWINDOW, 0);
    HWND hwnd = CreateWindowA("SglTest", "sgltritest", WS_OVERLAPPEDWINDOW | WS_VISIBLE, 0, 0, r.right - r.left, r.bottom - r.top, 0, 0, wc.hInstance, 0);

    void *status;
    if (ddraw) sgl_use_ddraw_mode(hwnd, (void *)draw_2d);
    else sgl_use_address_mode((void *)next_address, &status);
    sgl_use_eor_callback((void *)end_of_render);
    sgl_create_screen_device(0, W, H, 0, 1);

    /* A 64 x 64 checker, red and yellow, as in sgltest.c. */
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
    unsigned long free_before = sgl_get_free_texture_mem();

    SGLCONTEXT c;
    memset(&c, 0, sizeof c);
    c.cBackgroundColour[0] = 0.1f; c.cBackgroundColour[1] = 0.15f; c.cBackgroundColour[2] = 0.3f;
    c.fFogR = 0.5f; c.fFogG = 0.5f; c.fFogB = 0.6f;
    c.u32FogDensity = 10; /* about 97% of the colour kept at z = 2, 33% at z = 66 (scene/tri.rs `rate`) */
    c.bCullBackfacing = 1;
    c.u32GlobalTrans = 255;

    SGLVERTEX v[64];
    int quads[8][4], tri[1][3] = {{0, 1, 2}};
    for (int i = 0; i < frames; i++) {
        float a = i * 0.05f;
        sgltri_startofframe(&c);

        /* A grey floor, eight rows of quads from z = 2 to z = 66, fogged. */
        for (int k = 0; k <= 8; k++) {
            float z = 2 + k * 8;
            v[k * 2] = project(-8, -1.5f, z, 0xff808080, 0, 0);
            v[k * 2 + 1] = project(8, -1.5f, z, 0xff808080, 0, 0);
        }
        for (int k = 0; k < 8; k++) {
            quads[k][0] = k * 2 + 2; quads[k][1] = k * 2 + 3; quads[k][2] = k * 2 + 1; quads[k][3] = k * 2;
        }
        c.u32Flags = 0; c.bFogOn = 1; c.nTextureName = 0;
        /* In shadow mode, with a shadow's brightness of 0.4. */
        c.eShadowLightVolMode = 1; c.u.fShadowBrightness = 0.4f;
        sgltri_quads(&c, 8, quads, v);
        c.bFogOn = 0;

        /* A shadow box through the floor in the middle, a yellow light box on the right. */
        static const float slo[3] = {-3, -2, 7}, shi[3] = {1, -1, 11};
        static const float llo[3] = {3, -2, 10}, lhi[3] = {6, -1, 18};
        SGLVERTEX bv[8];
        int bf[12][3];
        box_volume(slo, shi, bv, bf);
        sgltri_shadow(&c, 12, bf, bv, NULL);
        box_volume(llo, lhi, bv, bf);
        c.eShadowLightVolMode = 2; c.u.u32LightVolColour = 0x00606000;
        sgltri_shadow(&c, 12, bf, bv, NULL);
        c.eShadowLightVolMode = 0; c.u.u32LightVolColour = 0;

        /* A Gouraud triangle on the left: red, green and blue corners. */
        v[0] = project(-3.5f, -1, 6, 0xffff0000, 0, 0);
        v[1] = project(-2, 1.5f, 6, 0xff00ff00, 0, 0);
        v[2] = project(-0.5f, -1, 6, 0xff0000ff, 0, 0);
        c.u32Flags = SGLTT_GOURAUD;
        sgltri_triangles(&c, 1, tri, v);

        /* The checker on a square of side 2 that turns about the vertical axis at z = 6. */
        float ca = cosf(a), sa = sinf(a);
        static const float s[4][2] = {{-1, -1}, {-1, 1}, {1, 1}, {1, -1}};
        for (int q = 0; q < 4; q++)
            v[q] = project(1.5f + s[q][0] * ca, s[q][1], 6 + s[q][0] * sa, 0xffffffff, (s[q][0] + 1) / 2, (1 - s[q][1]) / 2);
        int one[1][4] = {{0, 1, 2, 3}};
        c.u32Flags = SGLTT_TEXTURE | SGLTT_GOURAUD; c.nTextureName = tex;
        c.bCullBackfacing = 0;
        sgltri_quads(&c, 1, one, v);
        c.bCullBackfacing = 1; c.nTextureName = 0;

        /* A flat orange triangle up right with a white highlight on its corners. */
        v[0] = project(2, 2, 7, 0xffff8000, 0, 0);
        v[1] = project(3.5f, 3, 7, 0xff000000, 0, 0);
        v[2] = project(4, 1.5f, 7, 0xff000000, 0, 0);
        for (int q = 0; q < 3; q++) v[q].u32Specular = 0xff606060;
        c.u32Flags = SGLTT_HIGHLIGHT;
        sgltri_triangles(&c, 1, tri, v);

        /* A white triangle in front at half opacity, over the Gouraud triangle and the floor. */
        v[0] = project(-4, -1.4f, 4, 0x80ffffff, 0, 0);
        v[1] = project(-2.5f, 0.5f, 4, 0x80ffffff, 0, 0);
        v[2] = project(-1, -1.4f, 4, 0x80ffffff, 0, 0);
        c.u32Flags = SGLTT_VERTEXTRANS;
        sgltri_triangles(&c, 1, tri, v);

        sgltri_render(&c);
        while (sgltri_isrendercomplete(&c, 0) != 0) Sleep(0);
        if (!ddraw) {
            bar(frame, W * 2);
            sgl_wrapper_present(hwnd, frame, W, H, W * 2, 16);
        }
        MSG m;
        while (PeekMessageA(&m, 0, 0, 0, PM_REMOVE)) { TranslateMessage(&m); DispatchMessageA(&m); }
        Sleep(16);
    }

    int earliest, recent;
    sgl_get_errors(&earliest, &recent);
    sgl_versions *ver = sgl_get_versions();
    sgl_win_versions *win = sgl_get_win_versions();
    sgl_delete_all_textures();
    printf("renders %d, 2D callbacks %d, errors %d %d\n", renders, pictures, earliest, recent);
    printf("library %s, header %s, mode %s\n", ver->library, ver->required_header, win->s[10]);
    printf("free texture memory %lu, after deleting all %lu\n", free_before, sgl_get_free_texture_mem());
    Sleep(500);
    return 0;
}
