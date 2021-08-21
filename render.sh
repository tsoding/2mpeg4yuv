#!/bin/sh

set -xe

./build.sh
./2mpeg4yuv render
ffmpeg -y -i output.y4m -f f32le -ar 48.0k -ac 1 -i output.pcm -map 0:v:0 -map 1:a:0 output.mp4
