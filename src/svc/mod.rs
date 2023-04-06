use crate::app::AppState;

pub trait HttpServer {
    fn set_app_state(&self, status: AppState);
}
