use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};

use adw::{gtk::ContentFit, prelude::*, StatusPage};
use gst::Pipeline;
use relm4::{
    gtk::{gdk_pixbuf::Pixbuf, Box as GtkBox, Picture, Stack},
    prelude::*,
    ComponentParts,
};

use crate::preferences::PreferencesModel;

use super::{async_glib::Promise, config::SlaveConfigModel};

pub struct SlaveVideoInit {
    preferences: Rc<RefCell<PreferencesModel>>,
    config: Arc<Mutex<SlaveConfigModel>>,
}

#[tracker::track]
#[derive(Debug)]
pub struct SlaveVideoModel {
    #[no_eq]
    pub pixbuf: Option<Pixbuf>,
    #[no_eq]
    pub pipeline: Option<Pipeline>,
    #[no_eq]
    pub config: Arc<Mutex<SlaveConfigModel>>,
    pub record_handle: Option<(gst::Element, gst::Pad, Vec<gst::Element>)>,
    pub preferences: Rc<RefCell<PreferencesModel>>,
}

#[derive(Debug)]
pub enum SlaveVideoMsg {
    StartPipeline,
    StopPipeline,
    SetPixbuf(Option<Pixbuf>),
    StartRecord(PathBuf),
    StopRecord(Option<Promise<()>>),
    ConfigUpdated(SlaveConfigModel),
    SaveScreenshot(PathBuf),
    RequestFrame,
}

#[relm4::component(pub)]
impl SimpleComponent for SlaveVideoModel {
    type Init = SlaveVideoInit;
    type Input = SlaveVideoMsg;
    type Output = ();

    view! {
        frame = GtkBox {
            append = &Stack {
                set_vexpand: true,
                set_hexpand: true,
                add_child = &StatusPage {
                    set_icon_name: Some("face-uncertain-symbolic"),
                    set_title: "无信号",
                    set_description: Some("请点击上方按钮启动视频拉流"),
                    #[track = "model.changed(SlaveVideoModel::pixbuf())"]
                    set_visible: model.pixbuf == None,
                },
                add_child = &Picture {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_can_shrink: true,
                    #[track = "model.changed(SlaveVideoModel::config())"]
                    set_content_fit: if *model.config.lock().unwrap().get_keep_video_display_ratio() {ContentFit::Contain} else {ContentFit::Fill},
                    #[track = "model.changed(SlaveVideoModel::pixbuf())"]
                    set_pixbuf: match &model.pixbuf {
                        Some(pixbuf) => Some(&pixbuf),
                        None => None,
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = SlaveVideoModel {
            preferences: init.preferences,
            config: init.config,
            pixbuf: None,
            pipeline: None,
            record_handle: None,
            tracker: 0
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

impl SlaveVideoModel {
    pub fn is_running(&self) -> bool {
        self.pipeline.is_some()
    }

    pub fn is_recording(&self) -> bool {
        self.record_handle.is_some()
    }
}
