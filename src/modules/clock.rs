
use {
    crate::modules::Module,
    std::thread,
    chrono::{
        self,
        Duration,
        offset::Local as LocalTZ,
        Timelike,
    },
    gtk::{
        Label,
        LabelExt,
        Widget,
        WidgetExt,
    },
    glib::{Continue, MainContext},
};

type LocalDateTime = chrono::DateTime<LocalTZ>;

#[derive(Clone)]
pub struct Clock(Label);

impl Clock {
    pub fn new() -> Clock {
        let label = Label::new(None);
        label.show();

        let (tx, rx) = MainContext::channel(Default::default());

        rx.attach(None, {
            let label = label.clone();
            move |now: LocalDateTime| {
                dbg!(now);
                let text = format!("{}", now.format("%F %a %H%M"));
                label.set_label(&text);
                Continue(true)
            }
        });

        tx.send(LocalTZ::now()).unwrap();

        thread::spawn(move || {
            loop {
                let now = LocalTZ::now();
                if tx.send(now).is_err() { break; }

                let next = (now + Duration::minutes(1))
                    .with_second(0)
                    .unwrap();

                if let Some(duration) = (next - now).to_std().ok() {
                    thread::sleep(duration);
                }
            }
        });

        Clock(label)
    }
}

impl Module for Clock {
    fn get_widget(&self) -> Widget {
        use glib::object::Cast;
        self.0.clone().upcast()
    }
}

