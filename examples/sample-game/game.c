/* The sample game: a tiny simulation that calls the engine DLL. It has the parts a real game
 * has and an agent must learn to find: a fixed-step update, a random number generator, a table
 * of constants, objects with a table of functions (a "vtable"), and a data file loader. */
#include <stdint.h>
#include <stdio.h>

typedef struct Random { uint32_t state; } Random;
__declspec(dllimport) void rng_seed(Random *r, uint32_t seed);
__declspec(dllimport) uint32_t rng_next(Random *r);
__declspec(dllimport) float rng_float(Random *r);
__declspec(dllimport) void *mem_alloc(size_t size);

/* Constants a port must read from the binary, not guess. */
__declspec(dllexport) const float growth_table[4] = {0.05f, 0.125f, 0.3333333f, 1.0f};

/* An object with a table of functions, as C++ compilers lay out virtual methods. */
typedef struct Entity Entity;
typedef struct EntityVtbl {
    void (*update)(Entity *self, float dt);
    float (*value)(const Entity *self);
} EntityVtbl;
struct Entity {
    const EntityVtbl *vtbl; /* +0x00 */
    float age;              /* +0x08 */
    int stage;              /* +0x0c */
};

static void tree_update(Entity *self, float dt) {
    self->age += dt * growth_table[self->stage];
    if (self->age >= 1.0f && self->stage < 3) {
        self->age -= 1.0f;
        self->stage += 1;
    }
}
static float tree_value(const Entity *self) { return (float)self->stage + self->age; }
__declspec(dllexport) const EntityVtbl tree_vtbl = {tree_update, tree_value};

__declspec(dllexport) Entity *tree_new(void) {
    Entity *e = (Entity *)mem_alloc(sizeof(Entity));
    e->vtbl = &tree_vtbl;
    e->age = 0.0f;
    e->stage = 0;
    return e;
}

/* A fixed-step simulation: `steps` updates of 1/60 s, with random rain that speeds growth.
 * Writes one float per step to `out`. The result depends on every random draw, in order. */
__declspec(dllexport) void simulate(uint32_t seed, int steps, float *out) {
    Random rng;
    rng_seed(&rng, seed);
    Entity *tree = tree_new();
    for (int i = 0; i < steps; i++) {
        float dt = 1.0f / 60.0f;
        if (rng_next(&rng) % 4 == 0) dt *= 1.0f + rng_float(&rng);
        tree->vtbl->update(tree, dt);
        out[i] = tree->vtbl->value(tree);
    }
}

/* A data file: magic "SMPL", a u32 count, then records of (u32 id, f32 weight, u8 name length,
 * name bytes). Returns the number of records, or -1 if the file is not a sample file. */
__declspec(dllexport) int load_records(const uint8_t *data, size_t size, float *weights) {
    if (size < 8 || data[0] != 'S' || data[1] != 'M' || data[2] != 'P' || data[3] != 'L') return -1;
    uint32_t count = *(const uint32_t *)(data + 4);
    size_t at = 8;
    for (uint32_t i = 0; i < count; i++) {
        if (at + 9 > size) return -1;
        weights[i] = *(const float *)(data + at + 4);
        at += 9 + data[at + 8];
    }
    return (int)count;
}

int main(void) {
    float out[600];
    simulate(1234, 600, out);
    printf("after 10 s: %f\n", out[599]);
    return 0;
}
