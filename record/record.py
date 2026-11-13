# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Record videos of Digital running test programs on rv64_cpu.

For each program, a copy of the circuits enables EN, selects the program
with PROG, and marks its ROM as program memory, so that the start command
of the remote interface of Digital can start the simulation.  The script
starts Digital on an X display, records the display with ffmpeg, and
clocks the processor through the remote interface until DONE rises.

Usage: record.py <rv64_cpu> <out> <java> <Digital.jar> <program>...

The display is $DISPLAY, and each program is a number such as 22.
"""
import json
import os
import re
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time

PORT = 41114
WIDTH, HEIGHT = 1920, 1080
HALF_CYCLE = 0.05  # Seconds between clock edges
MAX_CYCLES = 2000

SETTINGS = '''<?xml version="1.0" encoding="utf-8"?>
<attributes>
  <entry>
    <string>openRemotePort</string>
    <boolean>true</boolean>
  </entry>
  <entry>
    <string>showTutorial</string>
    <boolean>false</boolean>
  </entry>
</attributes>
'''


def remote(command):
    """Send a command to the remote interface of Digital and return its
    answer, in the framing of DataOutputStream.writeUTF."""
    with socket.create_connection(('127.0.0.1', PORT), timeout=30) as s:
        data = command.encode()
        s.sendall(struct.pack('>H', len(data)) + data)
        size = struct.unpack('>H', recv(s, 2))[0]
        return recv(s, size).decode()


def recv(s, n):
    out = b''
    while len(out) < n:
        chunk = s.recv(n - len(out))
        if not chunk:
            raise OSError('connection closed')
        out += chunk
    return out


def set_default(dig, label, value):
    """Give the input called label a default value."""
    pattern = re.compile(r'(<elementName>In</elementName>\s*<elementAttributes>)(.*?)(</elementAttributes>)', re.S)
    for m in pattern.finditer(dig):
        if re.search(rf'<string>Label</string>\s*<string>{label}</string>', m.group(2)):
            body = re.sub(r'\s*<entry>\s*<string>Default</string>\s*<long>\d+</long>\s*</entry>', '', m.group(2))
            entry = f'''
        <entry>
          <string>Default</string>
          <long>{value}</long>
        </entry>'''
            return dig[:m.start()] + m.group(1) + body.rstrip() + entry + '\n      ' + m.group(3) + dig[m.end():]
    raise ValueError(label)


def mark_program_memory(dig, label):
    """Mark the ROM called label as program memory."""
    pattern = re.compile(r'(<elementName>ROM</elementName>\s*<elementAttributes>)(.*?)(</elementAttributes>)', re.S)
    for m in pattern.finditer(dig):
        if re.search(rf'<string>Label</string>\s*<string>{label}</string>', m.group(2)):
            entry = '''
        <entry>
          <string>isProgramMemory</string>
          <boolean>true</boolean>
        </entry>'''
            return dig[:m.start()] + m.group(1) + m.group(2).rstrip() + entry + '\n      ' + m.group(3) + dig[m.end():]
    raise ValueError(label)


def wait_window(timeout=90):
    end = time.time() + timeout
    while time.time() < end:
        res = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', 'Digital'],
                             capture_output=True, text=True)
        ids = res.stdout.split()
        if ids:
            return ids[-1]
        time.sleep(1)
    raise TimeoutError('no Digital window')


def wait_port(timeout=60):
    end = time.time() + timeout
    while time.time() < end:
        try:
            socket.create_connection(('127.0.0.1', PORT), timeout=2).close()
            return
        except OSError:
            time.sleep(1)
    raise TimeoutError('no remote port')


def record(repo, out, java, jar, prog):
    hexes = sorted(f for f in os.listdir(f'{repo}/progs/hex') if f.startswith(f'{prog:02d}-'))
    (image,) = hexes
    name = image[:-len('.hex')]
    work = tempfile.mkdtemp()
    shutil.copytree(f'{repo}/src', f'{work}/src')
    top = f'{work}/src/rv64_cpu.dig'
    with open(top) as f:
        dig = f.read()
    dig = set_default(set_default(dig, 'EN', 1), 'PROG', prog)
    with open(top, 'w') as f:
        f.write(dig)
    with open(f'{work}/src/inst-mem.dig') as f:
        mem = mark_program_memory(f.read(), name)
    with open(f'{work}/src/inst-mem.dig', 'w') as f:
        f.write(mem)

    log = open(f'{out}/{name}.log', 'w')
    digital = subprocess.Popen([java, '-jar', jar, top], stdout=log, stderr=subprocess.STDOUT)
    try:
        win = wait_window()
        time.sleep(3)
        subprocess.run(['xdotool', 'windowmove', win, '0', '0'], check=True)
        subprocess.run(['xdotool', 'windowsize', win, str(WIDTH), str(HEIGHT)], check=True)
        time.sleep(2)
        subprocess.run(['xdotool', 'windowfocus', '--sync', win], check=False)
        subprocess.run(['xdotool', 'key', '--window', win, 'F1'], check=False)
        time.sleep(2)
        wait_port()
        video = subprocess.Popen(
            ['ffmpeg', '-loglevel', 'error', '-y', '-f', 'x11grab', '-video_size', f'{WIDTH}x{HEIGHT}',
             '-framerate', '15', '-i', os.environ['DISPLAY'], '-c:v', 'libx264', '-preset', 'veryfast',
             '-pix_fmt', 'yuv420p', f'{out}/{name}.mp4'], stdin=subprocess.PIPE)
        time.sleep(2)
        print(name, 'start:', remote(f'start:{repo}/progs/hex/{image}'), flush=True)
        time.sleep(2)
        cycles, done = 0, False
        while cycles < MAX_CYCLES and not done:
            for _ in range(2):
                remote('clock')
                time.sleep(HALF_CYCLE)
            cycles += 1
            if cycles % 5 == 0:
                values = json.loads(remote('measure')[len('ok:'):])
                done = values.get('DONE') == 1
        values = json.loads(remote('measure')[len('ok:'):])
        print(name, 'cycles:', cycles, {k: hex(values[k]) for k in ('DONE', 'A0', 'A1', 'A2') if k in values},
              flush=True)
        time.sleep(3)
        subprocess.run(['ffmpeg', '-loglevel', 'error', '-y', '-f', 'x11grab', '-video_size',
                        f'{WIDTH}x{HEIGHT}', '-i', os.environ['DISPLAY'], '-frames:v', '1',
                        f'{out}/{name}.png'], check=True)
        video.communicate(b'q', timeout=60)
    finally:
        digital.terminate()
        try:
            digital.wait(timeout=20)
        except subprocess.TimeoutExpired:
            digital.kill()
        log.close()


def main():
    repo, out, java, jar = sys.argv[1:5]
    os.makedirs(out, exist_ok=True)
    with open(os.path.expanduser('~/.digital.cfg'), 'w') as f:
        f.write(SETTINGS)
    for prog in map(int, sys.argv[5:]):
        record(os.path.abspath(repo), os.path.abspath(out), java, jar, prog)


main()
