use {
    crate::modules::{
        Module,
        sway,
        Clock,
    },
    std::{
        error::Error,
        result::Result,
    },
    gdk::Monitor,
    gtk::{
        prelude::*,
        Box as WidgetBox,
        CssProvider,
        Orientation,
        StyleContext,
        WidgetExt,
        Window,
        WindowType,
    },
    gtk_layer_shell as gtkls,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom
}

pub enum BarSection {
    Left,
    Center,
    Right
}

fn prep_layer_shell(w: &Window, monitor: &Monitor, edge: Edge) {
    // NOTE if this starts segfaulting, esp. in a wl_list operation, experience notes that this is
    //      probably due to earlier wayland-client api misuse.
    gtkls::init_for_window(w);
    gtkls::set_monitor(w, monitor);
    gtkls::set_layer(w, gtkls::Layer::Top);
    gtkls::auto_exclusive_zone_enable(w);

    gtkls::set_anchor(w, gtkls::Edge::Left,  true);
    gtkls::set_anchor(w, gtkls::Edge::Right, true);

    let edge = match edge {
        Edge::Bottom => gtkls::Edge::Bottom,
        Edge::Top    => gtkls::Edge::Top
    };

    gtkls::set_anchor(w, edge, true);
}

pub struct Bar {
//  window:     Window,
    master_box: WidgetBox,
    left_box:   WidgetBox,
    right_box:  WidgetBox,
    modules:    Vec<Box<dyn Module>>,
}

impl Bar {
    pub fn new(monitor: &Monitor, edge: Edge) -> Result<Bar, Box<dyn Error>> {
        let window = Window::new(WindowType::Toplevel);
        prep_layer_shell(&window, monitor, edge);

        window.set_resizable(true);
        window.set_decorated(false);
        window.set_title("rk-bar");
        window.set_widget_name("bar");

        let css = CssProvider::new();
        css.load_from_path("rk-bar.css")?;
        StyleContext::add_provider_for_screen(&window.get_screen().unwrap(), &css, 800);

        let master_box = WidgetBox::new(Orientation::Horizontal, 0);
        master_box.set_homogeneous(false);
        window.add(&master_box);

        let left_box = WidgetBox::new(Orientation::Horizontal, 0);
        left_box.set_homogeneous(false);
        master_box.pack_start(&left_box, false, false, 0);

        let right_box = WidgetBox::new(Orientation::Horizontal, 0);
        right_box.set_homogeneous(false);
        master_box.pack_end(&right_box, false, false, 0);

        window.show_all();
        Ok(Bar { /*window,*/ master_box, left_box, right_box, modules: Vec::new() })
    }

    fn add_module(&mut self, section: BarSection, module: impl Module + 'static) {
        let module = Box::new(module);

        match section {
            BarSection::Left   => self.left_box.pack_start(&module.get_widget(), false, false, 0),
            BarSection::Right  => self.right_box.pack_start(&module.get_widget(), false, false, 0),
            BarSection::Center => self.master_box.set_center_widget(Some(&module.get_widget()))
        }

        self.modules.push(module);
    }

    pub fn create(monitor: &Monitor) -> Result<Bar, Box<dyn Error>> {
        let mut bar = Bar::new(monitor, Edge::Bottom)?;

        let mut sway_builder = sway::Connection::build()?;

        let workspaces = sway::Workspaces::new(&mut sway_builder);
        bar.add_module(BarSection::Left, workspaces);

        let title = sway::Title::new(&mut sway_builder);
        bar.add_module(BarSection::Center, title);

        let clock = Clock::new();
        bar.add_module(BarSection::Right, clock);

        let _sway = sway_builder.connect()?;

        Ok(bar)
    }
}

