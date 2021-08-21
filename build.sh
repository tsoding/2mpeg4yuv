#!/bin/sh

set -xe

rustc -C panic=abort -C opt-level=3 -o 2mpeg4yuv src/main.rs

