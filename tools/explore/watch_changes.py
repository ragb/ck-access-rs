#!/usr/bin/env python3
"""Watch a full Live Set dump and report every byte that changes.

Reverse-engineering aid: run it, then change ONE parameter at a time on the CK.
Each change prints the block address and offset that moved, e.g.

    46 20 00 [offset 0x02]  0x40 -> 0x00

so you can map undocumented parameters (v1.10 Pan / Rotary Speaker, …).

Usage:
    python watch_changes.py --port "CK Series-1" --seconds 120
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
    """Request the full Live Set and return {address: data} for each block."""
    list(inp.iter_pending())
    outp.send(req((0x0E, 0, 0)))
    blocks = {}
    d = time.time() + timeout
    while time.time() < d:
        for m in inp.iter_pending():
            if m.type == "sysex":
                r = [0xF0, *m.data, 0xF7]
                if len(r) > 13 and r[1] == 0x43:
                    addr = (r[8], r[9], r[10])
                    blocks[addr] = r[11:-2]
                    if addr[0] == 0x0F:  # footer
                        return blocks
                d = time.time() + 0.4
        time.sleep(0.005)
    return blocks


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--port", required=True)
    ap.add_argument("--seconds", type=float, default=120)
    args = ap.parse_args()

    inp = mido.open_input(find(mido.get_input_names(), args.port))
    outp = mido.open_output(find(mido.get_output_names(), args.port))
    print("baseline… change ONE parameter at a time on the CK. (Ctrl-C to stop)")
    prev = snapshot(inp, outp)
    deadline = time.time() + args.seconds
    try:
        while time.time() < deadline:
            time.sleep(2.5)  # gentle: the CK is unreliable under rapid dump requests
            cur = snapshot(inp, outp)
            if not cur:
                continue  # device didn't answer this round; keep last good baseline
            for addr, data in cur.items():
                if addr not in prev:
                    continue  # only diff blocks present in both snapshots
                old = prev.get(addr, [])
                for i in range(max(len(old), len(data))):
                    o = old[i] if i < len(old) else None
                    n = data[i] if i < len(data) else None
                    if o != n:
                        a = " ".join(f"{x:02X}" for x in addr)
                        print(f"{a} [offset 0x{i:02X}]  {o!r} -> {n!r}", flush=True)
            prev = cur
    except KeyboardInterrupt:
        pass
    finally:
        inp.close()
        outp.close()
    print("done")


if __name__ == "__main__":
    main()
