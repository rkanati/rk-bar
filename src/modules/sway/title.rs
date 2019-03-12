use {
    super::{
        ConnectionBuilder,
        Event,
        EventType,
    },
    crate::modules::Module,
    swayipc::reply::WindowChange,
    gtk::{
        Label,
        LabelExt,
        StyleContextExt,
        Widget,
        WidgetExt,
    },
};

#[derive(Clone)]
pub struct Title {
    label: Label,
}

impl Title {
    pub fn new(conn_builder: &mut ConnectionBuilder) -> Title {
        let label = Label::new(None);
        label.show();
        label.set_widget_name("title");

        conn_builder.subscribe(
            &[EventType::Window],
            {   let label = label.clone();
                move |_, event| if let Event::Window(info) = event {
                    let update_text = || {
                        let title = info.container.name.clone()
                            .unwrap_or("".to_string());
                        label.set_label(&title);
                    };

                    match info.change {
                        WindowChange::Focus => {
                            update_text();
                        },

                        WindowChange::Title if info.container.focused => {
                            update_text();
                        },

                        WindowChange::Urgent => {
                            let sctx = label.get_style_context();
                            let class = "urgent";
                            if info.container.urgent { sctx.add_class(class); }
                            else                     { sctx.remove_class(class); }
                        },

                        _ => { }
                    }
                }
            }
        );

        Title { label }
    }
}

impl Module for Title {
    fn get_widget(&self) -> Widget {
        use glib::object::Cast;
        self.label.clone().upcast()
    }
}

