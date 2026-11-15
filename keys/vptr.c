// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

// Makes a virtual pointer with wlroots' virtual-pointer protocol, which
// sway offers even when it has no input device, moves it to a point of the
// output, and presses and releases each button named, a second apart.  The
// pointer lives from the start to the end of the program, since wlrctl's
// pointer ends too soon for a program to bind it and get its buttons.
//
// Usage: vptr X Y WIDTH HEIGHT BUTTON..., with BUTTON one of left, side,
// extra, forward, and back, which are the codes of the Linux input
// subsystem
#include <linux/input-event-codes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <wayland-client.h>

#include "wlr-virtual-pointer-unstable-v1-client-protocol.h"

static struct wl_seat *seat;
static struct zwlr_virtual_pointer_manager_v1 *manager;

static void global(void *data, struct wl_registry *registry, uint32_t name, const char *interface, uint32_t version) {
    if (!strcmp(interface, wl_seat_interface.name)) {
        seat = wl_registry_bind(registry, name, &wl_seat_interface, 1);
    } else if (!strcmp(interface, zwlr_virtual_pointer_manager_v1_interface.name)) {
        manager = wl_registry_bind(registry, name, &zwlr_virtual_pointer_manager_v1_interface, 1);
    }
}

static void global_remove(void *data, struct wl_registry *registry, uint32_t name) {}

static const struct wl_registry_listener listener = {global, global_remove};

static uint32_t code(const char *name) {
    if (!strcmp(name, "left")) return BTN_LEFT;
    if (!strcmp(name, "side")) return BTN_SIDE;
    if (!strcmp(name, "extra")) return BTN_EXTRA;
    if (!strcmp(name, "forward")) return BTN_FORWARD;
    if (!strcmp(name, "back")) return BTN_BACK;
    fprintf(stderr, "vptr: no button %s\n", name);
    exit(2);
}

int main(int argc, char **argv) {
    if (argc < 6) {
        fprintf(stderr, "usage: vptr X Y WIDTH HEIGHT BUTTON...\n");
        return 2;
    }
    struct wl_display *display = wl_display_connect(NULL);
    if (!display) {
        fprintf(stderr, "vptr: no display\n");
        return 1;
    }
    wl_registry_add_listener(wl_display_get_registry(display), &listener, NULL);
    wl_display_roundtrip(display);
    if (!seat || !manager) {
        fprintf(stderr, "vptr: no seat or no virtual pointer manager\n");
        return 1;
    }
    struct zwlr_virtual_pointer_v1 *pointer = zwlr_virtual_pointer_manager_v1_create_virtual_pointer(manager, seat);
    wl_display_roundtrip(display);
    // Time for cinq to bind the pointer that has just come to the seat
    sleep(2);
    zwlr_virtual_pointer_v1_motion_absolute(pointer, 0, atoi(argv[1]), atoi(argv[2]), atoi(argv[3]), atoi(argv[4]));
    zwlr_virtual_pointer_v1_frame(pointer);
    wl_display_roundtrip(display);
    sleep(1);
    for (int i = 5; i < argc; i++) {
        uint32_t button = code(argv[i]);
        zwlr_virtual_pointer_v1_button(pointer, 0, button, WL_POINTER_BUTTON_STATE_PRESSED);
        zwlr_virtual_pointer_v1_frame(pointer);
        wl_display_roundtrip(display);
        usleep(80000);
        zwlr_virtual_pointer_v1_button(pointer, 0, button, WL_POINTER_BUTTON_STATE_RELEASED);
        zwlr_virtual_pointer_v1_frame(pointer);
        wl_display_roundtrip(display);
        printf("vptr %s 0x%x sent\n", argv[i], button);
        fflush(stdout);
        sleep(1);
    }
    zwlr_virtual_pointer_v1_destroy(pointer);
    wl_display_roundtrip(display);
    return 0;
}
