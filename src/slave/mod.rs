mod config;

use adw::{prelude::*, ToastOverlay};
use relm4::{
    factory::{positions::GridPosition, Position},
    gtk::{
        Align, Box as GtkBox, Button as GtkButton, CenterBox, Frame, Grid, Label, MenuButton,
        Orientation, Popover, Separator, ToggleButton,
    },
    prelude::*,
};

use crate::AppMsg;

#[tracker::track]
pub struct SlaveModel {
    connected: Option<bool>,
    recording: Option<bool>,
}

#[derive(Debug)]
pub enum SlaveMsg {
    ConfigUpdated,
    ToggleRecord,
    ToggleConnect,
    TogglePolling,
    PollingChanged(bool),
    RecordingChanged(bool),
    TakeScreenshot,
    //AddInputSource(InputSource),
    //RemoveInputSource(InputSource),
    //SetSlaveStatus(SlaveStatusClass, i16),
    UpdateInputSources,
    ToggleDisplayInfo,
    //InputReceived(InputSourceEvent),
    OpenFirmwareUpater,
    OpenParameterTuner,
    DestroySlave,
    ErrorMessage(String),
    CommunicationError(String),
    //ConnectionChanged(Option<async_std::sync::Arc<RpcClient>>),
    ShowToastMessage(String),
    //CommunicationMessage(SlaveCommunicationMsg),
    //InformationsReceived(HashMap<String, String>),
    SetConfigPresented(bool),
}

