pub mod workspaces;
pub use workspaces::Workspaces;

pub mod title;
pub use title::Title;

use {
    std::{
        cell::RefCell,
        collections::HashMap,
        ops::DerefMut,
        sync::Arc,
        thread,
    },
    swayipc::{
        self,
        Connection as Sway,
        EventType,
        reply::{CommandOutcome, Event, Workspace},
    },
    glib::{Continue, MainContext},
};

pub type Result<T> = swayipc::Fallible<T>;

#[derive(Clone)]
pub struct Connection(Arc<RefCell<Sway>>);

impl Connection {
    pub fn build() -> Result<ConnectionBuilder> {
        ConnectionBuilder::new()
    }

    fn new() -> Result<Connection> {
        let conn = Sway::new()?;
        Ok(Connection(Arc::new(RefCell::new(conn))))
    }

    pub fn with_connection<R>(&self, f: impl FnOnce(&mut Sway) -> R) -> R {
        f(self.0.borrow_mut().deref_mut())
    }

    pub fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        self.with_connection(|conn| conn.get_workspaces())
    }

    pub fn run_commands(&self, commands: &str) -> Result<Vec<CommandOutcome>> {
        self.with_connection(|conn| conn.run_command(commands))
    }
}

//impl Drop for Connection {
//    fn drop(&mut self) {
//        // FIXME: we have to orphan the thread here -- i3ipc-rs uses an iterator instead of a
//        // channel, so we have no way to wake it up
//    }
//}

type Subscriber = Box<dyn FnMut(&Connection, &Event)>;

pub struct ConnectionBuilder {
    conn: Connection,
    subs: HashMap<EventType, Vec<Subscriber>>,
}

impl std::ops::Deref for ConnectionBuilder {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.conn
    }
}

impl ConnectionBuilder {
    fn new() -> Result<ConnectionBuilder> {
        Ok(ConnectionBuilder {
            conn: Connection::new()?,
            subs: HashMap::new(),
        })
    }

    pub fn subscribe(
        &mut self,
        events: &[EventType],
        f: impl FnMut(&Connection, &Event) + Clone + 'static)
    {
        let subscriber = Box::new(f);

        for event in events {
            let set = self.subs.entry(*event).or_insert(Vec::new());
            set.push(subscriber.clone());
        }
    }

    pub fn connect(self) -> Result<Connection> {
        let (chan_tx, chan_rx) = MainContext::channel(Default::default());

        let event_types: Vec<EventType> = self.subs
            .keys()
            .copied()
            .collect();

        chan_rx.attach(None, {
            let conn = self.conn.clone();
            let mut subs = self.subs;

            move |e| {
                // sigh
                use EventType::*;
                let sub_type = match e {
                    Event::Workspace(_)       => Workspace,
                    Event::Mode(_)            => Mode,
                    Event::Window(_)          => Window,
                    Event::BarConfigUpdate(_) => BarConfigUpdate,
                    Event::Binding(_)         => Binding,
                    Event::Shutdown(_)        => Shutdown,
                    Event::Tick(_)            => Tick,
                    Event::BarStateUpdate(_)  => BarStateUpdate,
                    Event::Input(_)           => Input
                };

                if let Some(subscribers) = subs.get_mut(&sub_type) {
                    for f in subscribers {
                        f(&conn, &e);
                    }
                }

                Continue(true)
            }
        });

        thread::spawn({
            let events = Sway::new()?.subscribe(&event_types)?;
            move || {
                for e in events {
                    let e = e.unwrap();
                    if let Err(_) = chan_tx.send(e) {
                        break;
                    }
                }
            }
        });

        Ok(self.conn)
    }
}

