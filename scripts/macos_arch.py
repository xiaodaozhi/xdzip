"""Opt-in architecture for experimental Intel native CI; release remains arm64."""
import os

def macos_arch(env=None):
    return "x86_64" if (os.environ if env is None else env).get("XDZIP_MACOS_INTEL_CI") == "1" else "arm64"
