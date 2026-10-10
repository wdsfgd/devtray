#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

echo "Building release binary..."
cargo build --release

if command -v strip >/dev/null 2>&1; then
    echo "Stripping unused sections (.eh_frame, .comment, notes)..."
    strip --strip-all \
        --remove-section=.eh_frame \
        --remove-section=.eh_frame_hdr \
        --remove-section=.comment \
        --remove-section=.note.gnu.build-id \
        --remove-section=.annobin.notes \
        --remove-section=.note.ABI-tag \
        target/release/devtray
fi

echo "Done! Release executable size:"
ls -lh target/release/devtray
