use adw::{prelude::*, AboutWindow};
use relm4::{
    gtk::{Inhibit, License},
    prelude::*,
};

#[tracker::track]
pub struct AboutModel {
    is_show: bool,
}

#[derive(Debug)]
pub enum AboutMsg {
    Show,
    Hidden,
}

#[relm4::component(pub)]
impl SimpleComponent for AboutModel {
    type Init = ();
    type Input = AboutMsg;
    type Output = ();

    view! {
        #[root]
        AboutWindow {
            set_modal: true,
            #[track = "model.changed(AboutModel::is_show())"]
            set_visible: *model.get_is_show(),
            connect_close_request[sender] => move |_| {
                sender.input(AboutMsg::Hidden);
                Inhibit(true)
            },
            set_website: "https://github.com/BohongHuang/rov-host",
            set_developers: &["黄博宏 https://bohonghuang.github.io", "彭剑锋 https://qff233.com"],
            set_application_name: "水下机器人上位机",
            set_copyright: "© 2021-2023 集美大学水下智能创新实验室",
            set_comments: "跨平台的水下机器人上位机程序",
            set_application_icon: "input-gaming",
            set_version: env!("CARGO_PKG_VERSION"),
            set_license_type: License::Gpl30,
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AboutModel {
            is_show: false,
            tracker: 0,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match message {
            AboutMsg::Show => self.set_is_show(true),
            AboutMsg::Hidden => self.set_is_show(false),
        }
    }
}
