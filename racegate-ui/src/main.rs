#![allow(non_snake_case)]

use dioxus::prelude::*;

mod app;

use app::*;

fn main() {
    dioxus_web::launch(App);
}
