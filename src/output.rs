
use {
    crate::bar::Bar,
    std::{cell::RefCell, sync::Arc},
    //wayland_protocols::unstable::xdg_output::v1::client::{
    //    zxdg_output_manager_v1::ZxdgOutputManagerV1 as XdgOutputManager,
    //    zxdg_output_v1::ZxdgOutputV1 as XdgOutput,
    //},
    //wayland_client::{self as wl_cli, GlobalManager},
};

/*mod ffi {
    use {
        super::*,
        gdk_sys::{GdkDisplay, GdkMonitor},
        glib::translate::ToGlibPtr,
        wayland_client::{
            self as wl_cli,
            //protocol::wl_output::WlOutput,
            sys::client::wl_display,
        },
    };

    #[repr(C)] struct wl_output(std::ffi::c_void);
    //#[repr(C)] struct wl_display(std::ffi::c_void);

    extern "C" {
        fn gdk_wayland_monitor_get_wl_output(_: *mut GdkMonitor) -> *mut wl_output;
        fn gdk_wayland_display_get_wl_display(_: *mut GdkDisplay) -> *mut wl_display;
    }

    pub fn swayname_for_monitor(mon: &gdk::Monitor, mgr: &XdgOutputManager)
        //-> Option<wl_cli::Main<XdgOutput>>
        -> Option<String>
    {
        let mon: *mut GdkMonitor = mon.to_glib_none().0;
        let wlout = unsafe { gdk_wayland_monitor_get_wl_output(mon) };
        if wlout.is_null() { return None; }
        let wlout = unsafe { wl_cli::Proxy::from_c_ptr(wlout as *mut _) };
        let xdgout = mgr.get_xdg_output(&wlout.into());

        let swayname = Arc::new(std::cell::Cell::new(None));
        xdgout.quick_assign({
            let swayname = swayname.clone();
            move |_, e, _| {
                type Event = <XdgOutput as wl_cli::Interface>::Event;
                if let Event::Name { name } = e {
                    swayname.set(Some(name));
                }
            }
        });
        // XXX rountrip here?

        swayname.take()
    }

    pub fn display_wl_for_gdk(disp: &gdk::Display) -> Option<wl_cli::Display> {
        let disp: *mut GdkDisplay = disp.to_glib_none().0;
        let wldisp = unsafe { gdk_wayland_display_get_wl_display(disp) };
        if wldisp.is_null() { None }
        else { Some(unsafe { wl_cli::Display::from_external_display(wldisp) }) }
    }
}*/

//use ffi::*;

pub struct Output {
    monitor: gdk::Monitor,
    //name:    String,
    bar:     Bar,
}

impl Output {
    fn new(monitor: &gdk::Monitor) -> Output { //, name: String) -> Output {
        let bar = Bar::create(monitor).unwrap();
        Output { monitor: monitor.clone(), /*name,*/ bar }
    }
}

struct Inner {
    outputs: Vec<Output>,
    //xdg:     wl_cli::Main<XdgOutputManager>,
}

impl Inner {
    fn add_monitor(&mut self, monitor: &gdk::Monitor) {
        //let name = None;//swayname_for_monitor(monitor, &self.xdg);
        self.outputs.push(Output::new(monitor));//, name.unwrap_or("???".to_owned())));
    }

    fn remove_monitor(&mut self, monitor: &gdk::Monitor) {
        self.outputs.retain(|output| output.monitor == *monitor);
    }
}

pub struct OutputManager(Arc<RefCell<Inner>>);

impl OutputManager {
    pub fn new(disp: &gdk::Display) -> Result<OutputManager, Box<dyn std::error::Error>> {
        /*let xdg: wl_cli::Main<XdgOutputManager> = {
            let wl_disp = display_wl_for_gdk(disp).unwrap();
            let mut eq = wl_disp.create_event_queue();

            let attached_disp = wl_disp.attach(eq.token());
            let globals = GlobalManager::new(&attached_disp);
            eq.sync_roundtrip(&mut (), |_, _, _| { }).unwrap();

            globals.instantiate_exact(1)?
        };*/

        let inner = Inner { outputs: Vec::new() };//, xdg };
        let inner = Arc::new(RefCell::new(inner));

        // FIXME: GDK doesn't provide a race-free way to enumerate monitors, afaik
        {
            let mut i = 0;
            while let Some(monitor) = disp.get_monitor(i) {
                i += 1;
                inner.borrow_mut().add_monitor(&monitor);
            }
        }

        disp.connect_monitor_added({
            let inner = inner.clone();
            move |_disp, monitor| inner.borrow_mut().add_monitor(monitor)
        });

        disp.connect_monitor_removed({
            let inner = inner.clone();
            move |_disp, monitor| inner.borrow_mut().remove_monitor(monitor)
        });

        Ok(OutputManager(inner))
    }
}

