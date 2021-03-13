use gst::gst_element_error;
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use std::sync::mpsc::{sync_channel, Receiver, TryRecvError, TrySendError};
use std::sync::Once;

pub type FrameData = Vec<u8>;

static INIT_GST: Once = Once::new();

pub struct VideoStream {
    pipeline_description: String,
}

impl VideoStream {
    pub fn new<S: AsRef<str>>(pipeline_description: S) -> Self {
        INIT_GST.call_once(|| {
            log::trace!("Initializing GStreamer..");
            gst::init().expect("Could not initialize GStreamer!");
        });
        Self {
            pipeline_description: String::from(pipeline_description.as_ref()),
        }
    }
}

#[derive(Debug)]
pub struct GstErrorMessage {
    pub src: String,
    pub error: String,
    pub debug: Option<String>,
    pub source: glib::Error,
}

#[derive(Debug)]
pub enum StreamError {
    GstError(GstErrorMessage),
    FrameCaptureError,
}

impl IntoIterator for VideoStream {
    type Item = Result<Option<FrameData>, StreamError>;
    type IntoIter = VideoStreamIterator;

    fn into_iter(self) -> Self::IntoIter {
        let (sender, receiver) = sync_channel(1);

        log::debug!("Creating GStreamer Pipeline..");
        let pipeline = gst::parse_launch(
            format!(
                "{} ! pngenc snapshot=false ! appsink name=sink",
                self.pipeline_description
            )
            .as_str(),
        )
        .expect("Pipeline description invalid, cannot create")
        .downcast::<gst::Pipeline>()
        .expect("Expected a gst::Pipeline");

        // Get access to the appsink element.
        let appsink = pipeline
            .get_by_name("sink")
            .expect("Sink element not found")
            .downcast::<gst_app::AppSink>()
            .expect("Sink element is expected to be an appsink!");

        appsink
            .set_property("sync", &false)
            .expect("Failed to disable gst pipeline sync");
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    // Pull the sample in question out of the appsink's buffer.
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let buffer_ref = sample.get_buffer().ok_or_else(|| {
                        gst_element_error!(
                            appsink,
                            gst::ResourceError::Failed,
                            ("Failed to get buffer from appsink")
                        );

                        if let Err(err) = sender.try_send(Err(StreamError::FrameCaptureError)) {
                            log::error!("Could not send message in stream: {}", err)
                        }

                        gst::FlowError::Error
                    })?;

                    // At this point, buffer is only a reference to an existing memory region somewhere.
                    // When we want to access its content, we have to map it while requesting the required
                    // mode of access (read, read/write).
                    // This type of abstraction is necessary, because the buffer in question might not be
                    // on the machine's main memory itself, but rather in the GPU's memory.
                    // So mapping the buffer makes the underlying memory region accessible to us.
                    // See: https://gstreamer.freedesktop.org/documentation/plugin-development/advanced/allocation.html
                    let buffer = buffer_ref.map_readable().map_err(|_| {
                        gst_element_error!(
                            appsink,
                            gst::ResourceError::Failed,
                            ("Failed to map buffer readable")
                        );

                        if let Err(err) = sender.try_send(Err(StreamError::FrameCaptureError)) {
                            log::error!("Could not send message in stream: {}", err)
                        }

                        gst::FlowError::Error
                    })?;
                    log::trace!("Frame extracted from pipeline");

                    match sender.try_send(Ok(Some(buffer.to_vec()))) {
                        Ok(_) => Ok(gst::FlowSuccess::Ok),
                        Err(TrySendError::Full(_)) => {
                            log::trace!("Channel is full, discarded frame");
                            Ok(gst::FlowSuccess::Ok)
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            log::debug!("Returning EOS in pipeline callback fn");
                            Err(gst::FlowError::Eos)
                        }
                    }
                })
                .build(),
        );

        let bus = pipeline
            .get_bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        pipeline
            .set_state(gst::State::Playing)
            .expect("Cannot start pipeline");
        log::info!("Pipeline started: {}", self.pipeline_description);

        VideoStreamIterator {
            description: self.pipeline_description,
            receiver,
            pipeline,
            bus,
        }
    }
}

pub struct VideoStreamIterator {
    description: String,
    receiver: Receiver<Result<Option<FrameData>, StreamError>>,
    pipeline: gst::Pipeline,
    bus: gst::Bus,
}

impl Iterator for VideoStreamIterator {
    type Item = Result<Option<FrameData>, StreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.receiver.try_recv() {
            Ok(event) => return Some(event),
            Err(TryRecvError::Empty) => {
                // Check if there are errors in the GStreamer pipeline itself.
                if let Some(msg) = self.bus.pop() {
                    use gst::MessageView;

                    match msg.view() {
                        MessageView::Eos(..) => {
                            // The End-of-stream message is posted when the stream is done, which in our case
                            // happens immediately after matching the slate image because we return
                            // gst::FlowError::Eos then.
                            return None;
                        }
                        MessageView::Error(err) => {
                            let error_msg = GstErrorMessage {
                                src: msg
                                    .get_src()
                                    .map(|s| String::from(s.get_path_string()))
                                    .unwrap_or_else(|| String::from("None")),
                                error: err.get_error().to_string(),
                                debug: err.get_debug(),
                                source: err.get_error(),
                            };
                            return Some(Err(StreamError::GstError(error_msg)));
                        }
                        _ => (),
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                log::debug!("The Pipeline channel is disconnected: {}", self.description);
                return None;
            }
        }
        // Nothing to report in this iteration.
        // Frames could not be captured, but there are no errors in the pipeline.
        Some(Ok(None))
    }
}

impl Drop for VideoStreamIterator {
    fn drop(&mut self) {
        if self.pipeline.set_state(gst::State::Null).is_err() {
            log::error!("Could not stop pipeline");
        }
        log::debug!("Pipeline stopped!");
    }
}
