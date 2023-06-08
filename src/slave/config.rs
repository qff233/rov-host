use std::{rc::Rc, str::FromStr};

use relm4::{
    adw::{prelude::*, ActionRow, ComboRow, ExpanderRow, PreferencesGroup},
    gtk::{
        Align, Box as GtkBox, Entry, Inhibit, Label, Orientation, ScrolledWindow, Separator,
        SpinButton, StringList, Switch, Viewport,
    },
    ComponentParts, RelmWidgetExt, SimpleComponent,
};
use strum::IntoEnumIterator;
use url::Url;

use crate::preferences::PreferencesModel;

use super::video_ext::{
    ColorspaceConversion, VideoAlgorithm, VideoCodec, VideoCodecProvider, VideoDecoder,
    VideoEncoder,
};

#[tracker::track]
#[derive(Debug, Clone)]
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
    pub video_encoder: VideoEncoder,
    pub reencode_recording_video: bool,
    pub appsink_queue_leaky_enabled: bool,
    pub video_latency: u32,
}

#[derive(Debug)]
pub enum SlaveConfigMsg {
    UpdatePreferences(PreferencesModel),
    SetVideoUrl(Url),
    SetSlaveUrl(Url),
    SetKeepVideoDisplayRatio(bool),
    SetPolling(Option<bool>),
    SetConnected(Option<bool>),
    SetVideoAlgorithm(Option<VideoAlgorithm>),
    SetVideoDecoder(VideoDecoder),
    SetColorspaceConversion(ColorspaceConversion),
    SetVideoDecoderCodec(VideoCodec),
    SetVideoDecoderCodecProvider(VideoCodecProvider),
    SetSwapXY(bool),
    SetUsePlaybin(bool),
    SetVideoEncoderCodec(VideoCodec),
    SetVideoEncoderCodecProvider(VideoCodecProvider),
    SetReencodeRecordingVideo(bool),
    SetAppSinkQueueLeakyEnabled(bool),
    SetVideoLatency(u32),
}

#[relm4::component(pub)]
impl SimpleComponent for SlaveConfigModel {
    type Init = Rc<PreferencesModel>;
    type Input = SlaveConfigMsg;
    type Output = ();

