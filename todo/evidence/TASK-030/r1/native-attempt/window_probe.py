"""Operate only this probe's native N:SIDE window and its own focus sink via X11.
The game runs normally; no ECS state or game results are written by this script.
"""
import ctypes as C
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path.cwd()
OUT = Path(sys.argv[1]).resolve()
OUT.mkdir(parents=True, exist_ok=False)
x = C.CDLL('libX11.so.6')
t = C.CDLL('libXtst.so.6')
U, D, I = C.c_ulong, C.c_void_p, C.c_int

def bind(lib, name, result, args):
    fn = getattr(lib, name); fn.restype = result; fn.argtypes = args; return fn

open_display = bind(x, 'XOpenDisplay', D, [C.c_char_p])
close_display = bind(x, 'XCloseDisplay', I, [D])
root_window = bind(x, 'XDefaultRootWindow', U, [D])
query_tree = bind(x, 'XQueryTree', I, [D,U,C.POINTER(U),C.POINTER(U),C.POINTER(C.POINTER(U)),C.POINTER(C.c_uint)])
fetch_name = bind(x, 'XFetchName', I, [D,U,C.POINTER(C.c_void_p)])
free = bind(x, 'XFree', I, [D])
get_focus = bind(x, 'XGetInputFocus', I, [D,C.POINTER(U),C.POINTER(I)])
set_focus = bind(x, 'XSetInputFocus', I, [D,U,I,U])
flush = bind(x, 'XSync', I, [D,I])
create = bind(x, 'XCreateSimpleWindow', U, [D,U,I,I,C.c_uint,C.c_uint,C.c_uint,U,U])
map_window = bind(x, 'XMapWindow', I, [D,U])
store_name = bind(x, 'XStoreName', I, [D,U,C.c_char_p])
destroy = bind(x, 'XDestroyWindow', I, [D,U])
keysym = bind(x, 'XStringToKeysym', U, [C.c_char_p])
keycode = bind(x, 'XKeysymToKeycode', C.c_ubyte, [D,U])
fake_key = bind(t, 'XTestFakeKeyEvent', I, [D,C.c_uint,I,U])
raise_window = bind(x, 'XRaiseWindow', I, [D,U])
send_event = bind(x, 'XSendEvent', I, [D,U,I,C.c_long,D])
atom = bind(x, 'XInternAtom', U, [D,C.c_char_p,I])
get_prop = bind(x, 'XGetWindowProperty', I, [D,U,U,C.c_long,C.c_long,I,U,C.POINTER(U),C.POINTER(I),C.POINTER(U),C.POINTER(U),C.POINTER(C.POINTER(C.c_ubyte))])

display = open_display(b':0')
if not display: raise RuntimeError('X11 :0 unavailable')
old_focus, old_revert = U(), I();get_focus(display,C.byref(old_focus),C.byref(old_revert))
root = root_window(display)
sink = create(display,root,0,0,16,16,0,0,0)
store_name(display,sink,b'N:SIDE owned focus probe');map_window(display,sink);flush(display,0)
started=time.monotonic();events=[];game=None;video=None;window=None;held=set()
niri_env=os.environ.copy();niri_env['NIRI_SOCKET']='/run/user/1000/niri.wayland-1.1356.sock'
def niri_windows():
    return json.loads(subprocess.check_output(['niri','msg','-j','windows'],env=niri_env,text=True))
old_niri_focus=next((w['id'] for w in niri_windows() if w.get('is_focused')),None)

def event(name, **data):
    events.append({'elapsed_seconds':round(time.monotonic()-started,3),'event':name,**data})
    (OUT/'events.json').write_text(json.dumps(events,indent=2)+'\n')

def children(w):
    r,p=U(),U();ptr=C.POINTER(U)();n=C.c_uint()
    if not query_tree(display,w,C.byref(r),C.byref(p),C.byref(ptr),C.byref(n)):return []
    result=[ptr[i] for i in range(n.value)]
    if ptr:free(ptr)
    return result

