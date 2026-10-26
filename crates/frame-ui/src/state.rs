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
}

impl FrameSetup {
    pub fn new(session: Session, spec: Spec) -> FrameSetup {
        FrameSetup { session: Rc::new(RefCell::new(session)), spec: Rc::new(spec), library: None, save_settings: None, tick_ms: 50, css: None }
    }

    pub fn with_library(mut self, library: AnyLibrary) -> FrameSetup {
        self.library = Some(Rc::new(library));
        self
    }

    pub fn with_css(mut self, css: impl Into<Rc<str>>) -> FrameSetup {
        self.css = Some(css.into());
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
        let setup = self.setup.peek().clone();
        let (out, changed) = {
            let mut session = setup.session.borrow_mut();
            let out = f(&mut session);
            (out, session.settings_changed())
        };
        if changed {
            if let Some(save) = &setup.save_settings {
                save(setup.session.borrow().settings());
            }
        }
        let mut rev = self.rev;
        rev += 1;
        out
    }

    /// Poll the game. Draw again and save only when it sent something.
    pub(crate) fn tick(&self) {
        let setup = self.setup.peek().clone();
        let (before, after) = {
            let mut session = setup.session.borrow_mut();
            let before = session.log().back().map(|e| e.id);
            session.tick();
            (before, session.log().back().map(|e| e.id))
        };
        if before != after {
            self.update(|_| ());
        }
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
