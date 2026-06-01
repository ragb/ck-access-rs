# tools/explore

Throwaway Python for reverse-engineering / verifying the CK SysEx protocol
against real hardware. Not part of the Rust build.

```
python -m venv .venv
. .venv/bin/activate          # or .venv\Scripts\activate on Windows
pip install -r requirements.txt

python probe.py --list                 # list MIDI ports
python probe.py --port CK              # identity + dump every block to captures/
python capture.py --port CK           # live-log SysEx as you tweak the device
```

`captures/` is gitignored. Once a capture is confirmed correct, copy it into
`ck-core/tests/fixtures/` and add a byte-exact round-trip test (see
`../../ck-core/tests/fixtures/README.md`).

On the device: set **MIDI In/Out = On**, **USB In/Out** as appropriate, and make
sure the **MIDI Device Number** matches `--device` (default 0).
