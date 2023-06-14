mod about;
mod preferences;
mod slave;
mod ui;

use adw::{prelude::*, CenteringPolicy, ColorScheme, HeaderBar, StatusPage, StyleManager};
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    factory::FactoryVecDeque,
    gtk::{
        Align, Box as GtkBox, Button, Grid, Image, Inhibit, Label, MenuButton, Orientation,
        Separator, Stack, ToggleButton,
    },
    new_action_group, new_stateless_action,
    prelude::*,
    Component, ComponentParts, RelmApp,
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use about::*;
use preferences::*;
use slave::SlaveModel;

use crate::slave::SlaveInput;

#[tracker::track]
struct AppModel {
    sync_recording: Option<bool>,
    is_fullscreen: bool,
    #[no_eq]
    slaves: FactoryVecDeque<SlaveModel>,
    #[do_not_track]
    about_model: Controller<AboutModel>,
    #[do_not_track]
    prefermances_model: Controller<PreferencesModel>,
}

new_action_group!(AppActionGroup, "main");
new_stateless_action!(PreferencesAction, AppActionGroup, "preferences");
new_stateless_action!(AboutDialogAction, AppActionGroup, "about");

#[relm4::component]
impl SimpleComponent for AppModel {
    view! {
        adw::ApplicationWindow {
            set_title: Some("水下机器人上位机"),
            set_default_width: 1280,
            set_default_height: 720,
            set_icon_name: Some("input-gaming"),
            #[track = "model.changed(AppModel::is_fullscreen())"]
            set_fullscreened: *model.get_is_fullscreen(),
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::StopInputSystem);
                Inhibit(false)
            },
            #[wrap(Some)]
            set_content = &GtkBox {
                set_orientation: Orientation::Vertical,
                append = &HeaderBar {
                    set_centering_policy: CenteringPolicy::Strict,
                    pack_start = &Button {
                        set_halign: Align::Center,
                        #[watch]
                        set_css_classes?: model.sync_recording.map(|x| if x { &["destructive-action"] as &[&str] } else { &[] as &[&str] }),
                        #[wrap(Some)]
                        set_child = &GtkBox {
                            set_spacing: 6,
                            append = &Image {
                                #[watch]
                                set_icon_name?: model.sync_recording.map(|x| Some(if x { "media-playback-stop-symbolic" } else { "media-record-symbolic" }))
                            },
                            append = &Label {
                                #[watch]
                                set_label?: model.sync_recording.map(|x| if x { "停止" } else { "同步录制" }),
                            },
                        },
                        #[track = "model.changed(AppModel::slaves())"]
                        set_visible: model.get_slaves().len() > 1,
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::ToggleSyncRecording);
                        }
                    },
                    pack_end = &MenuButton {
                        set_menu_model: Some(&main_menu),
                        set_icon_name: "open-menu-symbolic",
                        set_focus_on_click: false,
                        set_valign: Align::Center,
                    },
                    pack_end = &ToggleButton {
                        set_icon_name: "view-fullscreen-symbolic",
                        set_tooltip_text: Some("切换全屏模式"),
                        #[track = "model.changed(AppModel::is_fullscreen())"]
                        set_active: *model.get_is_fullscreen(),
                        connect_clicked[sender] => move |button| {
                            sender.input(AppMsg::SetFullscreened(button.is_active()));
                        }
                    },
                    pack_end = &Separator {},
                    pack_end = &Button {
                        set_icon_name: "list-remove-symbolic",
                        set_tooltip_text: Some("移除机位"),
                        #[track = "model.changed(AppModel::sync_recording()) || model.changed(AppModel::slaves())"]
                        set_sensitive: model.get_slaves().len() > 0 && *model.get_sync_recording() ==  Some(false),
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::RemoveLastSlave);
                        },
                    },
                    pack_end = &Button {
                        set_icon_name: "list-add-symbolic",
                        set_tooltip_text: Some("新建机位"),
                        #[track = "model.changed(AppModel::sync_recording())"]
                        set_sensitive: model.sync_recording == Some(false),
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::NewSlave);
                        },
                    },
                },
                append = &Stack {
                    set_hexpand: true,
                    set_vexpand: true,
                    add_child = &StatusPage {
                        set_icon_name: Some("window-new-symbolic"),
                        set_title: "无机位",
                        set_description: Some("请点击标题栏右侧按钮添加机位"),
                        #[track = "model.changed(AppModel::slaves())"]
                        set_visible: model.get_slaves().len() == 0,
                    },
                    add_child = &GtkBox {
                        #[track = "model.changed(AppModel::slaves())"]
                        set_visible: model.get_slaves().len() != 0,
                        append: slave
                    }
                }
            }
        }
    }

    menu! {
        main_menu: {
            "首选项" => PreferencesAction,
            "关于"   => AboutDialogAction,
        }
    }

    type Init = ();
    type Input = AppMsg;
    type Output = ();

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let about_model = AboutModel::builder()
            .transient_for(root)
            .launch(())
            .detach();
        let prefermances_model = PreferencesModel::builder()
            .transient_for(root)
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                PreferencesOutput::SetColorScheme(scheme) => AppMsg::SetColorScheme(scheme),
                PreferencesOutput::UpdataPreferences(preferences) => {
                    AppMsg::UpdataPreferences(preferences)
                }
            });

        let model = AppModel {
            is_fullscreen: false,
            sync_recording: Some(false),
            slaves: FactoryVecDeque::new(Grid::default(), sender.input_sender()),
            about_model,
            prefermances_model,
            tracker: 0,
        };

        let slave = model.get_slaves().widget();
        let widgets = view_output!();

        let mut app_group = RelmActionGroup::<AppActionGroup>::new();
        {
            let sender = sender.clone();
            let action_preferences: RelmAction<PreferencesAction> =
                RelmAction::new_stateless(move |_| sender.input(AppMsg::OpenPreferencesWindow));
            app_group.add_action(action_preferences);
        }
        {
            let sender = sender.clone();
            let action_about: RelmAction<AboutDialogAction> =
                RelmAction::new_stateless(move |_| sender.input(AppMsg::OpenAboutDialog));
            app_group.add_action(action_about);
        }
        root.insert_action_group("main", Some(&app_group.into_action_group()));

        for _ in 0..*model.prefermances_model.model().get_initial_slave_num() {
            sender.input(AppMsg::NewSlave)
        }
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::ComponentSender<Self>) {
        self.reset();

        use AppMsg::*;
        match message {
            UpdataPreferences(preferences) => {
                let slave = self.get_slaves();
                for i in 0..slave.len() {
                    slave.send(i, SlaveInput::UpdataPreferences(preferences.clone()));
                }
            }
            RemoveLastSlave => {
                let index = self.get_mut_slaves().len() - 1;
                self.get_mut_slaves().guard().remove(index);
            }
            NewSlave => {
                let preference = self.prefermances_model.model().clone();
                self.get_mut_slaves().guard().push_back(preference);
            }
            DestroySlave(index) => {
                let len = self.get_mut_slaves().len();
                if index < len {
                    self.get_mut_slaves().guard().remove(index);
                }
            }
            SetFullscreened(val) => self.set_is_fullscreen(val),
            OpenAboutDialog => self.about_model.sender().send(AboutMsg::Show).unwrap(),
            OpenPreferencesWindow => self
                .prefermances_model
                .sender()
                .send(PreferencesMsg::Show)
                .unwrap(),
            StopInputSystem => {}
            SetColorScheme(scheme) => StyleManager::default().set_color_scheme(match scheme {
                AppColorScheme::FollowSystem => ColorScheme::Default,
                AppColorScheme::Light => ColorScheme::ForceLight,
                AppColorScheme::Dark => ColorScheme::ForceDark,
            }),
            ToggleSyncRecording => {}
        }
    }
}

#[derive(EnumIter, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppColorScheme {
    FollowSystem,
    Light,
    Dark,
}

impl ToString for AppColorScheme {
    fn to_string(&self) -> String {
        match self {
            AppColorScheme::FollowSystem => "跟随系统",
            AppColorScheme::Light => "浅色",
            AppColorScheme::Dark => "暗色",
        }
        .to_string()
    }
}

impl Default for AppColorScheme {
    fn default() -> Self {
        Self::FollowSystem
    }
}

#[derive(Debug)]
pub enum AppMsg {
    UpdataPreferences(PreferencesModel),
    NewSlave,
    RemoveLastSlave,
    DestroySlave(usize),
    // DispatchInputEvent(InputEvent),
    SetColorScheme(AppColorScheme),
    ToggleSyncRecording,
    SetFullscreened(bool),
    OpenAboutDialog,
    OpenPreferencesWindow,
    StopInputSystem,
}

fn main() {
    gst::init().expect("无法初始化 GStreamer");
    // let model = AppModel {
    //     preferences: Rc::new(RefCell::new(PreferencesModel::load_or_default())),
    //
    // };
    // model.input_system.run();
    let app = RelmApp::new("org.jmu-stu.rov-host");
    app.run::<AppModel>(());
}
