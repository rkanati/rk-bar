use {
    super::{
        Connection as Sway,
        ConnectionBuilder as SwayBuilder,
        Event,
        EventType,
    },
    crate::modules::Module,
    gtk::{
        Box as WidgetBox,
        BoxExt,
        Button,
        ButtonExt,
        ContainerExt,
        Orientation,
        ReliefStyle,
        StyleContextExt,
        Widget,
        WidgetExt,
    },
};

#[derive(Clone)]
pub struct Workspaces {
    button_box: WidgetBox,
}

impl Workspaces {
    pub fn new(builder: &mut SwayBuilder) -> Workspaces {
        // make a box to put workspace buttons in
        let button_box = WidgetBox::new(Orientation::Horizontal, 0);
        button_box.set_homogeneous(false);
        button_box.show();
        button_box.set_widget_name("workspaces");

        let me = Workspaces { button_box };

        builder.subscribe(
            &[EventType::Workspace],
            {   let me = me.clone();
                move |conn, event| match event {
                    Event::Workspace(_) => me.refresh(&conn),
                    _ => { }
                }
            }
        );

        me.refresh(builder);
        me
    }

    pub fn refresh(&self, sway: &Sway) {
        self.button_box.foreach(|button| { self.button_box.remove(button); });

        let workspaces = match sway.get_workspaces() {
            Ok(ws) => ws,
            Err(_) => return
        };

        for ws in workspaces {
            let button = Button::with_label(&ws.name);
            button.set_relief(ReliefStyle::None);
            button.set_can_focus(false);
            button.show();

            let sctx = button.get_style_context();
            if ws.focused     { sctx.add_class("focused"); }
            else if ws.urgent { sctx.add_class("urgent"); }

            if ws.visible { sctx.add_class("visible"); }

            button.connect_clicked({
                let sway = sway.clone();
                let name = ws.name.clone();
                move |_| {
                    if let Err(e) = sway.run_commands(&format!("workspace {}", name)) {
                        eprintln!("Error switching workspace: {}", e);
                    }
                }
            });

            self.button_box.pack_start(&button, false, false, 0);
            self.button_box.reorder_child(&button, ws.num + 1);
        }
    }
}

impl Module for Workspaces {
    fn get_widget(&self) -> Widget {
        use glib::object::Cast;
        self.button_box.clone().upcast()
    }
}

