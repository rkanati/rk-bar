
pub mod sway;

pub mod clock;
pub use clock::Clock;

use gtk::Widget;

pub trait Module {
    fn get_widget(&self) -> Widget;
}

