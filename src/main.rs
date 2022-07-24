extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
extern crate directories;

use nwd::NwgUi;
use nwg::NativeUi;
use directories::BaseDirs;

use std::fs::{create_dir_all, self};
use std::path::Path;
use std::{io, env};

const WINDOW_SIZE: (i32, i32) = (500, 350);
const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Default, NwgUi)]
pub struct GameManagerApp {
    #[nwg_resource(source_file: Some("./assets/cog.ico"))]
    icon: nwg::Icon,

    #[nwg_resource(source_file: Some("./assets/bg.png"))]
    bitmap: nwg::Bitmap,

    #[nwg_control(size: WINDOW_SIZE, position: (300, 300), title: "Game Manager", flags: "WINDOW|RESIZABLE|VISIBLE", accept_files: true, icon: Some(&data.icon))]
    #[nwg_events(
        OnResize: [GameManagerApp::resize],
        OnWindowMaximize: [GameManagerApp::resize], 
        OnWindowMinimize: [GameManagerApp::resize],
        OnWindowClose: [nwg::stop_thread_dispatch()], 
        OnFileDrop: [GameManagerApp::drop_files(SELF, EVT_DATA)],
     )]
    window: nwg::Window,

    #[nwg_control(bitmap: Some(&data.bitmap), size: WINDOW_SIZE, parent: window)]
    img: nwg::ImageFrame,
}

impl GameManagerApp {
    fn resize(&self) {
        let (x, y) = self.window.size();
        self.img.set_size(x, y);
    }

    fn drop_files(&self, data: &nwg::EventData) {
        let drop = data.on_file_drop();

        for file_path in drop.files() {
            let file_name = Path::new(& file_path).file_name().unwrap().to_str().unwrap();

            self.store_file(Path::new(&file_path), file_name).unwrap();
            println!("Added {}", file_name);
        }

    }

    /* Store shortcut to a given file in %APPDATA% */
    fn store_file(&self, file_path: &Path, file_name: &str) -> io::Result<()> {
        let base_dirs = BaseDirs::new().ok_or(io::Error::new(io::ErrorKind::Other, "Could not get base dirs"))?;
        let config_data = Path::new(&base_dirs.config_dir()).join(PROGRAM_NAME);

        create_dir_all(&config_data)?;
        let shortcut_path = config_data.join(file_name).with_extension("lnk");
        if shortcut_path.exists() || shortcut_path.with_extension("url").exists() {
            return Ok(());
        }

        match file_path.extension() {
            Some(extension) => {
                println!("{}", extension.to_str().unwrap());
                match extension.to_str() {
                    Some("url") => {
                        fs::copy(file_path, config_data.join(file_name))?;
                    }
                    _ => {
                        fs::hard_link(file_path, shortcut_path)?;
                    }
                }
            }
            None => {
                fs::hard_link(file_path, shortcut_path)?;
            }
        }
        Ok(())
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");

    let _app = GameManagerApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
