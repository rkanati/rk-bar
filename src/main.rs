mod bar;
mod modules;
mod output;

use {
    crate::output::OutputManager,
    std::{
        error::Error,
        result::Result,
    },
    gdk::{Display, Monitor},
    gtk,
};

fn main() -> Result<(), Box<dyn Error>> {
    gtk::init().unwrap();

    let disp = Display::get_default().unwrap();

    let output_mgr = OutputManager::new(&disp)?;


    gtk::main();
    Ok(())
}

