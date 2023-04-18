use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_websocket_hooks::use_ws_context_provider_json;
use fermi::{use_init_atom_root, use_read, use_set, Atom};
use gloo_console::log;
use racegate::app::SystemState;

pub static SYSTEM_STATE: Atom<Option<SystemState>> = |_| None;

#[allow(non_snake_case)]
pub fn App(cx: Scope) -> Element {
    use_init_atom_root(&cx);
    let set_system_state = Rc::clone(use_set(&cx, SYSTEM_STATE));

    use_ws_context_provider_json::<SystemState>(&cx, "ws://192.168.1.207:80/state", move |msg| {
        set_system_state(Some(msg));
    });

    cx.render(rsx!(
        h1 { "racegate" },
        Main { },
    ))
}

#[allow(non_snake_case)]
fn Main(cx: Scope) -> Element {
    if let Some(system_state) = use_read(&cx, SYSTEM_STATE) {
        Dashboard(cx, system_state)
    } else {
        cx.render(rsx!(div { "loading..." }))
    }
}

#[allow(non_snake_case)]
fn Dashboard<'a>(cx: Scope<'a>, system_state: &'a SystemState) -> Element<'a> {
    let duration = system_state.race().duration();
    let duration = format!("{:?}", duration);
    cx.render(rsx!(
        h2 { "dashboard" },
        div { duration }
    ))
}
