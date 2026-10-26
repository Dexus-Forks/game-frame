//! What every view shares: the session (the game, its settings, the event log), the player's
//! library when there is one, the look from `library.json`, and a revision number that goes up
//! after each change so the views draw again.

use dioxus::prelude::*;
use gameframe_frame::{Session, Settings};
use gameframe_library::{Hooks, Library, Spec, Store};
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// The library as the views hold it: any store, any hooks.
pub type AnyLibrary = Library<Box<dyn Store>, Box<dyn Hooks>>;

/// Called with the settings whenever they change, to save them: a file on the desktop,
/// browser storage on the web.
pub type SaveSettings = Rc<dyn Fn(&Settings)>;

/// Called with `true` when the frame should be on screen and `false` when the game should have
/// it: the desktop shows or hides its window.
pub type ShowWindow = Rc<dyn Fn(bool)>;

/// Everything the frame needs to start. Make it in `main`, then launch [`crate::Frame`] with it.
#[derive(Clone)]
pub struct FrameSetup {
    pub session: Rc<RefCell<Session>>,
    /// The look and the library's sections, from the project's `library.json`.
    pub spec: Rc<Spec>,
    /// The player's library. `None` hides the Library tab.
    pub library: Option<Rc<AnyLibrary>>,
    pub save_settings: Option<SaveSettings>,
    /// How often the game is polled, in milliseconds.
    pub tick_ms: u32,
    /// The game's own stylesheet, drawn after the frame's: the third way to restyle, as
    /// `theme.css` is on the HTML library page.
    pub css: Option<Rc<str>>,
    pub show_window: Option<ShowWindow>,
    /// Leave the program, after `frame.exit`. `None` hides "Exit to desktop".
    pub exit: Option<Rc<dyn Fn()>>,
    /// Start the game as soon as the frame is ready: for a program whose frame is turned off
    /// at start, which opens its window hidden.
    pub start_at_once: bool,
    /// The game's own themes: name, label and stylesheet. `"preset"` and `"presets"` in
    /// `library.json` name them as they name the built-in ones, and a game theme of the same
    /// name wins.
    pub themes: Vec<(Rc<str>, Rc<str>, Rc<str>)>,
}

impl FrameSetup {
    pub fn new(session: Session, spec: Spec) -> FrameSetup {
        FrameSetup { session: Rc::new(RefCell::new(session)), spec: Rc::new(spec), library: None, save_settings: None, tick_ms: 50, css: None, show_window: None, exit: None, start_at_once: false, themes: Vec::new() }
    }

    pub fn with_library(mut self, library: AnyLibrary) -> FrameSetup {
        self.library = Some(Rc::new(library));
        self
    }

    pub fn with_css(mut self, css: impl Into<Rc<str>>) -> FrameSetup {
        self.css = Some(css.into());
        self
    }

    /// A theme of the game's own, drawn in place of a built-in one when `library.json`'s
    /// `"preset"` or the player's choice names it, such as a look taken from the game's menus.
    pub fn with_theme(mut self, name: &str, label: &str, css: impl Into<Rc<str>>) -> FrameSetup {
        self.themes.push((name.into(), label.into(), css.into()));
        self
    }

    /// Show or hide the frame's window when the session says (see `Session::shown`): after
    /// "Back to the game", when the game asks for the frame with `frame.open`, and when the game
    /// stops.
    pub fn with_window(mut self, show: impl Fn(bool) + 'static) -> FrameSetup {
        self.show_window = Some(Rc::new(show));
        self
    }

    /// Leave the program when the session says (see `Session::wants_exit`): the player chose
    /// "Exit to desktop", or the game ended by itself while `frame.on-exit` is `"desktop"`.
    pub fn with_exit(mut self, exit: impl Fn() + 'static) -> FrameSetup {
        self.exit = Some(Rc::new(exit));
        self
    }

    pub fn start_at_once(mut self, on: bool) -> FrameSetup {
        self.start_at_once = on;
        self
    }

    pub fn with_save(mut self, save: impl Fn(&Settings) + 'static) -> FrameSetup {
        self.save_settings = Some(Rc::new(save));
        self
    }
}

impl PartialEq for FrameSetup {
    fn eq(&self, other: &FrameSetup) -> bool {
        Rc::ptr_eq(&self.session, &other.session)
    }
}

