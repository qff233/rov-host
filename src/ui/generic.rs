// use relm4::{
//     self,
//     gtk::{self, prelude::*, MessageDialog},
//     view,
// };

// pub fn error_message<T>(title: &str, msg: &str, window: Option<&T>) -> MessageDialog
// where
//     T: IsA<gtk::Window>,
// {
//     // view! {
//     //     dialog = MessageDialog {
//     //         set_message_type: gtk::MessageType::Error,
//     //         set_text: Some(msg),
//     //         set_title: Some(title),
//     //         set_modal: true,
//     //         set_transient_for: window,
//     //         add_button: args!("确定", ResponseType::Ok),
//     //         connect_response => |dialog, _response| {
//     //             dialog.destroy();
//     //         }
//     //     }
//     // }
//     dialog.show();
//     dialog
// }
