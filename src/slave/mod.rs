mod async_glib;
mod config;
mod video;

pub mod video_ext;

use relm4::{
    adw::{prelude::*, Flap, ToastOverlay},
    factory::{positions::GridPosition, Position},
    gtk::{
        glib::DateTime, Align, Box as GtkBox, Button as GtkButton, CenterBox, Frame, Grid, Image,
        Label, MenuButton, Orientation, Overlay, PackType, Popover, Revealer, Separator,
        ToggleButton,
    },
    prelude::*,
};

use crate::{
    preferences::PreferencesModel,
    slave::{config::SlaveConfigInput, video::SlaveVideoInput},
    AppMsg,
};

use self::{
    config::{SlaveConfigModel, SlaveConfigOutput},
    video::{SlaveVideoInit, SlaveVideoModel, SlaveVideoOutput},
};

#[tracker::track]
pub struct SlaveModel {
    connected: Option<bool>,
    recording: Option<bool>,
    polling: Option<bool>,
    #[no_eq]
    video_model: Controller<SlaveVideoModel>,
    #[no_eq]
    config_model: Controller<SlaveConfigModel>,
    // status: Arc<Mutex<HashMap<SlaveStatusClass, i16>>>,
    // rpc_client: Option<async_std::sync::Arc<RpcClient>>,
    // infos: FactoryVecDeque<SlaveInfoModel>,
    // pub input_event_sender: Sender<InputSourceEvent>,
    #[no_eq]
    preferences: PreferencesModel,
    sync_recording: bool,
    slave_info_displayed: bool,
    config_presented: bool,
    index: DynamicIndex,
}

#[derive(Debug)]
pub enum SlaveInput {
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

    UpdataPreferences(PreferencesModel),
    UpdateConfig(SlaveConfigModel),
}

