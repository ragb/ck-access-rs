#!/usr/bin/env python3
"""Reliable single-shot Live Set snapshot + diff (no polling, no tee buffering).

    python snap.py save  baseline.json   --port "CK Series-1"
    python snap.py diff   baseline.json   --port "CK Series-1"   # snap now & diff

`diff` takes a fresh snapshot and prints every block/offset that changed vs the
saved baseline. Use it to map parameters: save, change ONE thing, diff.
"""
import argparse
import json
import sys
import time

import mido


def req(a):
    return mido.Message.from_bytes([0xF0, 0x43, 0x20, 0x7F, 0x1C, 0x0B, *a, 0xF7])


def find(names, needle):
    for n in names:
        if needle.lower() in n.lower():
            return n
    raise SystemExit(f"no port matching {needle!r}")


def snapshot(port):
    inp = mido.open_input(find(mido.get_input_names(), port))
    outp = mido.open_output(find(mido.get_output_names(), port))
    list(inp.iter_pending())
    outp.send(req((0x0E, 0, 0)))
    blocks = {}
    d = time.time() + 3.0
    while time.time() < d:
        for m in inp.iter_pending():
            if m.type == "sysex":
                r = [0xF0, *m.data, 0xF7]
                if len(r) > 13 and r[1] == 0x43:
                    blocks["%02X %02X %02X" % (r[8], r[9], r[10])] = list(r[11:-2])
                    d = time.time() + 0.5
                    if r[8] == 0x0F:
                        d = time.time()
        time.sleep(0.005)
    inp.close()
    outp.close()
    return blocks


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("mode", choices=["save", "diff"])
    ap.add_argument("file")
    ap.add_argument("--port", required=True)
    args = ap.parse_args()

    cur = snapshot(args.port)
    if args.mode == "save":
        json.dump(cur, open(args.file, "w"))
        print(f"saved {len(cur)} blocks to {args.file}")
        return

    base = json.load(open(args.file))
    any_change = False
    for addr in sorted(set(base) | set(cur)):
        b = base.get(addr, [])
        c = cur.get(addr, [])
        for i in range(max(len(b), len(c))):
            o = b[i] if i < len(b) else None
            n = c[i] if i < len(c) else None
            if o != n:
                any_change = True
                print(f"{addr} [offset 0x{i:02X}]  {o} -> {n}")
    if not any_change:
        print("(no changes vs baseline)", file=sys.stderr)


if __name__ == "__main__":
    main()
