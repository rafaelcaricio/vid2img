//! Vid2img is a Rust crate that allows the use of a video file as a collection of frame images. This crate exposes
//! a [`vid2img::FileSource`] type that accepts a video file path and the desired size of the frames, then you can convert the
//! instance into a iterator (`.into_iter()`). On every iteration you will receive a video frame data encoded as PNG.
//!
//!
//! We use [GStreamer] for processing the video and capturing the frames. We make use
//! of the official [Rust wrapper] to the GStreamer API.
//!
//! [`vid2img::FileSource`]: FileSource
//! [GStreamer]: https://gstreamer.freedesktop.org/
//! [Rust wrapper]: https://gitlab.freedesktop.org/gstreamer/gstreamer-rs
//!
//! # Example usage
//!
//! ```no_run
//! use std::path::Path;
//! use vid2img::FileSource;
//!
//! fn main() {
//!     let file_path = Path::new("video.mp4");
//!
//!     let frame_source = FileSource::new(file_path, (200, 200)).unwrap();
//!     for frame in frame_source.into_iter() {
//!         if let Ok(Some(png_img_data)) = frame {
//!             // do something with the image data here ...
//!         }
//!     }
//! }
//! ```
mod file;
mod video_stream;

pub use video_stream::*;
pub use file::*;
