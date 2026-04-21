mod app;
mod components;
mod pages;
mod storage;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
