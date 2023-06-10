use std::{path::PathBuf, sync::Mutex};

use adw::{gtk::ContentFit, prelude::*, StatusPage};
use gst::{
    glib::MainContext,
    prelude::{ElementExtManual, PadExtManual},
    traits::{ElementExt, GstBinExt},
    Pipeline,
};
use relm4::{
    gtk::{
        gdk_pixbuf::Pixbuf,
        glib,
        glib::{clone, prelude::*},
        Box as GtkBox, Picture, Stack,
    },
    prelude::*,
    ComponentParts,
};

use crate::{
    preferences::PreferencesModel,
    slave::{async_glib::Future, video_ext::VideoSource},
};

use super::video_ext::*;
use super::{async_glib::Promise, config::SlaveConfigModel};

pub struct SlaveVideoInit {
    pub preferences: PreferencesModel,
    pub config: SlaveConfigModel,
}

#[tracker::track]
#[derive(Debug)]
pub struct SlaveVideoModel {
    #[no_eq]
    pub record_handle: Option<((gst::Element, gst::Pad), Vec<gst::Element>)>,
    #[no_eq]
    pub pixbuf: Option<Pixbuf>,
    pipeline: Option<Pipeline>,
    #[no_eq]
    slave_config: SlaveConfigModel,
    preferences: PreferencesModel,
}

#[derive(Debug)]
pub enum SlaveVideoInput {
    StartPipeline,
    StopPipeline,
    SetPixbuf(Option<Pixbuf>),
    StartRecord(PathBuf),
    StopRecord(Option<Promise<()>>),
    UpdatePreferences(PreferencesModel),
    UpdateConfig(SlaveConfigModel),
    SaveScreenshot(PathBuf),
    RequestFrame,
}

#[derive(Debug)]
pub enum SlaveVideoOutput {
    PollingChanged(bool),
    RecordingChanged(bool),
    ErrorMessage(String),
    ShowToastMessage(String),
}

