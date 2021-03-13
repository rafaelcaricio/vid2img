# vid2img - Video to Image

vid2img is a Rust crate that allows the use of a video file as a collection of frame images. This crate exposes
a `FileSource` type that accepts a video file path and the desired size of the frames, then you can convert the
instance into a iterator (`.into_iter()`). On every iteration you will receive a video frame data encoded as PNG.


```rust
use std::path::Path;
use vid2img::FileSource;

fn main() {
    let file_path = Path::new("video.mp4");

    let frame_source = FileSource::new(file_path, (200, 200)).unwrap();
    for frame in frame_source.into_iter() {
        if let Ok(Some(png_img_data)) = frame {
            // do something with the image data here ...
        }
    }
}
```

We use [GStreamer](https://gstreamer.freedesktop.org/) for processing the video and capturing the frames. We make use
of the official [Rust wrapper](https://gitlab.freedesktop.org/gstreamer/gstreamer-rs) to the GStreamer API.

## Installation
As we use GStreamer, the [installation steps](https://gitlab.freedesktop.org/gstreamer/gstreamer-rs#installation) for the GStreamer-rs crate must be followed.

We make use of the following GStreamer plugins:
[`uridecodebin`](https://gstreamer.freedesktop.org/documentation/playback/uridecodebin.html), [`videoconvert`](https://gstreamer.freedesktop.org/documentation/videoconvert/index.html),
[`videoscale`](https://gstreamer.freedesktop.org/documentation/videoscale/index.html),
[`capsfilter`](https://gstreamer.freedesktop.org/documentation/coreelements/capsfilter.html),
[`pngenc`](https://gstreamer.freedesktop.org/documentation/png/pngenc.html),
[`appsrc`](https://gstreamer.freedesktop.org/documentation/app/appsrc.html).

### Usage

To use this library, add the following dependency to `Cargo.toml`:

```toml
[dependencies]
vid2img = "0.1.0"
```
