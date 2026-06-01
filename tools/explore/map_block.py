#!/usr/bin/env python3
"""Interactive mapper: name a parameter, change it on the CK, and learn its offset.

Self-driven (you control the timing), so it doesn't depend on polling luck. Use
it to finish mapping the v1.10 Rotary Speaker block (46 20 00) and Part SW Mode.

    python map_block.py --port "CK Series-1"

For each parameter:
  1. type its name (e.g. "rotary_a_balance") and press Enter,
  2. change ONLY that parameter on the device,
  3. press Enter again.
It re-dumps the whole Live Set, diffs against the previous dump, and prints the
block + offset(s) that moved. Type 'q' to finish; it prints a summary mapping.
"""
import argparse
import time

import mido


def req(addr):
    return mido.Message.from_bytes([0xF0, 0x43, 0x20, 0x7F, 0x1C, 0x0B, *addr, 0xF7])


def find(names, needle):
    for n in names:
        if needle.lower() in n.lower():
            return n
    raise SystemExit(f"no port matching {needle!r} in {names}")


def snapshot(inp, outp, timeout=2.0):
    list(inp.iter_pending())
    outp.send(req((0x0E, 0, 0)))
    blocks = {}
    d = time.time() + timeout
    while time.time() < d:
        for m in inp.iter_pending():
            if m.type == "sysex":
                r = [0xF0, *m.data, 0xF7]
                if len(r) > 13 and r[1] == 0x43:
                    blocks[(r[8], r[9], r[10])] = r[11:-2]
                    if r[8] == 0x0F:
                        return blocks
                d = time.time() + 0.4
        time.sleep(0.005)
    return blocks


def diff(prev, cur):
    out = []
    for addr, data in cur.items():
        old = prev.get(addr, [])
        for i in range(max(len(old), len(data))):
            o = old[i] if i < len(old) else None
            n = data[i] if i < len(data) else None
            if o != n:
                out.append((addr, i, o, n))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--port", required=True)
    args = ap.parse_args()
    inp = mido.open_input(find(mido.get_input_names(), args.port))
    outp = mido.open_output(find(mido.get_output_names(), args.port))

    mapping = {}
    print("Taking baseline dump…")
    prev = snapshot(inp, outp)
    print(f"  {len(prev)} blocks. Ready.\n")
    try:
        while True:
            name = input("Parameter name (or 'q' to finish): ").strip()
            if name.lower() in ("q", "quit", ""):
                break
            input(f"  Now change '{name}' on the CK, then press Enter…")
            cur = snapshot(inp, outp)
            changes = diff(prev, cur)
            if not changes:
                print("  (no change detected — try a bigger move)\n")
            else:
                for addr, off, o, n in changes:
                    a = " ".join(f"{x:02X}" for x in addr)
                    print(f"  {a} [offset 0x{off:02X}]  {o} -> {n}")
                    mapping.setdefault(name, []).append((a, off))
                print()
            prev = cur
    except (KeyboardInterrupt, EOFError):
        pass
    finally:
        inp.close()
        outp.close()

    print("\n=== mapping ===")
    for name, locs in mapping.items():
        for a, off in locs:
            print(f"{name:24s} {a} offset 0x{off:02X}")


if __name__ == "__main__":
    main()
