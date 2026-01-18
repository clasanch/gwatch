pub mod app;
pub mod diff_view;
pub mod handlers;
pub mod layout;
pub mod overlays;
pub mod render;
pub mod theme;

pub use app::App;
pub use handlers::handle_key_event;
pub use render::draw_ui;
