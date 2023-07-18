/* preferences.rs
 *
 * Copyright 2021-2022 Bohong Huang
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::{fs, path::PathBuf, str::FromStr, time::Duration};

use adw::{
    prelude::*, ActionRow, ComboRow, ExpanderRow, PreferencesGroup, PreferencesPage,
    PreferencesWindow,
};
use glib::Sender;
use gtk::{Align, Entry, Inhibit, Label, SpinButton, StringList, Switch};
use relm4::{send, ComponentUpdate, Model, Widgets};
use relm4_macros::widget;

use derivative::*;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use url::Url;

use crate::{
    slave::video::{
        ColorspaceConversion, ImageFormat, VideoCodec, VideoCodecProvider, VideoDecoder,
        VideoEncoder,
    },
    AppColorScheme, AppModel, AppMsg,
};

pub fn get_data_path() -> PathBuf {
    const APP_DIR_NAME: &str = "rovhost";
    let mut data_path = dirs::data_local_dir().expect("无法找到本地数据文件夹");
    data_path.push(APP_DIR_NAME);
    if !data_path.exists() {
        fs::create_dir(data_path.clone()).expect("无法创建应用数据文件夹");
    }
    data_path
}

pub fn get_preference_path() -> PathBuf {
    let mut path = get_data_path();
    path.push("preferences.json");
    path
}

pub fn get_video_path() -> PathBuf {
    let mut video_path = get_data_path();
    video_path.push("Videos");
    if !video_path.exists() {
        fs::create_dir(video_path.clone()).expect("无法创建视频文件夹");
    }
    video_path
}

pub fn get_image_path() -> PathBuf {
    let mut video_path = get_data_path();
    video_path.push("Images");
    if !video_path.exists() {
        fs::create_dir(video_path.clone()).expect("无法创建图片文件夹");
    }
    video_path
}

#[tracker::track]
#[derive(Derivative, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[derivative(Default)]
pub struct PreferencesModel {
    #[derivative(Default(value = "1"))]
    pub initial_slave_num: u8,
    pub application_color_scheme: AppColorScheme,
    #[derivative(Default(value = "get_video_path()"))]
    pub video_save_path: PathBuf,
    #[derivative(Default(value = "get_image_path()"))]
    pub image_save_path: PathBuf,
    #[derivative(Default(value = "ImageFormat::JPEG"))]
    pub image_save_format: ImageFormat,
    pub default_reencode_recording_video: bool,
    pub default_video_encoder: VideoEncoder,
    #[derivative(Default(value = "Url::from_str(\"http://192.168.137.219:8888\").unwrap()"))]
    pub default_slave_url: Url,
    #[derivative(Default(
        value = "Url::from_str(\"rtp://127.0.0.1:5600?encoding-name=H264\").unwrap()"
    ))]
    pub default_video_url: Url,
    #[derivative(Default(value = "60"))]
    pub default_input_sending_rate: u16,
    #[derivative(Default(value = "true"))]
    pub default_keep_video_display_ratio: bool,
    pub default_video_decoder: VideoDecoder,
    pub default_colorspace_conversion: ColorspaceConversion,
    #[derivative(Default(value = "64"))]
    pub param_tuner_graph_view_point_num_limit: u16,
    #[derivative(Default(value = "250"))]
    pub param_tuner_graph_view_update_interval: u16,
    #[derivative(Default(value = "Duration::from_secs(10)"))]
    pub pipeline_timeout: Duration,
    #[derivative(Default(value = "false"))]
    pub default_appsink_queue_leaky_enabled: bool,
    #[derivative(Default(value = "false"))]
    pub default_use_decodebin: bool,
    #[derivative(Default(value = "false"))]
    pub video_sync_record_use_separate_directory: bool,
    #[derivative(Default(value = "200"))]
    pub default_video_latency: u32,
    #[derivative(Default(value = "500"))]
    pub default_status_info_update_interval: u16,
}

impl PreferencesModel {
    pub fn load_or_default() -> PreferencesModel {
        match fs::read_to_string(get_preference_path())
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
        {
            Some(model) => model,
            None => Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum PreferencesMsg {
    SetVideoSavePath(PathBuf),
    SetImageSavePath(PathBuf),
    SetImageSaveFormat(ImageFormat),
    SetInitialSlaveNum(u8),
    SetInputSendingRate(u16),
    SetParamTunerGraphViewUpdateInterval(u16),
    SetDefaultKeepVideoDisplayRatio(bool),
    SetDefaultVideoDecoderCodec(VideoCodec),
    SetDefaultVideoDecoderCodecProvider(VideoCodecProvider),
    SetDefaultVideoEncoderCodec(VideoCodec),
    SetDefaultVideoEncoderCodecProvider(VideoCodecProvider),
    SetParameterTunerGraphViewPointNumberLimit(u16),
    SetDefaultColorspaceConversion(ColorspaceConversion),
    SetDefaultReencodeRecordingVideo(bool),
    SetDefaultUseDecodebin(bool),
    SetDefaultAppSinkQueueLeakyEnabled(bool),
    SetVideoSyncRecordUseSeparateDirectory(bool),
    SetDefaultVideoLatency(u32),
    SetDefaultVideoUrl(Url),
    SetDefaultSlaveUrl(Url),
    SetPipelineTimeout(Duration),
    SetApplicationColorScheme(Option<AppColorScheme>),
    SetDefaultStatusInfoUpdateInterval(u16),
    SaveToFile,
    OpenVideoDirectory,
    OpenImageDirectory,
}

impl Model for PreferencesModel {
    type Msg = PreferencesMsg;
    type Widgets = PreferencesWidgets;
    type Components = ();
}

#[widget(pub)]
impl Widgets<PreferencesModel, AppModel> for PreferencesWidgets {
    view! {
        window = PreferencesWindow {
            set_title: Some("首选项"),
            set_transient_for: parent!(Some(&parent_widgets.app_window)),
            set_destroy_with_parent: true,
            set_modal: true,
            set_search_enabled: false,
            connect_close_request(sender) => move |window| {
                send!(sender, PreferencesMsg::SaveToFile);
                window.hide();
                Inhibit(true)
            },
            add = &PreferencesPage {
                set_title: "通信",
                set_icon_name: Some("network-transmit-receive-symbolic"),
                add = &PreferencesGroup {
                    set_description: Some("与机器人的连接通信设置"),
                    set_title: "连接",
                    add = &ActionRow {
                        set_title: "默认连接 URL",
                        set_subtitle: "连接第一机位的机器人使用的默认 URL，其他机位会自动累加 IPV4 地址",
                        add_suffix = &Entry {
                            set_text: track!(model.changed(PreferencesModel::default_slave_url()), model.get_default_slave_url().to_string().as_str()),
                            set_valign: Align::Center,
                            set_width_request: 200,
                            connect_changed(sender) => move |entry| {
                                if let Ok(url) = Url::from_str(&entry.text()) {
                                    send!(sender, PreferencesMsg::SetDefaultSlaveUrl(url));
                                    entry.remove_css_class("error");
                                } else {
                                    entry.add_css_class("error");
                                }
                            }
                         },
                    },
                },
                add = &PreferencesGroup {
                    set_description: Some("机器人状态信息接收设置"),
                    set_title: "状态信息",
                    add = &ActionRow {
                        set_title: "状态信息更新时间间隔",
                        set_subtitle: "用于确定每秒钟向机器人请求接收状态信息并测试连接状态的频率（需要重新连接以应用设置）",
                        add_suffix = &SpinButton::with_range(50.0, 10000.0, 50.0) {
                            set_value: track!(model.changed(PreferencesModel::default_status_info_update_interval()), model.default_status_info_update_interval as f64),
                            set_digits: 0,
                            set_valign: Align::Center,
                            set_can_focus: false,
                            connect_value_changed(sender) => move |button| {
                                send!(sender, PreferencesMsg::SetDefaultStatusInfoUpdateInterval(button.value() as u16));
                            }
                        },
                        add_suffix = &Label {
                            set_label: "毫秒",
                        },
                    },
                },
            },
            add = &PreferencesPage {
                set_title: "控制",
                set_icon_name: Some("input-gaming-symbolic"),
                add = &PreferencesGroup {
                    set_title: "发送",
                    set_description: Some("向机器人发送控制信号的设置（需要重新连接以应用设置）"),
                    add = &ActionRow {
                        set_title: "增量发送",
                        set_subtitle: "每次发送只发送相对上一次发送的变化值以节省数据发送量",
                        set_sensitive: false,
                        add_suffix: increamental_sending_switch = &Switch {
                            set_active: false,
                            set_valign: Align::Center,
                        },
                        set_activatable_widget: Some(&increamental_sending_switch),
                    },
                    add = &ActionRow {
                        set_title: "输入发送率",
                        set_subtitle: "每秒钟向机器人发送的控制数据包的个数，该值越高意味着控制越灵敏，但在较差的网络条件下可能产生更大的延迟",
                        add_suffix = &SpinButton::with_range(1.0, 1000.0, 1.0) {
                            set_value: track!(model.changed(PreferencesModel::default_input_sending_rate()), model.default_input_sending_rate as f64),
                            set_digits: 0,
                            set_valign: Align::Center,
                            set_can_focus: false,
                            connect_value_changed(sender) => move |button| {
                                send!(sender, PreferencesMsg::SetInputSendingRate(button.value() as u16));
                            }
                        },
                        add_suffix = &Label {
                            set_label: "Hz",
                        },
                    },
                },
            },
            add = &PreferencesPage {
                set_title: "视频",
                set_icon_name: Some("video-display-symbolic"),
                add = &PreferencesGroup {
                    set_title: "显示",
                    set_description: Some("上位机的显示的画面设置"),
                    add = &ActionRow {
                        set_title: "默认保持长宽比",
                        set_subtitle: "在改变窗口大小的时是否保持画面比例，这可能导致画面无法全屏",
                        add_suffix: default_keep_video_display_ratio_switch = &Switch {
                            set_active: track!(model.changed(PreferencesModel::default_keep_video_display_ratio()), model.default_keep_video_display_ratio),
                            set_valign: Align::Center,
                            connect_state_set(sender) => move |_switch, state| {
                                send!(sender, PreferencesMsg::SetDefaultKeepVideoDisplayRatio(state));
                                Inhibit(false)
                            }
                        },
                        set_activatable_widget: Some(&default_keep_video_display_ratio_switch),
                    },
                },
            },
            add = &PreferencesPage {
                set_title: "调试",
                set_icon_name: Some("preferences-other-symbolic"),
                add = &PreferencesGroup {
                    set_title: "控制环",
                    set_description: Some("配置控制环调试选项"),
                    add = &ActionRow {
                        set_title: "反馈曲线最大点数",
                        set_subtitle: "绘制控制环反馈曲线时使用最多使用点数，这将影响最多能观测的历史数据",
                        add_suffix = &SpinButton::with_range(1.0, 255.0, 1.0) {
                            set_value: track!(model.changed(PreferencesModel::param_tuner_graph_view_point_num_limit()), model.param_tuner_graph_view_point_num_limit as f64),
                            set_digits: 0,
                            set_valign: Align::Center,
                            set_can_focus: false,
                            connect_value_changed(sender) => move |button| {
                                send!(sender, PreferencesMsg::SetParameterTunerGraphViewPointNumberLimit(button.value() as u16));
                            },
                        },
                    },
                    add = &ActionRow {
                        set_title: "反馈曲线更新时间间隔",
                        set_subtitle: "控制环反馈曲线的更新速率，这将影响最多能观测的历史数据",
                        add_suffix = &SpinButton::with_range(50.0, 10000.0, 50.0) {
                            set_value: track!(model.changed(PreferencesModel::param_tuner_graph_view_update_interval()), model.param_tuner_graph_view_update_interval as f64),
                            set_digits: 0,
                            set_valign: Align::Center,
                            set_can_focus: false,
                            connect_value_changed(sender) => move |button| {
                                send!(sender, PreferencesMsg::SetParamTunerGraphViewUpdateInterval(button.value() as u16));
                            }
                        },
                        add_suffix = &Label {
                            set_label: "毫秒",
                        },
                    },
                },
            },
        }
    }

    fn post_init() {}
}

impl ComponentUpdate<AppModel> for PreferencesModel {
    fn init_model(parent_model: &AppModel) -> Self {
        parent_model.preferences.borrow().clone()
    }

    fn update(
        &mut self,
        msg: PreferencesMsg,
        _components: &(),
        _sender: Sender<PreferencesMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();
        match msg {
            PreferencesMsg::SetVideoSavePath(path) => self.set_video_save_path(path),
            PreferencesMsg::SetInitialSlaveNum(num) => self.set_initial_slave_num(num),
            PreferencesMsg::SetInputSendingRate(rate) => self.set_default_input_sending_rate(rate),
            PreferencesMsg::SetDefaultKeepVideoDisplayRatio(value) => {
                self.set_default_keep_video_display_ratio(value)
            }
            PreferencesMsg::SaveToFile => serde_json::to_string_pretty(&self)
                .ok()
                .and_then(|json| fs::write(get_preference_path(), json).ok())
                .unwrap(),
            PreferencesMsg::SetImageSavePath(path) => self.set_image_save_path(path),
            PreferencesMsg::SetImageSaveFormat(format) => self.set_image_save_format(format),
            PreferencesMsg::SetParameterTunerGraphViewPointNumberLimit(limit) => {
                self.set_param_tuner_graph_view_point_num_limit(limit)
            }
            PreferencesMsg::OpenVideoDirectory => gtk::show_uri(
                None as Option<&PreferencesWindow>,
                glib::filename_to_uri(self.get_video_save_path().to_str().unwrap(), None)
                    .unwrap()
                    .as_str(),
                gdk::CURRENT_TIME,
            ),
            PreferencesMsg::OpenImageDirectory => gtk::show_uri(
                None as Option<&PreferencesWindow>,
                glib::filename_to_uri(self.get_image_save_path().to_str().unwrap(), None)
                    .unwrap()
                    .as_str(),
                gdk::CURRENT_TIME,
            ),
            PreferencesMsg::SetDefaultColorspaceConversion(conversion) => {
                self.set_default_colorspace_conversion(conversion)
            }
            PreferencesMsg::SetDefaultVideoUrl(url) => self.default_video_url = url, // 防止输入框的光标移动至最前
            PreferencesMsg::SetDefaultSlaveUrl(url) => self.default_slave_url = url,
            PreferencesMsg::SetDefaultVideoDecoderCodec(codec) => {
                self.get_mut_default_video_decoder().0 = codec
            }
            PreferencesMsg::SetDefaultVideoDecoderCodecProvider(provider) => {
                self.get_mut_default_video_decoder().1 = provider
            }
            PreferencesMsg::SetDefaultReencodeRecordingVideo(reencode) => {
                if !reencode {
                    self.set_default_use_decodebin(false);
                }
                self.set_default_reencode_recording_video(reencode)
            }
            PreferencesMsg::SetDefaultVideoEncoderCodec(codec) => {
                self.get_mut_default_video_encoder().0 = codec
            }
            PreferencesMsg::SetDefaultVideoEncoderCodecProvider(provider) => {
                self.get_mut_default_video_encoder().1 = provider
            }
            PreferencesMsg::SetPipelineTimeout(timeout) => self.set_pipeline_timeout(timeout),
            PreferencesMsg::SetDefaultAppSinkQueueLeakyEnabled(leaky) => {
                self.set_default_appsink_queue_leaky_enabled(leaky)
            }
            PreferencesMsg::SetDefaultUseDecodebin(use_decodebin) => {
                if use_decodebin {
                    self.set_default_reencode_recording_video(true);
                }
                self.set_default_use_decodebin(use_decodebin);
            }
            PreferencesMsg::SetVideoSyncRecordUseSeparateDirectory(use_separate_directory) => {
                self.set_video_sync_record_use_separate_directory(use_separate_directory)
            }
            PreferencesMsg::SetDefaultVideoLatency(latency) => {
                self.set_default_video_latency(latency)
            }
            PreferencesMsg::SetApplicationColorScheme(scheme) => {
                if let Some(scheme) = scheme {
                    self.set_application_color_scheme(scheme);
                }
                send!(
                    parent_sender,
                    AppMsg::SetColorScheme(*self.get_application_color_scheme())
                );
            }
            PreferencesMsg::SetDefaultStatusInfoUpdateInterval(interval) => {
                self.set_default_status_info_update_interval(interval)
            }
            PreferencesMsg::SetParamTunerGraphViewUpdateInterval(interval) => {
                self.set_param_tuner_graph_view_update_interval(interval)
            }
        }
        send!(parent_sender, AppMsg::PreferencesUpdated(self.clone()));
    }
}
