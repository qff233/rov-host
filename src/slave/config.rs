

#[tracker::track]
pub struct SlaveConfigModel {
    polling: Option<bool>,
    connected: Option<bool>,
    // pub slave_url: Url,
    // pub video_url: Url,
    // pub video_algorithms: Vec<VideoAlgorithm>,
    pub keep_video_display_ratio: bool,
    // pub video_decoder: VideoDecoder,
    // pub colorspace_conversion: ColorspaceConversion,
    pub swap_xy: bool,
    pub use_decodebin: bool,
    // pub video_encoder: VideoEncoder,
    pub reencode_recording_video: bool,
    pub appsink_queue_leaky_enabled: bool,
    pub video_latency: u32,
}


