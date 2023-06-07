use relm4::{
    gtk::{
        prelude::*, Align, Box as GtkBox, Entry, Inhibit, Label, Orientation, ScrolledWindow,
        Separator, SpinButton, StringList, Switch, Viewport,
    },
    ComponentParts, SimpleComponent,
};
use url::Url;

use crate::preferences::PreferencesModel;

use super::video_ex::{ColorspaceConversion, VideoAlgorithm, VideoDecoder, VideoEncoder};

#[tracker::track]
#[derive(Debug)]
pub struct SlaveConfigModel {
    polling: Option<bool>,
    connected: Option<bool>,
    pub slave_url: Url,
    pub video_url: Url,
    pub video_algorithms: Vec<VideoAlgorithm>,
    pub keep_video_display_ratio: bool,
    pub video_decoder: VideoDecoder,
    pub colorspace_conversion: ColorspaceConversion,
    pub swap_xy: bool,
    pub use_decodebin: bool,
    // pub video_encoder: VideoEncoder,
    pub reencode_recording_video: bool,
    pub appsink_queue_leaky_enabled: bool,
    pub video_latency: u32,
}

#[relm4::component(pub)]
impl SimpleComponent for SlaveConfigModel {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        GtkBox {

        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = SlaveConfigModel {
            polling: Some(false),
            connected: Some(false),
            slave_url: PreferencesModel::default().default_slave_url,
            video_url: PreferencesModel::default().default_video_url,
            video_algorithms: Vec::new(),
            keep_video_display_ratio: PreferencesModel::default().default_keep_video_display_ratio,
            video_decoder: PreferencesModel::default().default_video_decoder,
            colorspace_conversion: PreferencesModel::default().default_colorspace_conversion,
            swap_xy: false,
            use_decodebin: false,
            // video_encoder: false,
            reencode_recording_video: false,
            appsink_queue_leaky_enabled: PreferencesModel::default()
                .default_appsink_queue_leaky_enabled,
            video_latency: PreferencesModel::default().default_video_latency,
            tracker: 0,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _message: Self::Input, _sender: relm4::ComponentSender<Self>) {}
}
