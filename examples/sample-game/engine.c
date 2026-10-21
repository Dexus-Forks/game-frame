/* The sample game's "engine" DLL: a random number generator and an allocator, exported by
 * name, the way many game engines ship a runtime DLL beside the game executable. */
#include <stdint.h>
#include <stdlib.h>

typedef struct Random { uint32_t state; } Random;

__declspec(dllexport) void rng_seed(Random *r, uint32_t seed) { r->state = seed; }

/* A linear congruential generator with the MSVC rand() constants. */
__declspec(dllexport) uint32_t rng_next(Random *r) {
    r->state = r->state * 214013u + 2531011u;
    return (r->state >> 16) & 0x7fff;
}

/* A float in [0, 1): note the order of operations, which a port must copy exactly. */
__declspec(dllexport) float rng_float(Random *r) {
    return (float)rng_next(r) * (1.0f / 32768.0f);
}

__declspec(dllexport) void *mem_alloc(size_t size) { return malloc(size); }
__declspec(dllexport) void mem_free(void *p) { free(p); }
