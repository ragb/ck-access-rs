#!/usr/bin/env python3
"""Probe a Yamaha CK61/CK88 over MIDI and dump each parameter block to a .syx.

This is throwaway reverse-engineering scaffolding (mirrors the sibling repos).
It speaks the Yamaha protocol directly so you can capture *real* dumps and then
promote them into ``ck-core/tests/fixtures/`` with a byte-exact Rust test.

Usage:
    python probe.py --list
    python probe.py --port CK            # identity + dump every block
    python probe.py --port CK --block live_set_common

Group Number 7F 1C, Model ID 0B. Bulk Request: F0 43 2n 7F 1C 0B ah am al F7.
The device replies with a Bulk Dump: F0 43 0n 7F 1C bh bl 0B ah am al dd.. cc F7.
"""
import argparse
import os
import time

import mido

YAMAHA = 0x43
GROUP = (0x7F, 0x1C)
MODEL = 0x0B
CAPTURES = os.path.join(os.path.dirname(__file__), "captures")

# (name, (high, mid, low)) — from docs/sysex-notes.md.
BLOCKS = [
    ("system_common", (0x20, 0x00, 0x00)),
    ("master_eq", (0x20, 0x40, 0x00)),
    ("live_set_common", (0x46, 0x00, 0x00)),
    ("live_set_eq", (0x46, 0x40, 0x00)),
    ("audio_trigger_path", (0x46, 0x10, 0x00)),
    ("zone_1", (0x4A, 0x00, 0x00)),
    ("zone_2", (0x4A, 0x01, 0x00)),
    ("zone_3", (0x4A, 0x02, 0x00)),
    ("zone_4", (0x4A, 0x03, 0x00)),
    ("part_a", (0x50, 0x00, 0x00)),
    ("part_b", (0x50, 0x01, 0x00)),
    ("part_c", (0x50, 0x02, 0x00)),
]


def bulk_request(device, addr):
    return [0xF0, YAMAHA, 0x20 | (device & 0x0F), *GROUP, MODEL, *addr, 0xF7]


def identity_request():
    return [0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7]


def find_port(names, needle):
    for n in names:
        if needle.lower() in n.lower():
            return n
    raise SystemExit(f"no MIDI port matching {needle!r} in {names}")


def wait_sysex(inp, timeout=2.5):
    deadline = time.time() + timeout
    while time.time() < deadline:
        for msg in inp.iter_pending():
            if msg.type == "sysex":
                return [0xF0, *msg.data, 0xF7]
        time.sleep(0.005)
    return None


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--list", action="store_true", help="list MIDI ports and exit")
    ap.add_argument("--port", help="port name substring (in + out)")
    ap.add_argument("--device", type=int, default=0, help="device number 0..15")
    ap.add_argument("--block", help="only dump this block name")
    args = ap.parse_args()

    if args.list or not args.port:
        print("Inputs: ", mido.get_input_names())
        print("Outputs:", mido.get_output_names())
        if not args.port:
            return

    in_name = find_port(mido.get_input_names(), args.port)
    out_name = find_port(mido.get_output_names(), args.port)
    print(f"in:  {in_name}\nout: {out_name}")
    os.makedirs(CAPTURES, exist_ok=True)

    with mido.open_input(in_name) as inp, mido.open_output(out_name) as outp:
        # Identity.
        outp.send(mido.Message.from_bytes(identity_request()))
        reply = wait_sysex(inp)
        if reply:
            print("identity:", " ".join(f"{b:02X}" for b in reply))
            fam = tuple(reply[8:10]) if len(reply) >= 10 else None
            print("  model:", {(0x62, 0x06): "CK61", (0x63, 0x06): "CK88"}.get(fam, "?"))
        else:
            print("identity: no reply (check cabling / device number / MIDI In-Out=On)")

        blocks = BLOCKS if not args.block else [b for b in BLOCKS if b[0] == args.block]
        for name, addr in blocks:
            outp.send(mido.Message.from_bytes(bulk_request(args.device, addr)))
            reply = wait_sysex(inp)
            if reply is None:
                print(f"{name:20s} {addr}: no reply")
                continue
            path = os.path.join(CAPTURES, f"{name}.syx")
            with open(path, "wb") as f:
                f.write(bytes(reply))
            print(f"{name:20s} {addr}: {len(reply)} bytes -> {path}")


if __name__ == "__main__":
    main()
