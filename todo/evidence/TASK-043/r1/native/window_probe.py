"""Bounded native-input smoke on a private Xvfb server; never opens the user's display."""
import ctypes as C
import hashlib
import json
import os
from pathlib import Path
import select
import subprocess
import time

ROOT = Path(__file__).resolve().parents[5]
OUT = Path(__file__).resolve().parent
x = C.CDLL("libX11.so.6")
t = C.CDLL("libXtst.so.6")
U, D, I = C.c_ulong, C.c_void_p, C.c_int


def bind(lib, name, result, args):
    fn = getattr(lib, name)
    fn.restype, fn.argtypes = result, args
    return fn


open_display = bind(x, "XOpenDisplay", D, [C.c_char_p])
close_display = bind(x, "XCloseDisplay", I, [D])
root_window = bind(x, "XDefaultRootWindow", U, [D])
query_tree = bind(x, "XQueryTree", I, [D, U, C.POINTER(U), C.POINTER(U), C.POINTER(C.POINTER(U)), C.POINTER(C.c_uint)])
fetch_name = bind(x, "XFetchName", I, [D, U, C.POINTER(D)])
free = bind(x, "XFree", I, [D])
focus = bind(x, "XSetInputFocus", I, [D, U, I, U])
sync = bind(x, "XSync", I, [D, I])
keysym = bind(x, "XStringToKeysym", U, [C.c_char_p])
keycode = bind(x, "XKeysymToKeycode", C.c_ubyte, [D, U])
fake_key = bind(t, "XTestFakeKeyEvent", I, [D, C.c_uint, I, U])
grab = bind(x, "XGrabPointer", I, [D, U, I, C.c_uint, I, I, U, U, U])
ungrab = bind(x, "XUngrabPointer", I, [D, U])
geometry = bind(x, "XGetGeometry", I, [D, U, C.POINTER(U), C.POINTER(I), C.POINTER(I), C.POINTER(C.c_uint), C.POINTER(C.c_uint), C.POINTER(C.c_uint), C.POINTER(C.c_uint)])
events, game, server, display = [], None, None, None
started = time.monotonic()


def record(name, **data):
    events.append({"seconds": round(time.monotonic() - started, 3), "check": name, **data})
    (OUT / "events.json").write_text(json.dumps(events, indent=2) + "\n")


def wait_for(predicate, label, seconds=20):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if game and game.poll() is not None:
            raise RuntimeError(f"game exited {game.returncode} while waiting for {label}")
        value = predicate()
        if value:
            return value
        time.sleep(.1)
    raise TimeoutError(label)


def find_game():
    r, p, n, children = U(), U(), C.c_uint(), C.POINTER(U)()
    query_tree(display, root, C.byref(r), C.byref(p), C.byref(children), C.byref(n))
    windows = [children[i] for i in range(n.value)]
    if children:
        free(children)
    for window in windows:
        name = D()
        if fetch_name(display, window, C.byref(name)) and name.value:
            title = C.string_at(name)
            free(name)
            if title == b"N:SIDE":
                return window


def tap(name):
    code = keycode(display, keysym(name.encode()))
    assert code, name
    fake_key(display, code, 1, 0)
    sync(display, 0)
    time.sleep(.2)
    fake_key(display, code, 0, 0)
    sync(display, 0)
    time.sleep(.3)


def pointer_status():
    status = grab(display, root, 0, 0, 1, 1, 0, 0, 0)
    if status == 0:
        ungrab(display, 0)
        sync(display, 0)
    return status


def pointer_check(name, locked):
    expected = 1 if locked else 0  # AlreadyGrabbed versus GrabSuccess
    wait_for(lambda: pointer_status() == expected, name)
    record(name, status="PASS", xgrab_status=expected)


def size():
    r, a, b, w, h, border, depth = U(), I(), I(), C.c_uint(), C.c_uint(), C.c_uint(), C.c_uint()
    assert geometry(display, window, C.byref(r), C.byref(a), C.byref(b), C.byref(w), C.byref(h), C.byref(border), C.byref(depth))
    return [w.value, h.value]