def title(w):
    name=C.c_void_p()
    if fetch_name(display,w,C.byref(name)) and name.value:
        value=C.string_at(name).decode(errors='replace');free(name);return value
    return ''

def find_new_game(initial):
    todo=[root]
    while todo:
        w=todo.pop()
        if title(w)=='N:SIDE':
            typ,fmt,n,left=U(),I(),U(),U();data=C.POINTER(C.c_ubyte)()
            get_prop(display,w,atom(display,b'_NET_WM_PID',0),0,1,0,6,C.byref(typ),C.byref(fmt),C.byref(n),C.byref(left),C.byref(data))
            pid=C.cast(data,C.POINTER(U))[0] if data and n.value else None
            if data:free(data)
            if pid==game.pid:return w
        todo.extend(children(w))
    return None

def all_windows():
    todo=[root];result=set()
    while todo:
        w=todo.pop();result.add(w);todo.extend(children(w))
    return result

def focus(w):
    assert w in (window,sink)
    target=next((n for n in niri_windows() if (n.get('pid')==game.pid if w==window else n.get('title')=='N:SIDE owned focus probe')),None)
    if target is None:raise RuntimeError('owned compositor window not found')
    subprocess.run(['niri','msg','action','focus-window','--id',str(target['id'])],env=niri_env,check=True,stdout=subprocess.DEVNULL)
    deadline=time.monotonic()+3
    while not any(n['id']==target['id'] and n.get('is_focused') for n in niri_windows()):
        if time.monotonic()>deadline:raise TimeoutError('compositor focus')
        time.sleep(.05)
    raise_window(display,w);set_focus(display,w,2,0);flush(display,0)
    actual=U();revert=I();get_focus(display,C.byref(actual),C.byref(revert))
    if actual.value!=w:raise RuntimeError(f'focus mismatch requested={w} actual={actual.value}')
    event('focus',target='game' if w==window else 'owned-sink',window=w)

def key(name,down):
    actual=U();revert=I();get_focus(display,C.byref(actual),C.byref(revert))
    if actual.value not in (window,sink):raise RuntimeError('refusing key outside owned windows')
    code=keycode(display,keysym(name.encode()));assert code
    fake_key(display,code,int(down),0);flush(display,0)
    if down:held.add(name)
    else:held.discard(name)
    event('key',key_name=name,down=down,target='game' if actual.value==window else 'owned-sink')

def tap(name):
    key(name,True);time.sleep(.2);key(name,False);time.sleep(.2)

def shot(name):
    path=OUT/(name+'.png')
    subprocess.run(['ffmpeg','-nostdin','-hide_banner','-loglevel','error','-f','x11grab','-draw_mouse','0','-window_id',str(window),'-i',':0','-frames:v','1','-update','1',str(path)],check=True,timeout=10)
    event('screenshot',path=path.name,sha256=hashlib.sha256(path.read_bytes()).hexdigest())

