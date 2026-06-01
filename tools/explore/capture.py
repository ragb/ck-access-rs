#!/usr/bin/env python3
"""Passively log every SysEx message arriving from the CK.

Turn knobs / change settings on the device and watch the Parameter Change and
Bulk Dump messages it emits — the quickest way to learn an address or confirm a
value mapping. Ctrl-C to stop.

Usage:
    python capture.py --port CK
    python capture.py --port CK --save captures/session1
"""
import argparse
import time

import mido


def find_port(names, needle):
    for n in names:
        if needle.lower() in n.lower():
            return n
    raise SystemExit(f"no MIDI input matching {needle!r} in {names}")


def describe(data):
    # data excludes F0/F7. CK: 43 <kind|dev> 7F 1C ...
    if len(data) >= 5 and data[0] == 0x43 and data[2:4] == (0x7F, 0x1C):
        kind = {0x00: "BulkDump", 0x10: "ParamChange", 0x20: "BulkReq", 0x30: "ParamReq"}.get(
            data[1] & 0xF0, "?"
        )
        return f"CK {kind} dev={data[1] & 0x0F}"
    if len(data) >= 3 and data[0] == 0x7E and data[2] == 0x06:
        return "Universal Identity"
    return "SysEx"


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--port", required=True, help="MIDI input port substring")
    ap.add_argument("--save", help="append captures to this .jsonl path (without extension)")
    args = ap.parse_args()

    in_name = find_port(mido.get_input_names(), args.port)
    print(f"listening on {in_name} (Ctrl-C to stop)")
    out = open(args.save + ".jsonl", "a") if args.save else None

    with mido.open_input(in_name) as inp:
        try:
            for msg in inp:
                if msg.type != "sysex":
                    continue
                data = msg.data
                hexs = " ".join(f"{b:02X}" for b in (0xF0, *data, 0xF7))
                print(f"[{describe(data)}] {hexs}")
                if out:
                    out.write(f'{{"t": {time.time()}, "bytes": "{hexs}"}}\n')
                    out.flush()
        except KeyboardInterrupt:
            print("\nstopped")
        finally:
            if out:
                out.close()


if __name__ == "__main__":
    main()
