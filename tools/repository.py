"""Checkout root and SHA-256 identities shared by the maintenance scripts."""
from __future__ import annotations

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def digest(path) -> str:
    """SHA-256 hex digest of a file's contents."""
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def data_digest(data: bytes) -> str:
    """SHA-256 hex digest of bytes already in memory."""
    return hashlib.sha256(data).hexdigest()
