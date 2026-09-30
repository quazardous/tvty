#!/usr/bin/env python3
"""The Windows setup, tried whole in a fresh Windows Sandbox.

    WBOX_PYTHON scripts/sandbox_install_test.py [--from FOLDER]

Boots a sandbox with a network (dev/tvty-wbox-win/install.yaml), and runs
scripts/sandbox/install-test.ps1 in it: winget, then
packaging/windows/tvty-setup.ps1 as a user runs it, then what is there and
a picture of the whole desktop. `--from` gives it a local build's release
files (target/distrib) instead of the published release.

What it leaves in dev/tvty-wbox-win/home/install-test: out.txt, setup.log,
desk.png. `--aiball-ref` installs aiball at a tag or a branch rather than
its latest release, to try a fix of it. Run under wbox's own interpreter, as scripts/wbox_ctl.py is.
The sandbox is left up, to look further; one sandbox runs at a time, so
any other is closed first.
"""

import argparse
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from wbox.compositor.wsb import Channel, SandboxCompositor

ROOT = Path(__file__).resolve().parent.parent
CONFIG = ROOT / "dev/tvty-wbox-win/install.yaml"
WORK = ROOT / "dev/tvty-wbox-win/home/install-test"
INSTANCE = "tvty-wbox-win"
RELEASE_FILES = ("*.ps1", "*.zip", "*.sha256", "sha256.sum")
STEP = ("===", "github:", "winget:", "tvty-setup:", "shortcuts:", "tvty:", "aiball:", "aiball at:", "daemon:", "processes:", "task:", "vbs left:", "desk.png:")


def transcript() -> str:
    out = WORK / "out.txt"
    return out.read_bytes().decode("utf-8-sig", "replace").replace("\r", "") if out.exists() else ""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--from", dest="source", help="a folder of release files (target/distrib)")
    parser.add_argument("--aiball-ref", help="a tag or a branch of aiball to install, instead of its latest release")
    parser.add_argument("--minutes", type=int, default=30, help="how long to wait for it")
    args = parser.parse_args()

    # A fresh Windows: whatever sandbox is up goes — first, as it holds the
    # folder it shares.
    SandboxCompositor(instance_name=INSTANCE)._shutdown()
    shutil.rmtree(WORK, ignore_errors=True)
    WORK.mkdir(parents=True, exist_ok=True)
    for left in WORK.iterdir():
        raise SystemExit(f"{left} cannot be removed: something still holds it")
    shutil.copy(ROOT / "scripts/sandbox/install-test.ps1", WORK)
    shutil.copy(ROOT / "packaging/windows/tvty-setup.ps1", WORK)
    if args.source:
        (WORK / "release").mkdir()
        for pattern in RELEASE_FILES:
            for file in Path(args.source).glob(pattern):
                shutil.copy(file, WORK / "release")

    if args.aiball_ref:
        (WORK / "aiball-ref.txt").write_text(args.aiball_ref)

    ctl = [sys.executable, str(ROOT / "scripts/wbox_ctl.py")]
    # `up` boots the sandbox, then starts the config's program (a debug tvty):
    # only the boot matters here, so a build that is not there is no failure.
    subprocess.run([*ctl, "up", str(CONFIG)], check=False, stdout=subprocess.DEVNULL)
    subprocess.run([*ctl, "down", str(CONFIG)], check=False, stdout=subprocess.DEVNULL)
    if not Channel(Path(tempfile.gettempdir()) / f"wbox_{INSTANCE}_sandbox").ready():
        raise SystemExit("the sandbox did not boot")

    channel = Channel(Path(tempfile.gettempdir()) / f"wbox_{INSTANCE}_sandbox")
    command = ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", r"C:\tvty-home\install-test\install-test.ps1"]
    try:
        # It opens no window: wbox waits for one in vain, the script runs.
        channel.call("launch", {"app_cmd": command}, timeout=15)
    except Exception:
        pass

    said = 0
    deadline = time.monotonic() + args.minutes * 60
    while time.monotonic() < deadline:
        lines = [line for line in transcript().split("\n") if line.startswith(STEP)]
        for line in lines[said:]:
            print(line[:240], flush=True)
        said = len(lines)
        if (WORK / "done").exists():
            print(f"the rest is in {WORK}")
            return 0 if "tvty-setup: done." in transcript() else 1
        time.sleep(5)
    print(f"not done after {args.minutes} minutes; what there is, in {WORK}")
    return 2


if __name__ == "__main__":
    sys.exit(main())