try:
    initial=all_windows()
    env=os.environ.copy();env.update(DISPLAY=':0',WINIT_UNIX_BACKEND='x11',RUST_LOG='info,n_side::app=debug')
    log=(OUT/'runtime.log').open('w')
    command=[str(ROOT/'game/target/debug/n-side'),'--project-root',str(ROOT),'--walk-preview']
    game=subprocess.Popen(command,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
    event('launch',command=command,pid=game.pid)
    deadline=time.monotonic()+30
    while window is None:
        if game.poll() is not None:raise RuntimeError('game exited during window creation')
        if time.monotonic()>deadline:raise TimeoutError('game window creation')
        window=find_new_game(initial);time.sleep(.1)
    focus(window);time.sleep(2);shot('01-title');tap('Return')
    deadline=time.monotonic()+15
    while '[game/state] loading' not in (OUT/'runtime.log').read_text():
        if time.monotonic()>deadline:raise TimeoutError('title confirm did not reach loading')
        time.sleep(.1)
    # Lose focus while loading. Only native OS focus changes, no game-state injection
    focus(sink)
    deadline=time.monotonic()+90
    while '[world/ready]' not in (OUT/'runtime.log').read_text():
        if game.poll() is not None:raise RuntimeError('game exited during loading')
        if time.monotonic()>deadline:raise TimeoutError('world ready')
        time.sleep(.2)
    time.sleep(2);shot('02-loaded-unfocused')
    video_log=(OUT/'video.log').open('w')
    video=subprocess.Popen(['ffmpeg','-nostdin','-hide_banner','-loglevel','error','-f','x11grab','-draw_mouse','0','-framerate','15','-window_id',str(window),'-i',':0','-t','20','-c:v','libx264','-pix_fmt','yuv420p',str(OUT/'video.mp4')],stdout=video_log,stderr=subprocess.STDOUT)
    # Return held in our other window: real server key-repeat must not continue the game
    key('Return',True);time.sleep(.1);focus(window);time.sleep(1.3);shot('03-held-return-refocus');key('Return',False)
    time.sleep(.3);shot('04-released-still-paused');tap('Return');time.sleep(.5)
    key('w',True);time.sleep(1);shot('05-moving');focus(sink);time.sleep(.5);key('w',False)
    shot('06-lost-focus');time.sleep(1);shot('07-unfocused-stable')
    focus(window);time.sleep(.5);shot('08-refocused-still-paused');tap('Return');time.sleep(.5)
    key('d',True);time.sleep(.8);key('d',False);shot('09-resumed-movement')
    # Fast native out/in pair; exact same-frame delivery is additionally tested by ECS tests
    focus(sink);focus(window);time.sleep(.5);shot('10-fast-refocus-paused');tap('Return');time.sleep(.5)
    tap('Escape');time.sleep(.3);tap('Down');tap('Return');time.sleep(.6);shot('11-title-cleanup')
    event('actions_finished')
    video.wait(timeout=25)
    if video.returncode:raise RuntimeError('video recording failed')
    event('video_saved',path='video.mp4')
    # Close this owned native window through the normal WM_DELETE_WINDOW path
    class ClientMessage(C.Structure):
        _fields_=[('type',I),('serial',U),('send_event',I),('display',D),('window',U),('message_type',U),('format',I),('data',C.c_long*5)]
    msg=ClientMessage(type=33,display=display,window=window,message_type=atom(display,b'WM_PROTOCOLS',0),format=32)
    msg.data[0]=atom(display,b'WM_DELETE_WINDOW',0)
    send_event(display,window,0,0,C.byref(msg));flush(display,0);game.wait(timeout=15)
    event('normal_exit',exit_code=game.returncode)
    if game.returncode:raise RuntimeError('game exit nonzero')
    report={'status':'AUTOMATION_COMPLETE','visual_review':'NOT RUN until actual images reviewed','command':command,'binary_sha256':hashlib.sha256(Path(command[0]).read_bytes()).hexdigest(),'game_exit_code':game.returncode,'video_exit_code':video.returncode,'scope':'native X11 focus and XTest keyboard; not physical keyboard/gamepad or cross-platform'}
    (OUT/'run.json').write_text(json.dumps(report,indent=2)+'\n')
finally:
    if window is not None:
        # Only release keys through the owned sink; never inject into another application
        focus(sink)
        for name in list(held):key(name,False)
    if video and video.poll() is None:video.terminate();video.wait(timeout=10)
    if game and game.poll() is None:os.killpg(game.pid,signal.SIGTERM);game.wait(timeout=10)
    if old_focus.value in (0,1):set_focus(display,old_focus.value,old_revert.value,0)
    else:
        # Restore only if the prior window still exists
        if old_focus.value in all_windows():set_focus(display,old_focus.value,old_revert.value,0)
    destroy(display,sink);flush(display,0);close_display(display)
    if old_niri_focus is not None and any(w['id']==old_niri_focus for w in niri_windows()):
        subprocess.run(['niri','msg','action','focus-window','--id',str(old_niri_focus)],env=niri_env,check=True,stdout=subprocess.DEVNULL)