try:
    read_fd, write_fd = os.pipe()
    with (OUT / "xvfb.log").open("w") as log:
        server = subprocess.Popen(["Xvfb", "-displayfd", str(write_fd), "-screen", "0", "1920x1080x24", "-nolisten", "tcp"], pass_fds=[write_fd], stdout=log, stderr=subprocess.STDOUT)
    os.close(write_fd)
    assert select.select([read_fd], [], [], 10)[0], "Xvfb displayfd timeout"
    number = os.read(read_fd, 32).strip()
    os.close(read_fd)
    assert number.isdigit(), "Xvfb did not allocate an independent display"
    display_name = b":" + number
    display = open_display(display_name)
    assert display, "cannot connect to this probe's Xvfb"
    root = root_window(display)
    env = os.environ.copy()
    env.pop("WAYLAND_DISPLAY", None)
    env.update(DISPLAY=display_name.decode(), WINIT_UNIX_BACKEND="x11", RUST_LOG="info,n_side::app=debug")
    binary = ROOT / "game/target/debug/n-side"
    command = [str(binary), "--project-root", str(ROOT), "--walk-preview", "--settings-dir", str(OUT / "settings")]
    with (OUT / "runtime.log").open("w") as log:
        game = subprocess.Popen(command, env=env, stdout=log, stderr=subprocess.STDOUT)
    record("launch", display=display_name.decode(), pid=game.pid, command=command, binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
    window = wait_for(find_game, "native window", 30)
    focus(display, window, 2, 0)
    sync(display, 0)
    time.sleep(3)
    pointer_check("title_releases_pointer", False)
    assert size() == [1280, 720], size()
    record("initial_resolution", status="PASS", dimensions=size())
    tap("Return")
    wait_for(lambda: "[world/ready]" in (OUT / "runtime.log").read_text(), "world ready", 120)
    pointer_check("walking_locks_pointer", True)
    tap("m")
    pointer_check("m_releases_pointer", False)
    tap("m")
    pointer_check("m_relocks_pointer", True)
    tap("Escape")
    pointer_check("pause_releases_pointer", False)
    tap("Escape")
    pointer_check("resume_relocks_pointer", True)
    focus(display, root, 2, 0)
    sync(display, 0)
    pointer_check("focus_loss_releases_pointer", False)
    focus(display, window, 2, 0)
    sync(display, 0)
    time.sleep(.5)
    pointer_check("refocus_remains_paused", False)
    tap("Escape")
    pointer_check("explicit_resume_relocks_pointer", True)
    tap("Escape")
    pointer_check("settings_pause_releases_pointer", False)
    for key in ["Up", "Return", "e", "Next", "Next", "Next", "Next", "Down", "Down", "Return"]:
        tap(key)
    wait_for(lambda: size() == [1600, 900], "requested native 1600x900")
    record("resolution_1600x900", status="PASS", dimensions=size())
    tap("Left")
    wait_for(lambda: size() == [1280, 720], "restored native 1280x720")
    record("resolution_1280x720", status="PASS", dimensions=size())
    for key in ["Escape", "Up", "Return", "Down", "Return"]:
        tap(key)
    game.wait(timeout=15)
    assert game.returncode == 0, game.returncode
    record("normal_menu_exit", status="PASS", exit_code=game.returncode)
    record("summary", status="PASS", not_run="Borderless fullscreen and VSync presentation: private Xvfb has no window manager or physical monitor; physical input and subjective controls also not covered", screenshots="not required for X server state assertions")
except Exception as error:
    record("failure", status="FAIL", error=repr(error))
    raise
finally:
    if game and game.poll() is None:
        game.terminate()
        game.wait(timeout=10)
    if display:
        close_display(display)
    if server and server.poll() is None:
        server.terminate()
        server.wait(timeout=10)
    record("owned_process_cleanup", game_exit=game.returncode if game else None, xvfb_exit=server.returncode if server else None)
