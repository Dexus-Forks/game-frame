/* winput: send keys, mouse moves and clicks to a Windows game from inside Wine (or Windows).
 *
 * Usage: winput.exe [--dry-run] STEP [STEP ...]
 *
 * Steps run in order:
 *   list                 print the visible top-level windows: handle, size and title
 *   window TITLE         find the first visible window whose title contains TITLE (any case),
 *                        restore it and bring it to the foreground; clicks are then relative to it
 *   key NAME[:MS]        press and release a key, held MS milliseconds (default 50)
 *   down NAME, up NAME   press or release a key, to hold it across other steps
 *   type TEXT            type letters, digits and spaces as keys; other characters (UTF-8,
 *                        up to U+FFFF) as Unicode input
 *   move DX DY           move the mouse by DX, DY mickeys (relative, as DirectInput reads it)
 *   click X Y [right]    click at X, Y: in the window's client area after `window`, else the screen
 *   wait MS              sleep MS milliseconds
 *
 * NAME is a letter, a digit, enter, esc, space, tab, backspace, shift, ctrl, alt, up, down,
 * left, right, home, end, pgup, pgdn, insert, delete, f1 to f12, or a scancode such as 0x1c.
 * Keys go as scancodes (KEYEVENTF_SCANCODE), which DirectInput reads. Arrows and the keys of
 * the navigation block are sent as extended keys.
 *
 * Why: on macOS a process needs the Accessibility permission to send input. Wine windows belong
 * to no application, so tools that drive the desktop cannot reach a game under Wine. Input sent
 * with SendInput inside the same Wine prefix needs neither
 * (docs/knowhow/wine-input-without-accessibility.md).
 *
 * --dry-run prints each input instead of sending it. A build for any other system than Windows
 * has only --dry-run, which is what tests/run.sh checks. It exits 1 when a window is not found or
 * an input is not sent, and 2 on a bad step. It does not read the screen or record input. It
 * does not move a game's own cursor when the game draws it from DirectInput's relative
 * movement. Use `move` for such a game.
 *
 * Build: tools/live/winput.py build (zig cc -target x86-windows-gnu ... -luser32).
 */
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#else
#include <unistd.h>
#endif

static int dry = 0;

struct key { const char *name; unsigned short scan; int ext; };
static const struct key KEYS[] = {
    {"esc", 0x01, 0}, {"1", 0x02, 0}, {"2", 0x03, 0}, {"3", 0x04, 0}, {"4", 0x05, 0}, {"5", 0x06, 0},
    {"6", 0x07, 0}, {"7", 0x08, 0}, {"8", 0x09, 0}, {"9", 0x0a, 0}, {"0", 0x0b, 0},
    {"backspace", 0x0e, 0}, {"tab", 0x0f, 0},
    {"q", 0x10, 0}, {"w", 0x11, 0}, {"e", 0x12, 0}, {"r", 0x13, 0}, {"t", 0x14, 0}, {"y", 0x15, 0},
    {"u", 0x16, 0}, {"i", 0x17, 0}, {"o", 0x18, 0}, {"p", 0x19, 0}, {"enter", 0x1c, 0}, {"ctrl", 0x1d, 0},
    {"a", 0x1e, 0}, {"s", 0x1f, 0}, {"d", 0x20, 0}, {"f", 0x21, 0}, {"g", 0x22, 0}, {"h", 0x23, 0},
    {"j", 0x24, 0}, {"k", 0x25, 0}, {"l", 0x26, 0}, {"shift", 0x2a, 0},
    {"z", 0x2c, 0}, {"x", 0x2d, 0}, {"c", 0x2e, 0}, {"v", 0x2f, 0}, {"b", 0x30, 0}, {"n", 0x31, 0},
    {"m", 0x32, 0}, {"alt", 0x38, 0}, {"space", 0x39, 0},
    {"f1", 0x3b, 0}, {"f2", 0x3c, 0}, {"f3", 0x3d, 0}, {"f4", 0x3e, 0}, {"f5", 0x3f, 0}, {"f6", 0x40, 0},
    {"f7", 0x41, 0}, {"f8", 0x42, 0}, {"f9", 0x43, 0}, {"f10", 0x44, 0}, {"f11", 0x57, 0}, {"f12", 0x58, 0},
    {"home", 0x47, 1}, {"up", 0x48, 1}, {"pgup", 0x49, 1}, {"left", 0x4b, 1}, {"right", 0x4d, 1},
    {"end", 0x4f, 1}, {"down", 0x50, 1}, {"pgdn", 0x51, 1}, {"insert", 0x52, 1}, {"delete", 0x53, 1},
};

static int find_key(const char *name, struct key *out) {
    char low[32];
    size_t i, n = strlen(name);
    if (n == 0 || n >= sizeof low) return 0;
    for (i = 0; i <= n; i++) low[i] = (char)tolower((unsigned char)name[i]);
    if (low[0] == '0' && low[1] == 'x' && n > 2) {
        char *end;
        unsigned long v = strtoul(low, &end, 16);
        if (*end || v == 0 || v > 0xff) return 0;
        out->name = name; out->scan = (unsigned short)v; out->ext = 0;
        return 1;
    }
    for (i = 0; i < sizeof KEYS / sizeof KEYS[0]; i++)
        if (strcmp(KEYS[i].name, low) == 0) { *out = KEYS[i]; return 1; }
    return 0;
}

