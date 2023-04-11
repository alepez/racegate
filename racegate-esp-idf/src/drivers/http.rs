use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::{thread::sleep, time::Duration};

use embedded_svc::http::Method;
use embedded_svc::io::Write;
use embedded_svc::ws::FrameType;
use esp_idf_svc::http::server::ws::EspHttpWsDetachedSender;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_sys::EspError;
use racegate::app::SystemState;

struct StateSender {
    ws: EspHttpWsDetachedSender,
}

#[derive(Clone)]
struct StateSenders(Arc<Mutex<VecDeque<StateSender>>>);

impl StateSenders {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(VecDeque::<StateSender>::new())))
    }

    fn add(&self, ws: EspHttpWsDetachedSender) {
        log::info!("detached sender created");
        if let Ok(mut senders) = self.0.lock() {
            log::info!("detached sender added");
            senders.push_back(StateSender { ws });
            log::info!("detached senders count: {}", senders.len());
        }
    }

    fn send(&self, system_state: &SystemState) {
        let frame_type = FrameType::Binary(false);

        let json = serde_json::to_vec(&system_state).unwrap();
        let data = json.as_slice();

        if let Ok(mut senders) = self.0.lock() {
            // FIXME Clean up closed
            let senders: &mut VecDeque<StateSender> = &mut senders;
            for sender in senders.iter_mut().filter(|x| !x.ws.is_closed()) {
                if sender.ws.send(frame_type, data).is_err() {
                    log::error!("error sending gate status");
                }
            }
        }
    }
}

pub struct HttpServer {
    #[allow(dead_code)]
    esp_http_server: EspHttpServer,
    app_state: Arc<Mutex<SystemState>>,
    state_senders: StateSenders,
}

fn add_handlers(server: &mut EspHttpServer) -> anyhow::Result<StateSenders> {
    let state_senders = StateSenders::new();
    let state_senders_copy = state_senders.clone();

    server.fn_handler("/", Method::Get, |request| {
        let mut response = request.into_ok_response()?;
        response.write_all(index_html())?;
        Ok(())
    })?;

    server.ws_handler("/test", |conn| -> Result<(), EspError> {
        let frame_type = FrameType::Binary(false);
        let data = "test".as_bytes();
        loop {
            if let Err(err) = conn.send(frame_type, data) {
                log::info!("error: {}", err);
                break;
            }
            sleep(Duration::from_millis(10));
        }
        Ok(())
    })?;

    server.ws_handler("/state", move |conn| -> Result<(), EspError> {
        if conn.is_new() {
            if let Ok(detached_sender) = conn.create_detached_sender() {
                state_senders_copy.add(detached_sender);
            }
        }
        Ok(())
    })?;

    Ok(state_senders)
}

impl HttpServer {
    pub fn new() -> anyhow::Result<Self> {
        let conf = Configuration::default();
        let mut esp_http_server = EspHttpServer::new(&conf)?;
        let app_state = Arc::new(Mutex::new(Default::default()));
        let state_senders = add_handlers(&mut esp_http_server)?;
        Ok(HttpServer {
            esp_http_server,
            app_state,
            state_senders,
        })
    }
}

impl racegate::svc::HttpServer for HttpServer {
    fn set_system_state(&self, state: &SystemState) {
        self.app_state
            .try_lock()
            .as_mut()
            .map(|x| {
                if x.ne(state) {
                    self.state_senders.send(state);
                }

                **x = state.clone();
            })
            .ok();
    }
}

fn index_html() -> &'static [u8] {
    include_bytes!("../../../racegate-ui/assets/index.html")
}
