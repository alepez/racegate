#![allow(non_snake_case)]



mod app;

use app::*;

fn main() {
    dioxus_web::launch(App);
}
