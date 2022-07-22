/*!
    A very simple application that show your name in a message box.
    See `basic` for the version without the derive macro
*/


extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

use std::path::Path;

const WINDOW_SIZE: (i32, i32) = (500, 350);

#[derive(Default, NwgUi)]
pub struct BasicApp {
    #[nwg_resource(source_file: Some("./assets/cog.ico"))]
    icon: nwg::Icon,

    #[nwg_resource(source_file: Some("./assets/bg.png"))]
    bitmap: nwg::Bitmap,

    #[nwg_control(size: WINDOW_SIZE, position: (300, 300), title: "Game Manager", flags: "WINDOW|RESIZABLE|VISIBLE", accept_files: true, icon: Some(&data.icon))]
    #[nwg_events(
        OnResize: [BasicApp::resize],
        OnWindowMaximize: [BasicApp::resize], 
        OnWindowMinimize: [BasicApp::resize],
        OnWindowClose: [nwg::stop_thread_dispatch()], 
        OnFileDrop: [BasicApp::drop_files(SELF, EVT_DATA)],
     )]
    window: nwg::Window,

    #[nwg_control(bitmap: Some(&data.bitmap), size: WINDOW_SIZE, parent: window)]
    img: nwg::ImageFrame,
}

impl BasicApp {
    fn resize(&self) {
        let (x, y) = self.window.size();
        self.img.set_size(x, y);
    }

    fn drop_files(&self, data: &nwg::EventData) {
        let drop = data.on_file_drop();

        for file_path in drop.files() {
            let file_name = Path::new(& file_path).file_name().unwrap().to_str().unwrap();

            println!("{}", file_name);
        }

    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");

    let _app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
