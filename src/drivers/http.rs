use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::{thread::sleep, time::Duration};

use embedded_svc::http::Method;
use embedded_svc::io::Write;
use embedded_svc::ws::FrameType;
use esp_idf_svc::http::server::ws::EspHttpWsDetachedSender;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_sys::EspError;

use crate::app::AppState;
use crate::hal::gate::GateStatus;

#[derive(Clone)]
struct GateSenders(Arc<Mutex<VecDeque<EspHttpWsDetachedSender>>>);

impl GateSenders {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(
            VecDeque::<EspHttpWsDetachedSender>::new(),
        )))
    }

    fn add(&self, x: EspHttpWsDetachedSender) {
        log::info!("detached sender created");
        if let Ok(mut senders) = self.0.lock() {
            log::info!("detached sender added");
            senders.push_back(x);
            log::info!("detached senders count: {}", senders.len());
        }
    }

    fn send(&self, gate_status: GateStatus) {
        let data = match gate_status {
            GateStatus::Inactive => b"0",
            GateStatus::Active => b"1",
        };

        let frame_type = FrameType::Binary(false);

        if let Ok(mut senders) = self.0.lock() {
            // FIXME Clean up closed
            let senders: &mut VecDeque<EspHttpWsDetachedSender> = &mut senders;
            for sender in senders.into_iter().filter(|x| !x.is_closed()) {
                if sender.send(frame_type, data).is_err() {
                    log::error!("error sending gate status");
                }
            }
        }
    }
}

pub struct HttpServer {
    #[allow(dead_code)]
    esp_http_server: EspHttpServer,
    app_state: Arc<Mutex<AppState>>,
    gate_senders: GateSenders,
}

fn add_handlers(server: &mut EspHttpServer) -> anyhow::Result<GateSenders> {
    let gate_senders = GateSenders::new();
    let gate_senders_copy = gate_senders.clone();

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

    server.ws_handler("/gate", move |conn| -> Result<(), EspError> {
        if conn.is_new() {
            if let Ok(detached_sender) = conn.create_detached_sender() {
                gate_senders_copy.add(detached_sender);
            }
        }
        Ok(())
    })?;

    Ok(gate_senders)
}

impl HttpServer {
    pub fn new() -> anyhow::Result<Self> {
        let conf = Configuration::default();
        let mut esp_http_server = EspHttpServer::new(&conf)?;
        let app_state = Arc::new(Mutex::new(Default::default()));
        let gate_senders = add_handlers(&mut esp_http_server)?;
        Ok(HttpServer {
            esp_http_server,
            app_state,
            gate_senders,
        })
    }
}

impl crate::svc::HttpServer for HttpServer {
    fn set_app_state(&self, state: AppState) {
        self.app_state
            .try_lock()
            .as_mut()
            .map(|x| {
                if state.gate_status != x.gate_status {
                    log::info!("gate changed");
                    self.gate_senders.send(state.gate_status);
                }

                **x = state;
            })
            .ok();
    }
}

fn index_html() -> &'static [u8] {
    include_bytes!("../../assets/index.html")
}
