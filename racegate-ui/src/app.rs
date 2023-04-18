use std::{rc::Rc, time::Duration};

use dioxus::prelude::*;
use dioxus_websocket_hooks::use_ws_context_provider_json;
use fermi::{use_init_atom_root, use_read, use_set, Atom};
use gloo_console::log;
use racegate::app::{gates::Gate, SystemState};
use racegate::CoordinatedInstant;

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

    let start_gate = system_state.gates().start_gate().clone();
    let finish_gate = system_state.gates().finish_gate().clone();

    let duration = DurationComponent(cx, duration);
    let start_gate = GateComponent(cx, "Start".to_owned(), start_gate, system_state.time());
    let finish_gate = GateComponent(cx, "Finish".to_owned(), finish_gate, system_state.time());

    cx.render(rsx!(
        h2 { "dashboard" },
        duration,
        start_gate,
        finish_gate
    ))
}

#[allow(non_snake_case)]
fn DurationComponent(cx: Scope, duration: Option<Duration>) -> Element {
    let duration_text = duration
        .map(format_duration)
        .unwrap_or_else(|| "-".to_owned());

    cx.render(rsx!(span { duration_text }))
}

fn format_duration(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64())
}

#[allow(non_snake_case)]
fn GateComponent(cx: Scope, name: String, gate: Gate, time: CoordinatedInstant) -> Element {
    let alive = gate.is_alive(time);
    let active = gate.is_active();

    cx.render(rsx!(
        div {
            span { 
                display: "inline-block",
                width: "4em",
                "{name}" 
            }
            span {
                display: "inline-block",
                margin_left: "1em",
                "{alive}",
            }
            span {
                display: "inline-block",
                margin_left: "1em",
                "{active}",
            }
        }
    ))
}