static void pause_ms(int ms) {
    if (dry) { printf("wait %d\n", ms); return; }
#ifdef _WIN32
    Sleep((DWORD)ms);
#else
    usleep((useconds_t)ms * 1000);
#endif
}

#ifdef _WIN32
static HWND target = NULL;

static int send_one(INPUT *in) {
    return SendInput(1, in, sizeof *in) == 1;
}
#endif

static int key_event(struct key k, int up) {
    if (dry) { printf("key %s scancode 0x%02x%s %s\n", k.name, k.scan, k.ext ? " extended" : "", up ? "up" : "down"); return 1; }
#ifdef _WIN32
    {
        INPUT in; memset(&in, 0, sizeof in);
        in.type = INPUT_KEYBOARD;
        in.ki.wScan = k.scan;
        in.ki.dwFlags = KEYEVENTF_SCANCODE | (k.ext ? KEYEVENTF_EXTENDEDKEY : 0) | (up ? KEYEVENTF_KEYUP : 0);
        return send_one(&in);
    }
#else
    return 0;
#endif
}

static int unicode_char(unsigned short ch) {
    if (dry) { printf("unicode U+%04X\n", ch); return 1; }
#ifdef _WIN32
    {
        INPUT in[2]; memset(in, 0, sizeof in);
        in[0].type = in[1].type = INPUT_KEYBOARD;
        in[0].ki.wScan = in[1].ki.wScan = ch;
        in[0].ki.dwFlags = KEYEVENTF_UNICODE;
        in[1].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
        return SendInput(2, in, sizeof in[0]) == 2;
    }
#else
    return 0;
#endif
}

static int press(struct key k, int ms) {
    if (!key_event(k, 0)) return 0;
    pause_ms(ms);
    return key_event(k, 1);
}

static int move(int dx, int dy) {
    if (dry) { printf("move %d %d\n", dx, dy); return 1; }
#ifdef _WIN32
    {
        INPUT in; memset(&in, 0, sizeof in);
        in.type = INPUT_MOUSE;
        in.mi.dx = dx; in.mi.dy = dy;
        in.mi.dwFlags = MOUSEEVENTF_MOVE;
        return send_one(&in);
    }
#else
    return 0;
#endif
}

static int click(int x, int y, int right) {
    if (dry) { printf("click %d %d %s\n", x, y, right ? "right" : "left"); return 1; }
#ifdef _WIN32
    {
        POINT p = {x, y};
        INPUT in[3]; memset(in, 0, sizeof in);
        int w = GetSystemMetrics(SM_CXSCREEN), h = GetSystemMetrics(SM_CYSCREEN);
        if (target) ClientToScreen(target, &p);
        in[0].type = in[1].type = in[2].type = INPUT_MOUSE;
        in[0].mi.dx = (LONG)((p.x * 65535LL) / (w > 1 ? w - 1 : 1));
        in[0].mi.dy = (LONG)((p.y * 65535LL) / (h > 1 ? h - 1 : 1));
        in[0].mi.dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE;
        in[1].mi.dwFlags = right ? MOUSEEVENTF_RIGHTDOWN : MOUSEEVENTF_LEFTDOWN;
        in[2].mi.dwFlags = right ? MOUSEEVENTF_RIGHTUP : MOUSEEVENTF_LEFTUP;
        if (SendInput(1, &in[0], sizeof in[0]) != 1) return 0;
        Sleep(30);
        if (SendInput(1, &in[1], sizeof in[1]) != 1) return 0;
        Sleep(50);
        return SendInput(1, &in[2], sizeof in[2]) == 1;
    }
#else
    return 0;
#endif
}

#ifdef _WIN32
struct search { const char *title; HWND found; int list; };

static int contains(const char *hay, const char *needle) {
    size_t i, j, n = strlen(needle);
    for (i = 0; hay[i]; i++) {
        for (j = 0; j < n && hay[i + j] && tolower((unsigned char)hay[i + j]) == tolower((unsigned char)needle[j]); j++);
        if (j == n) return 1;
    }
    return n == 0;
}

static BOOL CALLBACK each_window(HWND w, LPARAM data) {
    struct search *s = (struct search *)data;
    char title[256];
    RECT r;
    if (!IsWindowVisible(w) || GetWindowTextA(w, title, sizeof title) <= 0) return TRUE;
    if (s->list) {
        GetWindowRect(w, &r);
        printf("%p  %ldx%ld  %s\n", (void *)w, r.right - r.left, r.bottom - r.top, title);
        return TRUE;
    }
    if (contains(title, s->title)) { s->found = w; return FALSE; }
    return TRUE;
}
#endif

static int list_windows(void) {
    if (dry) { printf("list\n"); return 1; }
#ifdef _WIN32
    {
        struct search s = {"", NULL, 1};
        EnumWindows(each_window, (LPARAM)&s);
        return 1;
    }
#else
    return 0;
#endif
}