impl Position<GridPosition, DynamicIndex> for SlaveModel {
    fn position(&self, index: &DynamicIndex) -> GridPosition {
        let index = index.current_index() as i32;
        let row = index / 3;
        let column = index % 3;
        GridPosition {
            column,
            row,
            width: 1,
            height: 1,
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for SlaveModel {
    type Init = ();
    type Input = SlaveMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentInput = AppMsg;
    type ParentWidget = Grid;

    view! {
        #[root]
        ToastOverlay {
            #[wrap(Some)]
            set_child = &GtkBox {
                set_orientation: Orientation::Vertical,
                append = &CenterBox {
                    set_css_classes: &["toolbar"],
                    set_orientation: Orientation::Horizontal,
                    #[wrap(Some)]
                    set_start_widget = &GtkBox {
                        set_hexpand: true,
                        set_halign: Align::Start,
                        set_spacing: 5,
                        append = &GtkButton {
                            set_icon_name: "network-transmit-symbolic",
                            #[track = "self.changed(SlaveModel::connected())"]
                            set_sensitive: self.connected != None,
                            #[watch]
                            set_css_classes?: self.connected.map(|x| if x { vec!["circular", "suggested-action"] } else { vec!["circular"] }).as_ref(),
                            #[track = "self.changed(SlaveModel::connected())"]
                            set_tooltip_text: self.connected.map(|x| if x { "断开连接" } else { "连接" }),
                            connect_clicked[sender] => move |_button| {
                                sender.input(SlaveMsg::ToggleConnect);
                            },
                        },
                        append = &GtkButton {
                            set_icon_name: "video-display-symbolic",
                            // #[track = "self.changed(SlaveModel::recording()) || self.changed(SlaveModel::sync_recording()) || self.changed(SlaveModel::polling())"]
                            // set_sensitive: self.get_recording().is_some() && self.get_polling().is_some() && !self.sync_recording,
                            // #[watch]
                            // set_css_classes?: self.polling.map(|x| if x { vec!["circular", "destructive-action"] } else { vec!["circular"] }).as_ref(),
                            // set_tooltip_text: track!(model.changed(SlaveModel::polling()), model.polling.map(|x| if x { "停止拉流" } else { "启动拉流" })),
                            connect_clicked[sender] => move |_| {
                                sender.input(SlaveMsg::TogglePolling);
                            },
                        },
                        append = &Separator {},
                        append = &GtkButton {
                            set_icon_name: "camera-photo-symbolic",
                            #[watch]
                            // set_sensitive: self.video.model().get_pixbuf().is_some(),
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("画面截图"),
                            connect_clicked[sender] => move |_| {
                                sender.input(SlaveMsg::TakeScreenshot);
                            },
                        },
                        append = &GtkButton {
                            set_icon_name: "camera-video-symbolic",
                            // set_sensitive: track!(model.changed(SlaveModel::sync_recording()) || model.changed(SlaveModel::polling()) || model.changed(SlaveModel::recording()), !model.sync_recording && model.recording != None &&  model.polling == Some(true)),
                            // set_css_classes?: watch!(model.recording.map(|x| if x { vec!["circular", "destructive-action"] } else { vec!["circular"] }).as_ref()),
                            // set_tooltip_text: track!(model.changed(SlaveModel::recording()), model.recording.map(|x| if x { "停止录制" } else { "开始录制" })),
                            connect_clicked[sender] => move |_| {
                                sender.input(SlaveMsg::ToggleRecord);
                            },
                        },
                    },
                    #[wrap(Some)]
                    set_center_widget = &GtkBox {
                        set_hexpand: true,
                        set_halign: Align::Center,
                        set_spacing: 5,
                        append = &Label {
                            // set_text: track!(model.changed(SlaveModel::config()), model.config.model().get_slave_url().to_string().as_str()),
                        },
                        append = &MenuButton {
                            set_icon_name: "input-gaming-symbolic",
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("切换当前机位使用的输入设备"),
                            #[wrap(Some)]
                            set_popover = &Popover {
                                #[wrap(Some)]
                                set_child = &GtkBox {
                                    set_spacing: 5,
                                    set_orientation: Orientation::Vertical,
                                    append = &CenterBox {
                                        #[wrap(Some)]
                                        set_center_widget = &Label {
                                            set_margin_start: 10,
                                            set_margin_end: 10,
                                            set_markup: "<b>输入设备</b>"
                                        },
                                        #[wrap(Some)]
                                        set_end_widget = &GtkButton {
                                            set_icon_name: "view-refresh-symbolic",
                                            set_css_classes: &["circular"],
                                            set_tooltip_text: Some("刷新输入设备"),
                                            connect_clicked[sender] => move |_| {
                                                sender.input(SlaveMsg::UpdateInputSources);
                                            },
                                        },
                                    },
                                    append = &Frame {
                                        // set_child: track!(model.changed(SlaveModel::input_system()), Some(&input_sources_list_box(&model.input_sources, &model.input_system ,&sender))),
                                    },

                                },
                            },
                        },
                    },
                    #[wrap(Some)]
                    set_end_widget = &GtkBox {
                        set_hexpand: true,
                        set_halign: Align::End,
                        set_spacing: 5,
                        set_margin_end: 5,
                        append = &GtkButton {
                            set_icon_name: "software-update-available-symbolic",
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("固件更新"),
                            connect_clicked[sender] => move |_button| {
                                sender.input(SlaveMsg::OpenFirmwareUpater);
                            },
                        },
                        append = &GtkButton {
                            set_icon_name: "preferences-other-symbolic",
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("参数调校"),
                            connect_clicked[sender] => move |_button| {
                                sender.input(SlaveMsg::OpenParameterTuner);
                            },
                        },
                        append = &Separator {},
                        append = &ToggleButton {
                            set_icon_name: "emblem-system-symbolic",
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("机位设置"),
                            // set_active: track!(model.changed(SlaveModel::config_presented()), *model.get_config_presented()),
                            // connect_active_notify(sender) => move |button| {
                            //     send!(sender, SlaveMsg::SetConfigPresented(button.is_active()));
                            // },
                        },
                        append = &ToggleButton {
                            set_icon_name: "window-close-symbolic",
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("移除机位"),
                            set_visible: false,
                            connect_active_notify[sender] => move |_button| {
                                sender.input(SlaveMsg::DestroySlave);
                            },
                        },
                    },
                },
            }
        }
    }

    fn init_model(_value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            connected: Some(false),
            recording: None,
            tracker: 0,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        use SlaveMsg::*;
        match message {
            ConfigUpdated => {}
            ToggleRecord => {}
            ToggleConnect => {}
            TogglePolling => {}
            PollingChanged(_val) => {}
            RecordingChanged(_val) => {}
            TakeScreenshot => {}
            //AddInputSource(InputSource) => {}
            //RemoveInputSource(InputSource) => {}
            //SetSlaveStatus(SlaveStatusClass, i16) => {}
            UpdateInputSources => {}
            ToggleDisplayInfo => {}
            //InputReceived(InputSourceEvent) => {}
            OpenFirmwareUpater => {}
            OpenParameterTuner => {}
            DestroySlave => {}
            ErrorMessage(_str) => {}
            CommunicationError(_str) => {}
            //ConnectionChanged(Option<async_std::sync::Arc<RpcClient>>) => {}
            ShowToastMessage(_str) => {}
            //CommunicationMessage(SlaveCommunicationMsg) => {}
            //InformationsReceived(HashMap<String, String>) => {}
            SetConfigPresented(_val) => {}
        }
    }
}
