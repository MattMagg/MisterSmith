#!/usr/bin/env python3
"""Mem0 Platform setup and management for Mister Smith.

Thin shim that delegates to the centralized CLI in the mem0 repo.
Automatically passes the Mister Smith project config.

Usage:
  python3 scripts/mem0_setup.py <command> [args]
"""

import os
import sys
from pathlib import Path

# Central CLI location
CLI_PATH = Path(__file__).resolve().parent.parent.parent / "Repos" / "mem0" / "claude-code" / "cli.py"
CONFIG_PATH = Path(__file__).resolve().parent / "mem0_config.py"


def main():
    if not CLI_PATH.exists():
        print(f"ERROR: Central CLI not found at {CLI_PATH}")
        print("Expected: ~/Repos/mem0/claude-code/cli.py")
        sys.exit(1)

    # Build the command with --config pointing to our project config
    args = [
        sys.executable,
        str(CLI_PATH),
        "--config", str(CONFIG_PATH),
        "--cwd", str(Path(__file__).resolve().parent.parent),
    ] + sys.argv[1:]

    os.execv(sys.executable, args)


if __name__ == "__main__":
    main()
