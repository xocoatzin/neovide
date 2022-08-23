#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Test naming occasionally uses camelCase with underscores to separate sections of
// the test name.
#![cfg_attr(test, allow(non_snake_case))]
#[macro_use]
extern crate neovide_derive;

#[macro_use]
extern crate clap;

mod bridge;
mod channel_utils;
mod cmd_line;
mod dimensions;
mod editor;
mod error_handling;
mod event_aggregator;
mod frame;
mod redraw_scheduler;
mod renderer;
mod running_tracker;
mod settings;
mod window;

#[cfg(target_os = "windows")]
mod windows_utils;

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate lazy_static;

use std::env::args;

#[cfg(not(test))]
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};
use log::trace;

use bridge::start_bridge;
use cmd_line::CmdLineSettings;
use editor::start_editor;
use settings::SETTINGS;

use window::{
    create_window, 
    KeyboardSettings, 
    WindowSettings,
};
use renderer::{
    RendererSettings,
    Renderer,
};

pub use channel_utils::*;
pub use event_aggregator::*;
pub use running_tracker::*;
#[cfg(target_os = "windows")]
pub use windows_utils::*;

fn main() {
    //Will exit if -h or -v
    if let Err(err) = cmd_line::handle_command_line_arguments(args().collect()) {
        eprintln!("{}", err);
        return;
    }

    #[cfg(not(test))]
    init_logger();

    trace!("Neovide version: {}", crate_version!());

    WindowSettings::register();
    RendererSettings::register();
    KeyboardSettings::register();

    create_window();
}

#[cfg(not(test))]
pub fn init_logger() {
    let settings = SETTINGS.get::<CmdLineSettings>();

    let logger = if settings.log_to_file {
        Logger::try_with_env_or_str("neovide")
            .expect("Could not init logger")
            .log_to_file(FileSpec::default())
            .rotate(
                Criterion::Size(10_000_000),
                Naming::Timestamps,
                Cleanup::KeepLogFiles(1),
            )
            .duplicate_to_stderr(Duplicate::Error)
    } else {
        Logger::try_with_env_or_str("neovide = error").expect("Cloud not init logger")
    };

    logger.start().expect("Could not start logger");
}