/// The handle views use. Every change goes through it, so the views draw again and the
/// settings are saved.
#[derive(Clone, Copy)]
pub struct FrameContext {
    setup: Signal<FrameSetup>,
    rev: Signal<u64>,
}

impl FrameContext {
    pub(crate) fn provide(setup: FrameSetup) -> FrameContext {
        let setup = use_signal(|| setup);
        let rev = use_signal(|| 0u64);
        use_context_provider(|| FrameContext { setup, rev })
    }

    /// Read the session. Reading this subscribes the view to changes.
    pub fn with<R>(&self, f: impl FnOnce(&Session) -> R) -> R {
        let _ = (self.rev)();
        let setup = self.setup.peek();
        let session = setup.session.borrow();
        f(&session)
    }

    /// Change the session, save the settings if they changed, and draw again.
    pub fn update<R>(&self, f: impl FnOnce(&mut Session) -> R) -> R {
        let out = self.change(f);
        let mut rev = self.rev;
        rev += 1;
        out
    }

    /// Poll the game. Draw again only when it sent something.
    pub(crate) fn tick(&self) {
        let sent = self.change(|s| {
            let before = s.log().back().map(|e| e.id);
            s.tick();
            s.log().back().map(|e| e.id) != before
        });
        if sent {
            let mut rev = self.rev;
            rev += 1;
        }
    }

    /// Run `f`, save the settings if they changed, and show or hide the window if the session
    /// says so.
    fn change<R>(&self, f: impl FnOnce(&mut Session) -> R) -> R {
        let setup = self.setup.peek().clone();
        let (out, changed, shown, exit) = {
            let mut session = setup.session.borrow_mut();
            let (was, leaving) = (session.shown(), session.wants_exit());
            let out = f(&mut session);
            let shown = (session.shown() != was).then(|| session.shown());
            let exit = session.wants_exit() && !leaving;
            (out, session.settings_changed(), shown, exit)
        };
        if changed {
            if let Some(save) = &setup.save_settings {
                save(setup.session.borrow().settings());
            }
        }
        if let (Some(shown), Some(show)) = (shown, &setup.show_window) {
            show(shown);
        }
        if let (true, Some(leave)) = (exit, &setup.exit) {
            leave();
        }
        out
    }

    pub fn spec(&self) -> Rc<Spec> {
        self.setup.peek().spec.clone()
    }

    pub fn library(&self) -> Option<Rc<AnyLibrary>> {
        self.setup.peek().library.clone()
    }

    /// The revision, for resources that must load again after a change.
    pub fn revision(&self) -> u64 {
        (self.rev)()
    }

    pub(crate) fn css(&self) -> Option<Rc<str>> {
        self.setup.peek().css.clone()
    }

    /// A theme's stylesheet by name: the game's own first, then the built-in ones.
    pub fn theme_css(&self, name: &str) -> Option<Rc<str>> {
        let setup = self.setup.peek();
        match setup.themes.iter().find(|(n, _, _)| &**n == name) {
            Some((_, _, css)) => Some(css.clone()),
            None => crate::theme(name).map(Rc::from),
        }
    }

    /// A theme's label: the game's own first, then the built-in ones.
    pub fn theme_label(&self, name: &str) -> String {
        let setup = self.setup.peek();
        match setup.themes.iter().find(|(n, _, _)| &**n == name) {
            Some((_, label, _)) => label.to_string(),
            None => crate::theme_label(name).to_string(),
        }
    }

    /// Whether the program can leave for the desktop.
    pub(crate) fn has_exit(&self) -> bool {
        self.setup.peek().exit.is_some()
    }

    /// Whether the program shows and hides a window for the frame.
    pub(crate) fn has_window(&self) -> bool {
        self.setup.peek().show_window.is_some()
    }

    pub(crate) fn starts_at_once(&self) -> bool {
        self.setup.peek().start_at_once
    }

    pub(crate) fn tick_ms(&self) -> u32 {
        self.setup.peek().tick_ms
    }

    /// A setting's saved value.
    pub fn setting(&self, key: &str) -> Option<Value> {
        self.with(|s| s.settings().get(key).cloned())
    }
}

pub fn use_frame() -> FrameContext {
    use_context::<FrameContext>()
}

/// Wait, on either platform.
pub(crate) async fn sleep(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}
