# 2mpeg4yuv

![thumbnail](./thumbnail.png)

Simple playground project to explore the [YUV4MPEG2](https://wiki.multimedia.cx/index.php?title=YUV4MPEG2) format.

## Quick Start

Install the [Rust Compiler](https://www.rust-lang.org/)

### Preview

```console
$ ./build.sh
$ ./2mpeg4yuv preview
```

**WARNING! The preview may potentially produce loud clipping sounds!**

### Render

```console
$ ./render.sh
$ mplayer output.mp4
```

**WARNING! The render generates files up to 1.3GB! Make sure you have enough disk space**
