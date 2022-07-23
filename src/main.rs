/*!
    A very simple application that show your name in a message box.
    See `basic` for the version without the derive macro
*/


extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
extern crate directories;

use nwd::NwgUi;
use nwg::NativeUi;
use directories::BaseDirs;
use mslnk::ShellLink;

use std::fs::create_dir_all;
use std::path::Path;
use std::{io, env};

const WINDOW_SIZE: (i32, i32) = (500, 350);
// Get program's file name
const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

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

            self.store_file(&file_path, file_name).unwrap();
            println!("Added {}", file_name);
        }

    }

    fn store_file(&self, file_path: &str, file_name: &str) -> io::Result<()> {
        /* Store shortcut to a given file in %APPDATA% */
        let base_dirs = BaseDirs::new().ok_or(io::Error::new(io::ErrorKind::Other, "Could not get base dirs"))?;
        let config_data = Path::new(&base_dirs.config_dir()).join(PROGRAM_NAME);

        create_dir_all(&config_data)?;
        let shortcut_path = Path::new(&config_data).join(file_name).with_extension("lnk");
        // TODO: get new version of mslnk to convert '.unwrap()' to '?'
        let sl = ShellLink::new(file_path).unwrap();
        sl.create_lnk(shortcut_path).unwrap();
        Ok(())
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");

    let _app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