#[relm4::component(pub)]
impl SimpleComponent for SlaveVideoModel {
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
                    #[track = "model.changed(SlaveVideoModel::slave_config())"]
                    set_content_fit: if *model.slave_config.get_keep_video_display_ratio() {ContentFit::Contain} else {ContentFit::Fill},
                    #[track = "model.changed(SlaveVideoModel::pixbuf())"]
                    set_pixbuf: match &model.pixbuf {
                        Some(pixbuf) => Some(&pixbuf),
                        None => None,
                    },
                },
            },
        }
    }

    type Init = SlaveVideoInit;
    type Input = SlaveVideoInput;
    type Output = SlaveVideoOutput;

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = SlaveVideoModel {
            preferences: init.preferences,
            slave_config: init.config,
            pixbuf: None,
            pipeline: None,
            record_handle: None,
            tracker: 0,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        use SlaveVideoInput::*;
        match message {
            StartPipeline => {
                assert!(self.pipeline == None);
                let config = self.get_slave_config();
                let video_url = config.get_video_url();
                if let Some(video_source) = VideoSource::from_url(video_url) {
                    let video_decoder = config.get_video_decoder().clone();
                    let colorspace_conversion = config.get_colorspace_conversion().clone();
                    let use_decodebin = config.get_use_decodebin().clone();
                    let appsink_leaky_enabled = config.get_appsink_queue_leaky_enabled().clone();
                    let latency = config.get_video_latency().clone();

                    match if use_decodebin {
                        super::video::create_decodebin_pipeline(video_source, appsink_leaky_enabled)
                    } else {
                        super::video::create_pipeline(
                            video_source,
                            latency,
                            colorspace_conversion,
                            video_decoder,
                            appsink_leaky_enabled,
                        )
                    } {
                        Ok(pipeline) => {
                            let sender = sender.clone();
                            let (mat_sender, mat_receiver) =
                                MainContext::channel(glib::PRIORITY_DEFAULT);
                            super::video::attach_pipeline_callback(
                                &pipeline,
                                mat_sender,
                                self.get_slave_config(),
                            )
                            .unwrap();
                            {
                                let sender = sender.clone();
                                mat_receiver.attach(None, move |mat| {
                                    sender.input(SlaveVideoInput::SetPixbuf(Some(mat.as_pixbuf())));
                                    Continue(true)
                                });
                            }
                            match pipeline.set_state(gst::State::Playing) {
                                Ok(_) => {
                                    self.set_pipeline(Some(pipeline));
                                    sender
                                        .output(SlaveVideoOutput::PollingChanged(true))
                                        .unwrap();
                                }
                                Err(_) => {
                                    sender.output(SlaveVideoOutput::ErrorMessage(String::from("无法启动管道，这可能是由于管道使用的资源不存在或被占用导致的，请检查相关资源是否可用。"))).unwrap();
                                    sender
                                        .output(SlaveVideoOutput::PollingChanged(false))
                                        .unwrap();
                                }
                            }
                        }
                        Err(msg) => {
                            sender
                                .output(SlaveVideoOutput::ErrorMessage(String::from(msg)))
                                .unwrap();
                            sender
                                .output(SlaveVideoOutput::PollingChanged(false))
                                .unwrap();
                        }
                    }
                } else {
                    sender
                        .output(SlaveVideoOutput::ErrorMessage(String::from(
                            "拉流 URL 有误，请检查并修改后重试。",
                        )))
                        .unwrap();
                    sender
                        .output(SlaveVideoOutput::PollingChanged(false))
                        .unwrap();
                }
            }
            StopPipeline => {
                assert!(self.pipeline != None);
                let mut futures = Vec::<Future<()>>::new();
                let recording = self.is_recording();
                if recording {
                    let promise = Promise::new();
                    let future = promise.future();
                    sender.input(SlaveVideoInput::StopRecord(Some(promise)));
                    futures.push(future);
                }
                let promise = Promise::new();
                futures.push(promise.future());
                let promise = Mutex::new(Some(promise));
                if let Some(pipeline) = self.pipeline.take() {
                    let sinkpad = pipeline
                        .by_name("display")
                        .unwrap()
                        .static_pad("sink")
                        .unwrap();
                    sinkpad.add_probe(gst::PadProbeType::EVENT_BOTH, move |_pad, info| match &info
                        .data
                    {
                        Some(gst::PadProbeData::Event(event)) => {
                            if let gst::EventView::Eos(_) = event.view() {
                                promise.lock().unwrap().take().unwrap().success(());
                                gst::PadProbeReturn::Remove
                            } else {
                                gst::PadProbeReturn::Pass
                            }
                        }
                        _ => gst::PadProbeReturn::Pass,
                    });
                    if pipeline.current_state() == gst::State::Playing
                        && pipeline.send_event(gst::event::Eos::new())
                    {
                        // Future::sequence(futures.into_iter()).for_each(
                        //     clone!(@weak pipeline,@strong sender => move |_| {
                        //         sender.output(SlaveVideoOutput::PollingChanged(false));
                        //         pipeline.set_state(gst::State::Null).unwrap();
                        //     }),
                        // );
                        glib::timeout_add_local_once(
                            self.preferences.get_pipeline_timeout().clone(),
                            clone!(@weak pipeline,@strong sender => move || {
                                sender.output(SlaveVideoOutput::PollingChanged(false)).unwrap();
                                if recording {
                                    sender.output(SlaveVideoOutput::RecordingChanged(false)).unwrap();
                                }
                                sender.output(SlaveVideoOutput::ShowToastMessage(String::from("等待管道响应超时，已将其强制终止。"))).unwrap();
                                pipeline.set_state(gst::State::Null).unwrap();
                            }),
                        );
                    } else {
                        sender
                            .output(SlaveVideoOutput::PollingChanged(false))
                            .unwrap();
                        sender
                            .output(SlaveVideoOutput::RecordingChanged(false))
                            .unwrap();
                        pipeline.set_state(gst::State::Null).unwrap();
                    }
                }
            }
            SetPixbuf(pixbuf) => {
                if self.get_pixbuf().is_none() {
                    sender
                        .output(SlaveVideoOutput::PollingChanged(true))
                        .unwrap(); // 主要是更新截图按钮的状态
                }
                self.set_pixbuf(pixbuf)
            }
            StartRecord(pathbuf) => {
                let config = self.slave_config.clone();
                if let Some(pipeline) = &self.pipeline {
                    let encoder = if *config.get_reencode_recording_video() {
                        Some(config.get_video_encoder())
                    } else {
                        None
                    };
                    let colorspace_conversion = config.get_colorspace_conversion().clone();
                    let record_handle = match encoder {
                        Some(encoder) => {
                            let elements = encoder.gst_record_elements(
                                colorspace_conversion,
                                &pathbuf.to_str().unwrap(),
                            );
                            let elements_and_pad = elements.and_then(|elements| {
                                super::video::connect_elements_to_pipeline(
                                    pipeline,
                                    "tee_decoded",
                                    &elements,
                                )
                                .map(|pad| (elements, pad))
                            });
                            elements_and_pad
                        }
                        None => {
                            let elements = config
                                .video_decoder
                                .gst_record_elements(&pathbuf.to_str().unwrap());
                            let elements_and_pad = elements.and_then(|elements| {
                                super::video::connect_elements_to_pipeline(
                                    pipeline,
                                    "tee_source",
                                    &elements,
                                )
                                .map(|pad| (elements, pad))
                            });
                            elements_and_pad
                        }
                    };
                    match record_handle {
                        Ok((elements, pad)) => {
                            self.record_handle = Some((pad, Vec::from(elements)));
                            sender
                                .output(SlaveVideoOutput::RecordingChanged(true))
                                .unwrap();
                        }
                        Err(err) => {
                            sender
                                .output(SlaveVideoOutput::ErrorMessage(err.to_string()))
                                .unwrap();
                            sender
                                .output(SlaveVideoOutput::RecordingChanged(false))
                                .unwrap();
                        }
                    }
                }
            }
            StopRecord(_promise) => {
                // if let Some(pipeline) = &self.pipeline {
                //     if let Some((teepad, elements)) = &self.record_handle {
                //         // super::video::disconnect_elements_to_pipeline(pipeline, teepad, elements).unwrap().for_each(clone!(@strong parent_sender => move |_| {
                //         //     send!(parent_sender, SlaveMsg::RecordingChanged(false));
                //         //     if let Some(promise) = promise {
                //         //         promise.success(());
                //         //     }
                //         // }));
                //     }
                //     self.set_record_handle(None);
                // }
            }
            UpdateConfig(config) => self.set_slave_config(config),
            UpdatePreferences(preferences) => self.set_preferences(preferences),
            SaveScreenshot(pathbuf) => {
                assert!(self.pixbuf != None);
                if let Some(pixbuf) = &self.pixbuf {
                    let format = pathbuf
                        .extension()
                        .unwrap()
                        .to_str()
                        .and_then(ImageFormat::from_extension)
                        .unwrap();
                    match pixbuf.savev(&pathbuf, &format.to_string().to_lowercase(), &[]) {
                        Ok(_) => sender
                            .output(SlaveVideoOutput::ShowToastMessage(format!(
                                "截图保存成功：{}",
                                pathbuf.to_str().unwrap()
                            )))
                            .unwrap(),
                        Err(err) => sender
                            .output(SlaveVideoOutput::ShowToastMessage(format!(
                                "截图保存失败：{}",
                                err.to_string()
                            )))
                            .unwrap(),
                    };
                }
            }
            RequestFrame => {}
        }
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
