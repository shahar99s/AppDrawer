extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
extern crate directories;


use nwd::NwgUi;
use nwg::NativeUi;
use directories::BaseDirs;
use systemicons;

use std::path::PathBuf;
use std::fs::{create_dir_all, self};
use std::path::Path;
use std::process::Command;
use execute::Execute;
use std::{io, env};
use std::cell::RefCell;

const WINDOW_SIZE: (i32, i32) = (500, 350);
const APPS_RECT_SIZE: (f32, f32) = (375.0, 262.5);
const MIN_WINDOW_SIZE: [u32; 2] = [500, 350];

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");
const ICON_SIZE: i32 = 32;
const ICON_SIZE_U: u32 = 32;
const MAX_ICON_SIZE: i32 = 32;


#[derive(Default, NwgUi)]
pub struct GameManagerApp {
    app_path: RefCell<Vec<PathBuf>>,

    #[nwg_resource(source_file: Some("./assets/cog.ico"))]
    icon: nwg::Icon,

    #[nwg_resource(source_file: Some("./assets/bg.png"))]
    bitmap: nwg::Bitmap,

    #[nwg_resource(size: (ICON_SIZE, ICON_SIZE))]
    icons: nwg::ImageList,

    #[nwg_control(size: WINDOW_SIZE, position: (300, 300), title: "Game Manager", flags: "WINDOW|POPUP|RESIZABLE|VISIBLE", accept_files: true, icon: Some(&data.icon))]
    #[nwg_events(
        OnWindowClose: [nwg::stop_thread_dispatch()], 
        OnFileDrop: [GameManagerApp::drop_files(SELF, EVT_DATA)],
     )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 0, margin: [10,10,10,10])]
    layout: nwg::GridLayout,

    #[nwg_control(bitmap: Some(&data.bitmap), parent: window)]
    #[nwg_layout_item(layout: layout)]
    bg: nwg::ImageFrame,

    #[nwg_layout(parent: bg, justify_content: stretch::style::JustifyContent::Center, align_items: stretch::style::AlignItems::Center,
        flex_wrap: stretch::style::FlexWrap::Wrap,
)]
    apps_layout: nwg::FlexboxLayout,

    // It seems like 'ex_window_flags: LVS_EX_TRANSPARENTBKGND' is changing direction to rtl from ltr
    #[nwg_control(list_style: nwg::ListViewStyle::Icon, focus: true,
    ex_flags: nwg::ListViewExFlags::GRID, parent: bg,
    )]
    #[nwg_layout_item(layout: apps_layout, flex_grow: 0.7,
        size: stretch::geometry::Size{
            width: stretch::style::Dimension::Points(APPS_RECT_SIZE.0),
            height: stretch::style::Dimension::Points(APPS_RECT_SIZE.1)
        },
    )]
    #[nwg_events(
        OnListViewItemActivated: [GameManagerApp::launch_app(SELF, EVT_DATA)], 
     )]
    view: nwg::ListView,
}


impl GameManagerApp {
    fn launch_app(&self, data: &nwg::EventData) {
        let app_index = data.on_list_view_item_index();
        let app_path_vec = self.app_path.borrow_mut();
        let app_path = app_path_vec[app_index.0].as_path();
        let mut cmd = Command::new("cmd");        
        cmd.arg("/C");
        cmd.arg(app_path.to_str().unwrap());

        if cmd.execute_check_exit_status_code(0).is_err() {
           eprintln!("Failed to launch: {}", app_path.to_str().unwrap());
        }
        
    }

    fn drop_files(&self, data: &nwg::EventData) {
        let drop = data.on_file_drop();

        for file_path in drop.files() {
            let file_name = Path::new(& file_path).file_name().unwrap().to_str().unwrap();

            self.store_file(Path::new(&file_path), file_name).unwrap();
        }

    }

    fn copy_file(&self, file_path: &Path, shortcut_path: &Path) -> io::Result<()> {
        if shortcut_path.exists() || shortcut_path.with_extension("url").exists() {
            return Ok(());
        }

        match file_path.extension() {
            Some(extension) => {
                match extension.to_str() {
                    Some("url") => {
                        fs::copy(file_path, shortcut_path)?;
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
    
    /* Store shortcut to a given file in %APPDATA% */
    fn store_file(&self, file_path: &Path, file_name: &str) -> io::Result<()> {
        let base_dirs = BaseDirs::new().ok_or(io::Error::new(io::ErrorKind::Other, "Could not get base dirs"))?;
        let config_data = Path::new(&base_dirs.config_dir()).join(PROGRAM_NAME);

        create_dir_all(&config_data)?;
        let shortcut_path = &config_data.join(file_name);
        if self.copy_file(file_path, shortcut_path).is_err(){
            fs::copy(file_path, shortcut_path)?;
        }

        let icon = systemicons::get_icon(&file_path.to_str().unwrap(), MAX_ICON_SIZE).unwrap();
        self.add_icon(icon, file_name, file_path.to_owned());
        Ok(())
    }

    fn add_icon(&self, icon: Vec<u8>, file_name: &str, file_path: PathBuf) {
        // write icon flip func
        let mut bitmap = nwg::Bitmap::default();
        nwg::Bitmap::builder().source_bin(Some(&icon)).build(&mut bitmap).unwrap();

        self.icons.add_bitmap(&bitmap);
        self.app_path.borrow_mut().push(file_path);
        self.view.set_image_list(Some(&self.icons), nwg::ListViewImageListType::Normal);
        let len: i32 = (self.icons.len() - 1).try_into().unwrap();
        self.view.insert_item(nwg::InsertListViewItem
        {
            index: None,
            column_index: 0,
            text: Some(file_name.to_string()),
            image: Some(len),
        });
    }

    // Show the icon of each app in %appdata%
    fn display_apps(&self) -> io::Result<()> {
        let base_dirs = BaseDirs::new().ok_or(io::Error::new(io::ErrorKind::Other, "Could not get base dirs"))?;
        let config_data = Path::new(&base_dirs.config_dir()).join(PROGRAM_NAME);

        // iterate through %appdata%\Game Manager\
        for path in fs::read_dir(&config_data)? {
            let path = path?;
            let file_name = &path.file_name();

            let icon_path: PathBuf;
            if fs::symlink_metadata(&path.path())?.file_type().is_symlink() {
                icon_path = fs::read_link(&path.path()).unwrap();
            }
            else {
                icon_path = path.path();
            }
            let icon = systemicons::get_icon(icon_path.to_str().unwrap(), MAX_ICON_SIZE).unwrap();
            self.add_icon(icon, file_name.to_str().unwrap(), path.path());
        }

        Ok(())
    }

}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");

    let _app = GameManagerApp::build_ui(Default::default()).expect("Failed to build UI");
    _app.display_apps().unwrap();
    nwg::dispatch_thread_events();
}