static int window(const char *title) {
    if (dry) { printf("window %s\n", title); return 1; }
#ifdef _WIN32
    {
        struct search s = {title, NULL, 0};
        EnumWindows(each_window, (LPARAM)&s);
        if (!s.found) { fprintf(stderr, "winput: no visible window titled like \"%s\" (try: winput list)\n", title); return 0; }
        target = s.found;
        ShowWindow(target, SW_RESTORE);
        SetForegroundWindow(target);
        Sleep(200);
        return 1;
    }
#else
    return 0;
#endif
}

/* The next code point of UTF-8 text, and how many bytes it took; one byte for a bad sequence. */
static unsigned decode(const unsigned char *c, int *len) {
    if (c[0] >= 0xe0 && c[0] < 0xf0 && (c[1] & 0xc0) == 0x80 && (c[2] & 0xc0) == 0x80) {
        *len = 3; return ((c[0] & 0x0fu) << 12) | ((c[1] & 0x3fu) << 6) | (c[2] & 0x3fu);
    }
    if (c[0] >= 0xc0 && c[0] < 0xe0 && (c[1] & 0xc0) == 0x80) {
        *len = 2; return ((c[0] & 0x1fu) << 6) | (c[1] & 0x3fu);
    }
    *len = 1; return c[0];
}

static int type_text(const char *text) {
    const char *c;
    int len;
    for (c = text; *c; c += len) {
        len = 1;
        char name[2] = {(char)tolower((unsigned char)*c), 0};
        struct key k;
        if (*c == ' ') name[0] = 0;
        if (*c == ' ' ? find_key("space", &k) : (isalnum((unsigned char)*c) && find_key(name, &k))) {
            int shifted = isupper((unsigned char)*c), ok;
            struct key shift; find_key("shift", &shift);
            if (shifted && !key_event(shift, 0)) return 0;
            ok = press(k, 20);
            if (shifted && !key_event(shift, 1)) return 0;
            if (!ok) return 0;
        } else if (!unicode_char((unsigned short)decode((const unsigned char *)c, &len))) {
            return 0;
        }
        pause_ms(20);
    }
    return 1;
}

static int need(int argc, int i, int n, const char *step) {
    if (i + n < argc) return 1;
    fprintf(stderr, "winput: %s needs %d argument%s\n", step, n, n == 1 ? "" : "s");
    return 0;
}

int main(int argc, char **argv) {
    int i = 1;
    if (i < argc && strcmp(argv[i], "--dry-run") == 0) { dry = 1; i++; }
#ifndef _WIN32
    if (!dry) { fprintf(stderr, "winput: this build has only --dry-run; build it for Windows to send input\n"); return 2; }
#endif
    if (i >= argc || strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
        printf("usage: winput.exe [--dry-run] STEP ...\n"
               "steps: list | window TITLE | key NAME[:MS] | down NAME | up NAME | type TEXT |\n"
               "       move DX DY | click X Y [right] | wait MS\n");
        return i >= argc ? 2 : 0;
    }
    for (; i < argc; i++) {
        const char *s = argv[i];
        int ok = 1;
        if (strcmp(s, "list") == 0) ok = list_windows();
        else if (strcmp(s, "window") == 0) { if (!need(argc, i, 1, s)) return 2; ok = window(argv[++i]); }
        else if (strcmp(s, "key") == 0 || strcmp(s, "down") == 0 || strcmp(s, "up") == 0) {
            char name[32]; const char *colon; int ms = 50; struct key k;
            if (!need(argc, i, 1, s)) return 2;
            colon = strchr(argv[i + 1], ':');
            snprintf(name, sizeof name, "%.*s", colon ? (int)(colon - argv[i + 1]) : (int)strlen(argv[i + 1]), argv[i + 1]);
            if (colon) ms = atoi(colon + 1);
            if (!find_key(name, &k)) { fprintf(stderr, "winput: unknown key \"%s\"\n", name); return 2; }
            ok = s[0] == 'k' ? press(k, ms) : key_event(k, s[0] == 'u');
            i++;
        }
        else if (strcmp(s, "type") == 0) { if (!need(argc, i, 1, s)) return 2; ok = type_text(argv[++i]); }
        else if (strcmp(s, "move") == 0) { if (!need(argc, i, 2, s)) return 2; ok = move(atoi(argv[i + 1]), atoi(argv[i + 2])); i += 2; }
        else if (strcmp(s, "click") == 0) {
            int right = 0;
            if (!need(argc, i, 2, s)) return 2;
            if (i + 3 < argc && strcmp(argv[i + 3], "right") == 0) right = 1;
            ok = click(atoi(argv[i + 1]), atoi(argv[i + 2]), right);
            i += 2 + right;
        }
        else if (strcmp(s, "wait") == 0) { if (!need(argc, i, 1, s)) return 2; pause_ms(atoi(argv[++i])); }
        else { fprintf(stderr, "winput: unknown step \"%s\" (see --help)\n", s); return 2; }
        if (!ok) { fprintf(stderr, "winput: step \"%s\" failed\n", s); return 1; }
    }
    return 0;
}
