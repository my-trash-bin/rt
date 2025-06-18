#!/bin/sh

set -e

cd "$(dirname "$0")"

bindgen include/jsonc.h --allowlist-function "jsonc_parse" --allowlist-function "jsonc_free" --allowlist-type "jsonc_value" -o src/bindings.rs