#[derive(Debug)]
pub enum SlaveOutput {
    DestroySlave(usize),
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
                                sender.input(SlaveInput::ToggleConnect);
                            },
                        },
                        append = &GtkButton {
                            set_icon_name: "video-display-symbolic",
                            #[track = "self.changed(SlaveModel::recording()) || self.changed(SlaveModel::sync_recording()) || self.changed(SlaveModel::polling())"]
                            set_sensitive: self.get_recording().is_some() && self.get_polling().is_some() && !self.sync_recording,
                            #[watch]
                            set_css_classes?: self.polling.map(|x| if x { vec!["circular", "destructive-action"] } else { vec!["circular"] }).as_ref(),
                            #[track = "self.changed(SlaveModel::polling())"]
                            set_tooltip_text: self.polling.map(|x| if x { "停止拉流" } else { "启动拉流" }),
                            connect_clicked[sender] => move |_| {
                                sender.input(SlaveInput::TogglePolling);
                            },
                        },
                        append = &Separator {},
                        append = &GtkButton {
                            set_icon_name: "camera-photo-symbolic",
                            #[watch]
                            set_sensitive: self.video_model.model().get_pixbuf().is_some(),
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("画面截图"),
                            connect_clicked[sender] => move |_| {
                                sender.input(SlaveInput::TakeScreenshot);
                            },
                        },
                        append = &GtkButton {
                            set_icon_name: "camera-video-symbolic",
                            #[track = "self.changed(SlaveModel::sync_recording()) || self.changed(SlaveModel::polling()) || self.changed(SlaveModel::recording())"]
                            set_sensitive: !self.sync_recording && self.recording != None &&  self.polling == Some(true),
                            #[watch]
                            set_css_classes?: self.recording.map(|x| if x { vec!["circular", "destructive-action"] } else { vec!["circular"] }).as_ref(),
                            #[track = "self.changed(SlaveModel::recording())"]
                            set_tooltip_text: self.recording.map(|x| if x { "停止录制" } else { "开始录制" }),
                            connect_clicked[sender] => move |_| {
                                sender.input(SlaveInput::ToggleRecord);
                            },
                        },
                    },
                    #[wrap(Some)]
                    set_center_widget = &GtkBox {
                        set_hexpand: true,
                        set_halign: Align::Center,
                        set_spacing: 5,
                        append = &Label {
                            #[track = "self.changed(SlaveModel::config_model())"]
                            set_text: self.config_model.model().get_slave_url().to_string().as_str(),
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
                                                sender.input(SlaveInput::UpdateInputSources);
                                            },
                                        },
                                    },
                                    append = &Frame {
                                        // #[track = "model.changed(SlaveModel::input_system())"]
                                        // #[wrap(Some)]
                                        // set_child: &input_sources_list_box(&model.input_sources, &model.input_system ,&sender),
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
                                sender.input(SlaveInput::OpenFirmwareUpater);
                            },
                        },
                        append = &GtkButton {
                            set_icon_name: "preferences-other-symbolic",
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("参数调校"),
                            connect_clicked[sender] => move |_button| {
                                sender.input(SlaveInput::OpenParameterTuner);
                            },
                        },
                        append = &Separator {},
                        append = &ToggleButton {
                            set_icon_name: "emblem-system-symbolic",
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("机位设置"),
                            #[track = "self.changed(SlaveModel::config_presented())"]
                            set_active: *self.get_config_presented(),
                            connect_active_notify[sender] => move |button| {
                                sender.input(SlaveInput::SetConfigPresented(button.is_active()));
                            },
                        },
                        append = &ToggleButton {
                            set_icon_name: "window-close-symbolic",
                            set_css_classes: &["circular"],
                            set_tooltip_text: Some("移除机位"),
                            set_visible: false,
                            connect_active_notify[sender] => move |_button| {
                                sender.input(SlaveInput::DestroySlave);
                            },
                        },
                    },
                },
                append = &Flap {
                    set_flap: Some(self.config_model.widget()),
                    #[track = "self.changed(SlaveModel::config_presented())"]
                    set_reveal_flap: *self.get_config_presented(),
                    set_fold_policy: adw::FlapFoldPolicy::Auto,
                    set_locked: true,
                    set_flap_position: PackType::End,
                    #[wrap(Some)]
                    set_separator = &Separator {},
                    #[wrap(Some)]
                    set_content = &Overlay {
                        set_width_request: 640,
                        set_child: Some(self.video_model.widget()),
                        add_overlay = &GtkBox {
                            set_valign: Align::Start,
                            set_halign: Align::End,
                            set_hexpand: true,
                            set_margin_all: 20,
                            append = &Frame {
                                add_css_class: "card",
                                #[wrap(Some)]
                                set_child = &GtkBox {
                                    set_orientation: Orientation::Vertical,
                                    set_margin_all: 5,
                                    set_width_request: 50,
                                    set_spacing: 5,
                                    append = &GtkButton {
                                        #[wrap(Some)]
                                        set_child = &CenterBox {
                                            #[wrap(Some)]
                                            set_center_widget = &Label {
                                                set_margin_start: 10,
                                                set_margin_end: 10,
                                                set_text: "状态信息",
                                            },
                                            #[wrap(Some)]
                                            set_end_widget = &Image {
                                                //#[watch]
                                                //set_icon_name: Some(if self.slave_info_displayed { "go-down-symbolic" } else { "go-next-symbolic" }),
                                            },
                                        },
                                        connect_clicked[sender] => move |_button| {
                                            sender.input(SlaveInput::ToggleDisplayInfo);
                                        },
                                    },
                                    append = &Revealer {
                                        // #[watch]
                                        // set_reveal_child: self.slave_info_displayed,
                                        #[wrap(Some)]
                                        set_child = &GtkBox {
                                            set_spacing: 5,
                                            set_margin_all: 5,
                                            set_orientation: Orientation::Vertical,
                                            set_halign: Align::Center,
                                            append = &GtkBox {
                                                set_hexpand: true,
                                                set_halign: Align::Center,
                                                append = &Grid {
                                                    set_margin_all: 2,
                                                    set_row_spacing: 2,
                                                    set_column_spacing: 2,
                                                    // attach(0, 0, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "go-last-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     #[track = "model.changed(SlaveModel::status())"]
                                                    //     set_active: model.get_target_status(&SlaveStatusClass::RoboticArmClose) > 0,
                                                    // },
                                                    // attach(1, 0, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "object-flip-horizontal-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::RoboticArmOpen) > 0),
                                                    // },
                                                    // attach(2, 0, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "go-first-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::RoboticArmClose) > 0),
                                                    // },
                                                    // attach(0, 1, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "object-rotate-left-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::MotionRotate) < -JOYSTICK_DISPLAY_THRESHOLD),
                                                    // },
                                                    // attach(2, 1, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "object-rotate-right-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::MotionRotate) > JOYSTICK_DISPLAY_THRESHOLD),
                                                    // },
                                                    // attach(0, 3, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "go-bottom-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::MotionZ) < -JOYSTICK_DISPLAY_THRESHOLD),
                                                    // },
                                                    // attach(2, 3, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "go-top-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::MotionZ) > JOYSTICK_DISPLAY_THRESHOLD),
                                                    // },
                                                    // attach(1, 1, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "go-up-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::MotionY) > JOYSTICK_DISPLAY_THRESHOLD),
                                                    // },
                                                    // attach(0, 2, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "go-previous-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::MotionX) < -JOYSTICK_DISPLAY_THRESHOLD),
                                                    // },
                                                    // attach(2, 2, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "go-next-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::MotionX) > JOYSTICK_DISPLAY_THRESHOLD),
                                                    // },
                                                    // attach(1, 3, 1, 1) = &ToggleButton {
                                                    //     set_icon_name: "go-down-symbolic",
                                                    //     set_can_focus: false,
                                                    //     set_can_target: false,
                                                    //     set_active: track!(model.changed(SlaveModel::status()), model.get_target_status(&SlaveStatusClass::MotionY) < -JOYSTICK_DISPLAY_THRESHOLD),
                                                    // },
                                                },
                                            },
                                            append = &GtkBox {
                                                set_orientation: Orientation::Vertical,
                                                set_spacing: 5,
                                                set_hexpand: true,
                                                // factory!(model.infos),
                                            },
                                            append = &CenterBox {
                                                set_hexpand: true,
                                                #[wrap(Some)]
                                                set_start_widget = &Label {
                                                    set_markup: "<b>深度锁定</b>",
                                                },
                                                // #[wrap(Some)]
                                                // set_end_widget = &Switch {
                                                //     #[track = "self.changed(SlaveModel::status())"]
                                                //     set_active:  model.get_target_status(&SlaveStatusClass::DepthLocked) != 0,
                                                //     connect_state_set(sender) => move |_switch, state| {
                                                //         send!(sender, SlaveInput::SetSlaveStatus(SlaveStatusClass::DepthLocked, if state { 1 } else { 0 }));
                                                //         Inhibit(false)
                                                //     },
                                                // },
                                            },
                                            append = &CenterBox {
                                                set_hexpand: true,
                                                #[wrap(Some)]
                                                set_start_widget = &Label {
                                                    set_markup: "<b>方向锁定</b>",
                                                },
                                                // #[wrap(Some)]
                                                // set_end_widget = &Switch {
                                                //     #[track = "self.changed(SlaveModel::status())"]
                                                //     set_active: self.get_target_status(&SlaveStatusClass::DirectionLocked) != 0,
                                                //     connect_state_set(sender) => move |_switch, state| {
                                                //         send!(sender, SlaveInput::SetSlaveStatus(SlaveStatusClass::DirectionLocked, if state { 1 } else { 0 }));
                                                //         Inhibit(false)
                                                //     },
                                                // },
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                    connect_reveal_flap_notify[sender] => move |flap| {
                        sender.input(SlaveInput::SetConfigPresented(flap.reveals_flap()));
                    },
                },
            }
        }
    }

    type Init = PreferencesModel;
    type Input = SlaveInput;
    type Output = SlaveOutput;
    type CommandOutput = ();
    type ParentInput = AppMsg;
    type ParentWidget = Grid;

    fn init_model(
        preferences: Self::Init,
        index: &DynamicIndex,
        sender: FactorySender<Self>,
    ) -> Self {
        let config_model = SlaveConfigModel::builder()
            .launch(preferences.clone())
            .forward(sender.input_sender(), |msg| match msg {
                SlaveConfigOutput::UpdateConfig(config) => SlaveInput::UpdateConfig(config),
            });
        let video_model_init = SlaveVideoInit {
            preferences: preferences.clone(),
            config: config_model.model().clone(),
        };
        let video_model = SlaveVideoModel::builder().launch(video_model_init).forward(
            sender.input_sender(),
            |msg| match msg {
                SlaveVideoOutput::ErrorMessage(str) => SlaveInput::ErrorMessage(str),
                SlaveVideoOutput::PollingChanged(val) => SlaveInput::PollingChanged(val),
                SlaveVideoOutput::RecordingChanged(val) => SlaveInput::RecordingChanged(val),
                SlaveVideoOutput::ShowToastMessage(str) => SlaveInput::ShowToastMessage(str),
            },
        );
        Self {
            preferences,
            connected: Some(false),
            recording: Some(false),
            polling: Some(false),
            sync_recording: false,
            slave_info_displayed: false,
            config_presented: false,
            video_model,
            config_model,
            index: index.clone(),
            tracker: 0,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        self.reset();

        use SlaveInput::*;
        match message {
            ToggleRecord => {
                let video = &self.video_model;
                if video.model().get_record_handle().is_none() {
                    let mut pathbuf = self.preferences.get_video_save_path().clone();
                    pathbuf.push(format!(
                        "{}.mkv",
                        DateTime::now_local()
                            .unwrap()
                            .format_iso8601()
                            .unwrap()
                            .replace(":", "-")
                    ));
                    video.emit(SlaveVideoInput::StartRecord(pathbuf));
                } else {
                    video.emit(SlaveVideoInput::StopRecord(None));
                }
                self.set_recording(None);
            }
            ToggleConnect => {
                match self.get_connected() {
                    Some(true) => {
                        // 断开连接
                        self.set_connected(None);
                        self.config_model.emit(SlaveConfigInput::SetConnected(None));
                        // let sender = self.get_communication_msg_sender().clone().unwrap();
                        // task::spawn(async move {
                        //     sender.send(SlaveCommunicationMsg::Disconnect).await.expect("Communication main loop should be running");
                        // });
                    }
                    Some(false) => { // 连接
                         // let url = self.config_model.model().get_slave_url().clone();
                         // if let ("http", url_str) = (url.scheme(), url.as_str()) {
                         //     if let Ok(rpc_client) = RpcClientBuilder::default().build(url_str) {
                         //         let (comm_sender, comm_receiver) = async_std::channel::bounded::<SlaveCommunicationMsg>(128);
                         //         self.set_communication_msg_sender(Some(comm_sender.clone()));
                         //         let sender = sender.clone();
                         //         let control_sending_rate = *self.preferences.borrow().get_default_input_sending_rate();
                         //         self.set_connected(None);
                         //         self.config.send(SlaveConfigMsg::SetConnected(None)).unwrap();
                         //         let status_info_update_interval = *self.preferences.borrow().get_default_status_info_update_interval();
                         //         async_std::task::spawn(async move {
                         //             communication_main_loop(control_sending_rate,
                         //                                     Arc::new(rpc_client),
                         //                                     comm_sender,
                         //                                     comm_receiver,
                         //                                     sender.clone(),
                         //                                     status_info_update_interval as u64).await.unwrap_or_default();
                         //         });
                         //     } else {
                         //         error_message("错误", "无法创建 RPC 客户端。", app_window.upgrade().as_ref());
                         //     }
                         // } else {
                         //     error_message("错误", "连接 URL 有误，请检查并修改后重试 。", app_window.upgrade().as_ref());
                         // }
                    }
                    None => (),
                }
            }
            TogglePolling => match self.get_polling() {
                Some(true) => {
                    self.video_model.emit(SlaveVideoInput::StopPipeline);
                    self.set_polling(None);
                    self.config_model.emit(SlaveConfigInput::SetPolling(None));
                }
                Some(false) => {
                    self.video_model.emit(SlaveVideoInput::StartPipeline);
                    self.set_polling(None);
                    self.config_model.emit(SlaveConfigInput::SetPolling(None));
                }
                None => (),
            },
            PollingChanged(val) => {
                self.set_polling(Some(val));
                self.config_model
                    .emit(SlaveConfigInput::SetPolling(Some(val)));
            }
            RecordingChanged(val) => {
                if val {
                    if *self.get_recording() == Some(false) {
                        self.set_sync_recording(true);
                    }
                } else {
                    self.set_sync_recording(false);
                }
                self.set_recording(Some(val));
            }
            TakeScreenshot => {
                let mut pathbuf = self.preferences.get_image_save_path().clone();
                let format = self.preferences.get_image_save_format().clone();
                pathbuf.push(format!(
                    "{}.{}",
                    DateTime::now_local()
                        .unwrap()
                        .format_iso8601()
                        .unwrap()
                        .replace(":", "-"),
                    format.extension()
                ));
                self.video_model
                    .emit(SlaveVideoInput::SaveScreenshot(pathbuf));
            }
            //AddInputSource(InputSource) => {}
            //RemoveInputSource(InputSource) => {}
            //SetSlaveStatus(SlaveStatusClass, i16) => {}
            UpdateInputSources => {}
            ToggleDisplayInfo => self.set_slave_info_displayed(!self.get_slave_info_displayed()),
            //InputReceived(InputSourceEvent) => {}
            OpenFirmwareUpater => {}
            OpenParameterTuner => {}
            DestroySlave => {
                if let Some(polling) = self.get_polling() {
                    if *polling {
                        self.video_model.emit(SlaveVideoInput::StopPipeline);
                    }
                }
                if let Some(connected) = self.get_connected() {
                    if *connected {
                        sender.input(SlaveInput::ToggleConnect);
                    }
                }
                sender.output(SlaveOutput::DestroySlave(self.index.current_index()));
            }
            ErrorMessage(str) => {
                println!("错误: {}", str);
                // error_message("错误", &msg, app_window.upgrade().as_ref());
            }
            CommunicationError(_str) => {}
            //ConnectionChanged(Option<async_std::sync::Arc<RpcClient>>) => {}
            ShowToastMessage(_str) => {}
            //CommunicationMessage(SlaveCommunicationMsg) => {}
            //InformationsReceived(HashMap<String, String>) => {}
            SetConfigPresented(val) => self.set_config_presented(val),
            UpdataPreferences(preferences) => {
                self.config_model
                    .emit(SlaveConfigInput::UpdatePreferences(preferences.clone()));
                self.video_model
                    .emit(SlaveVideoInput::UpdatePreferences(preferences));

                sender.input(SlaveInput::UpdateConfig(self.config_model.model().clone()));
            }
            UpdateConfig(config) => self.video_model.emit(SlaveVideoInput::UpdateConfig(config)),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        use SlaveOutput::*;
        match output {
            DestroySlave(index) => Some(AppMsg::DestroySlave(index)),
        }
    }
}