    view! {
        GtkBox {
            add_css_class: "background",
            set_orientation: Orientation::Horizontal,
            set_hexpand: false,
            append = &Separator {
                set_orientation: Orientation::Horizontal,
            },
            append = &ScrolledWindow {
                set_width_request: 340,
                #[wrap(Some)]
                set_child = &Viewport {
                    #[wrap(Some)]
                    set_child = &GtkBox {
                        set_spacing: 20,
                        set_margin_all: 10,
                        set_orientation: Orientation::Vertical,
                        append = &PreferencesGroup {
                            #[track = "model.changed(SlaveConfigModel::connected())"]
                            set_sensitive: model.get_connected().eq(&Some(false)),
                            set_title: "通讯",
                            set_description: Some("设置下位机的通讯选项"),
                            add = &ActionRow {
                                set_title: "连接 URL",
                                set_subtitle: "连接下位机使用的 URL",
                                add_suffix = &Entry {
                                    set_text: model.get_slave_url().to_string().as_str(),
                                    set_width_request: 160,
                                    set_valign: Align::Center,
                                    connect_changed[sender] => move |entry| {
                                        if let Ok(url) = Url::from_str(&entry.text()) {
                                            sender.input(SlaveConfigMsg::SetSlaveUrl(url));
                                            entry.remove_css_class("error");
                                        } else {
                                            entry.add_css_class("error");
                                        }
                                    }
                                },
                            },
                        },
                        append = &PreferencesGroup {
                            set_title: "控制",
                            set_description: Some("调整机位控制选项"),
                            add = &ActionRow {
                                set_title: "交换 X/Y 轴",
                                set_subtitle: "若下位机规定的 X/Y 轴与上位机不一致，可以使用此选项进行交换",
                                add_suffix: swap_xy_switch = &Switch {
                                    #[track = "model.changed(SlaveConfigModel::swap_xy())"]
                                    set_active: *model.get_swap_xy(),
                                    set_valign: Align::Center,
                                    connect_state_set[sender] => move |_, state| {
                                        sender.input(SlaveConfigMsg::SetSwapXY(state));
                                        Inhibit(false)
                                    }
                                },
                                set_activatable_widget: Some(&swap_xy_switch),
                            },
                        },
                        append = &PreferencesGroup {
                            set_title: "画面",
                            set_description: Some("上位机端对画面进行的处理选项"),

                            add = &ActionRow {
                                set_title: "保持长宽比",
                                set_subtitle: "在改变窗口大小的时是否保持画面比例，这可能导致画面无法全屏",
                                add_suffix: default_keep_video_display_ratio_switch = &Switch {
                                    #[track = "model.changed(SlaveConfigModel::keep_video_display_ratio())"]
                                    set_active: *model.get_keep_video_display_ratio(),
                                    set_valign: Align::Center,
                                    connect_state_set[sender] => move |_, state| {
                                        sender.input(SlaveConfigMsg::SetKeepVideoDisplayRatio(state));
                                        Inhibit(false)
                                    }
                                },
                                set_activatable_widget: Some(&default_keep_video_display_ratio_switch),
                            },
                            add = &ComboRow {
                                set_title: "增强算法",
                                set_subtitle: "对画面使用的增强算法",
                                set_model: Some(&{
                                    let model = StringList::new(&[]);
                                    model.append("无");
                                    for value in VideoAlgorithm::iter() {
                                        model.append(&value.to_string());
                                    }
                                    model
                                }),
                                #[track = "model.changed(SlaveConfigModel::video_algorithms())"]
                                set_selected: VideoAlgorithm::iter().position(|x| model.video_algorithms.first().map_or_else(|| false, |y| *y == x)).map_or_else(|| 0, |x| x + 1) as u32,
                                connect_selected_notify[sender] => move |row| {
                                    sender.input(SlaveConfigMsg::SetVideoAlgorithm(if row.selected() > 0 { Some(VideoAlgorithm::iter().nth(row.selected().wrapping_sub(1) as usize).unwrap()) } else { None }));
                                }
                            }
                        },
                        append = &PreferencesGroup {
                            #[track = "model.changed(SlaveConfigModel::polling())"]
                            set_sensitive: model.get_polling().eq(&Some(false)),
                            set_title: "管道",
                            set_description: Some("配置视频流接收以及录制所使用的管道"),
                            add = &ActionRow {
                                set_title: "视频流 URL",
                                set_subtitle: "配置机位视频流的 URL",
                                add_suffix = &Entry {
                                    #[track = "model.changed(SlaveConfigModel::video_url())"]
                                    set_text: model.get_video_url().to_string().as_str(),
                                    set_valign: Align::Center,
                                    set_width_request: 160,
                                    connect_changed[sender] => move |entry| {
                                        if let Ok(url) = Url::from_str(&entry.text()) {
                                            sender.input(SlaveConfigMsg::SetVideoUrl(url));
                                            entry.remove_css_class("error");
                                        } else {
                                            entry.add_css_class("error");
                                        }
                                    }
                                },
                            },
                            add = &ActionRow {
                                set_title: "启用画面自动跳帧",
                                set_subtitle: "当机位画面与视频流延迟过大时，自动跳帧以避免延迟提升",
                                add_suffix: appsink_queue_leaky_enabled_switch = &Switch {
                                    #[track = "model.changed(SlaveConfigModel::appsink_queue_leaky_enabled())"]
                                    set_active: *model.get_appsink_queue_leaky_enabled(),
                                    set_valign: Align::Center,
                                    connect_state_set[sender] => move |_, state| {
                                        sender.input(SlaveConfigMsg::SetAppSinkQueueLeakyEnabled(state));
                                        Inhibit(false)
                                    }
                                },
                                set_activatable_widget: Some(&appsink_queue_leaky_enabled_switch),
                            },
                            add = &ExpanderRow {
                                set_title: "手动配置管道",
                                set_show_enable_switch: true,
                                set_expanded: !*model.get_use_decodebin(),
                                #[track = "model.changed(SlaveConfigModel::use_decodebin())"]
                                set_enable_expansion: !model.get_use_decodebin(),
                                connect_enable_expansion_notify[sender] => move |expander| {
                                    sender.input(SlaveConfigMsg::SetUsePlaybin(!expander.enables_expansion()));
                                },
                                add_row = &ActionRow {
                                    set_title: "接收缓冲区延迟",
                                    set_subtitle: "可以增加接收缓冲区延迟，牺牲视频的实时性来换取流畅度的提升",
                                    add_suffix = &SpinButton::with_range(0.0, 60000.0, 50.0) {
                                        #[track = "model.changed(SlaveConfigModel::video_latency())"]
                                        set_value: model.video_latency as f64,
                                        set_digits: 0,
                                        set_valign: Align::Center,
                                        set_can_focus: false,
                                        connect_value_changed[sender] => move |button| {
                                            sender.input(SlaveConfigMsg::SetVideoLatency(button.value() as u32));
                                        }
                                    },
                                    add_suffix = &Label {
                                        set_label: "毫秒",
                                    },
                                },
                                add_row = &ComboRow {
                                    set_title: "色彩空间转换",
                                    set_subtitle: "设置视频编解码、视频流显示要求的色彩空间转换所使用的硬件",
                                    set_model: Some(&{
                                        let model = StringList::new(&[]);
                                        for value in ColorspaceConversion::iter() {
                                            model.append(&value.to_string());
                                        }
                                        model
                                    }),
                                    #[track = "model.changed(SlaveConfigModel::colorspace_conversion())"]
                                    set_selected: ColorspaceConversion::iter().position(|x| x == model.colorspace_conversion).unwrap() as u32,
                                    connect_selected_notify[sender] => move |row| {
                                        sender.input(SlaveConfigMsg::SetColorspaceConversion(ColorspaceConversion::iter().nth(row.selected() as usize).unwrap()));
                                    }
                                },
                                add_row = &ComboRow {
                                    set_title: "解码器",
                                    set_subtitle: "解码视频流使用的解码器",
                                    set_model: Some(&{
                                        let model = StringList::new(&[]);
                                        for value in VideoCodec::iter() {
                                            model.append(&value.to_string());
                                        }
                                        model
                                    }),
                                    #[track = "model.changed(SlaveConfigModel::video_decoder())"]
                                    set_selected: VideoCodec::iter().position(|x| x == model.video_decoder.0).unwrap() as u32,
                                    connect_selected_notify[sender] => move |row| {
                                        sender.input(SlaveConfigMsg::SetVideoDecoderCodec(VideoCodec::iter().nth(row.selected() as usize).unwrap()))
                                    }
                                },
                                add_row = &ComboRow {
                                    set_title: "解码器接口",
                                    set_subtitle: "解码视频流使用的解码器接口",
                                    set_model: Some(&{
                                        let model = StringList::new(&[]);
                                        for value in VideoCodecProvider::iter() {
                                            model.append(&value.to_string());
                                        }
                                        model
                                    }),
                                    #[track = "model.changed(SlaveConfigModel::video_decoder())"]
                                    set_selected: VideoCodecProvider::iter().position(|x| x == model.video_decoder.1).unwrap() as u32,
                                    connect_selected_notify[sender] => move |row| {
                                        sender.input(SlaveConfigMsg::SetVideoDecoderCodecProvider(VideoCodecProvider::iter().nth(row.selected() as usize).unwrap()))
                                    },
                                },
                            },
                            add = &ExpanderRow {
                                set_title: "录制时重新编码",
                                set_show_enable_switch: true,
                                set_expanded: *model.get_reencode_recording_video(),
                                #[track = "model.changed(SlaveConfigModel::reencode_recording_video())"]
                                set_enable_expansion: *model.get_reencode_recording_video(),
                                connect_enable_expansion_notify[sender] => move |expander| {
                                    sender.input(SlaveConfigMsg::SetReencodeRecordingVideo(expander.enables_expansion()));
                                },
                                add_row = &ComboRow {
                                    set_title: "编码器",
                                    set_subtitle: "视频录制时使用的编码器",
                                    set_model: Some(&{
                                        let model = StringList::new(&[]);
                                        for value in VideoCodec::iter() {
                                            model.append(&value.to_string());
                                        }
                                        model
                                    }),
                                    #[track = "model.changed(SlaveConfigModel::video_encoder())"]
                                    set_selected: VideoCodec::iter().position(|x| x == model.video_encoder.0).unwrap() as u32,
                                    connect_selected_notify[sender] => move |row| {
                                        sender.input(SlaveConfigMsg::SetVideoEncoderCodec(VideoCodec::iter().nth(row.selected() as usize).unwrap()))
                                    }
                                },
                                add_row = &ComboRow {
                                    set_title: "编码器接口",
                                    set_subtitle: "视频录制时调用的编码器接口",
                                    set_model: Some(&{
                                        let model = StringList::new(&[]);
                                        for value in VideoCodecProvider::iter() {
                                            model.append(&value.to_string());
                                        }
                                        model
                                    }),
                                    #[track = "model.changed(SlaveConfigModel::video_encoder())"]
                                    set_selected:  VideoCodecProvider::iter().position(|x| x == model.video_encoder.1).unwrap() as u32,
                                    connect_selected_notify[sender] => move |row| {
                                        sender.input(SlaveConfigMsg::SetVideoEncoderCodecProvider(VideoCodecProvider::iter().nth(row.selected() as usize).unwrap()))
                                    }
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        preference: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = SlaveConfigModel {
            polling: Some(false),
            connected: Some(false),
            swap_xy: false,
            video_algorithms: Vec::new(),
            slave_url: preference.default_slave_url.clone(),
            video_url: preference.default_video_url.clone(),
            keep_video_display_ratio: preference.default_keep_video_display_ratio,
            video_decoder: preference.default_video_decoder.clone(),
            colorspace_conversion: preference.default_colorspace_conversion,
            use_decodebin: preference.default_use_decodebin,
            video_encoder: preference.default_video_encoder.clone(),
            reencode_recording_video: preference.default_reencode_recording_video,
            appsink_queue_leaky_enabled: preference.default_appsink_queue_leaky_enabled,
            video_latency: preference.default_video_latency,
            tracker: 0,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::ComponentSender<Self>) {
        self.reset();

        use SlaveConfigMsg::*;
        match message {
            UpdatePreferences(preference) => {
                self.keep_video_display_ratio = preference.default_keep_video_display_ratio;
                self.video_latency = preference.default_video_latency;
            }
            SetKeepVideoDisplayRatio(value) => self.set_keep_video_display_ratio(value),
            SetPolling(polling) => self.set_polling(polling),
            SetConnected(connected) => self.set_connected(connected),
            SetVideoAlgorithm(algorithm) => {
                self.get_mut_video_algorithms().clear();
                if let Some(algorithm) = algorithm {
                    self.get_mut_video_algorithms().push(algorithm);
                }
            }
            SetVideoDecoder(decoder) => self.set_video_decoder(decoder),
            SetColorspaceConversion(conversion) => self.set_colorspace_conversion(conversion),
            SetVideoUrl(url) => self.video_url = url,
            SetSlaveUrl(url) => self.slave_url = url,
            SetVideoDecoderCodec(codec) => self.get_mut_video_decoder().0 = codec,
            SetVideoDecoderCodecProvider(provider) => self.get_mut_video_decoder().1 = provider,
            SetSwapXY(swap) => self.set_swap_xy(swap),
            SetUsePlaybin(use_decodebin) => {
                if use_decodebin {
                    self.set_reencode_recording_video(true);
                }
                self.set_use_decodebin(use_decodebin);
            }
            SetVideoEncoderCodec(codec) => self.get_mut_video_encoder().0 = codec,
            SetVideoEncoderCodecProvider(provider) => self.get_mut_video_encoder().1 = provider,
            SetReencodeRecordingVideo(reencode) => {
                if !reencode {
                    self.set_use_decodebin(false);
                }
                self.set_reencode_recording_video(reencode)
            }
            SetAppSinkQueueLeakyEnabled(leaky) => self.set_appsink_queue_leaky_enabled(leaky),
            SetVideoLatency(latency) => self.set_video_latency(latency),
        }
    }
}
