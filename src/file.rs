use crate::StreamError;
use crate::{FrameData, VideoStream, VideoStreamIterator};
use std::path::{Path, PathBuf};

pub struct FileSource {
    source: PathBuf,
    frame_size: (u32, u32),
}

impl FileSource {
    pub fn new(source: &Path, frame_size: (u32, u32)) -> Result<Self, CaptureError> {
        if !source.exists() {
            return Err(CaptureError::FileNotFound);
        }
        Ok(Self {
            source: source.to_path_buf(),
            frame_size,
        })
    }
}

impl IntoIterator for FileSource {
    type Item = Result<Option<FrameData>, StreamError>;
    type IntoIter = VideoStreamIterator;

    fn into_iter(self) -> Self::IntoIter {
        let pipeline_description = format!(
            "uridecodebin uri=file://{} ! videoconvert ! videoscale ! capsfilter caps=\"video/x-raw, width={}, height={}\"",
            self.source.to_string_lossy(),
            self.frame_size.0,
            self.frame_size.1
        );
        VideoStream::new(pipeline_description).into_iter()
    }
}

pub enum CaptureError {
    FileNotFound,
}
