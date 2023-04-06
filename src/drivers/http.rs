use std::sync::{Arc, Mutex};
use std::{thread::sleep, time::Duration};

use embedded_svc::http::Method;
use embedded_svc::io::Write;
use embedded_svc::ws::FrameType;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_sys::EspError;

use crate::app::AppState;
use crate::hal::gate::GateStatus;

pub struct HttpServer {
    #[allow(dead_code)]
    esp_http_server: EspHttpServer,
    app_state: Arc<Mutex<AppState>>,
}

fn add_handlers(server: &mut EspHttpServer, app_state: Arc<Mutex<AppState>>) -> anyhow::Result<()> {
    server.fn_handler("/", Method::Get, |request| {
        let html = index_html();
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
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
        let frame_type = FrameType::Binary(false);
        let mut sent_gate_status: Option<GateStatus> = None;

        loop {
            let gate_status = app_state.try_lock().map(|x| x.gate_status).ok();

            if gate_status != sent_gate_status {
                let data = match gate_status {
                    Some(GateStatus::Inactive) => b"0",
                    Some(GateStatus::Active) => b"1",
                    _ => b"?",
                };

                log::info!("{:?}", gate_status);

                if conn.send(frame_type, data).is_err() {
                    log::error!("error sending gate status");
                    break;
                }

                sent_gate_status = gate_status;
            }

            sleep(Duration::from_millis(10));
        }

        Ok(())
    })?;

    Ok(())
}

impl HttpServer {
    pub fn new() -> anyhow::Result<Self> {
        let conf = Configuration::default();
        let mut esp_http_server = EspHttpServer::new(&conf)?;
        let app_state = Arc::new(Mutex::new(Default::default()));
        add_handlers(&mut esp_http_server, app_state.clone())?;
        Ok(HttpServer {
            esp_http_server,
            app_state,
        })
    }
}

impl crate::svc::HttpServer for HttpServer {
    fn set_app_state(&self, state: AppState) {
        self.app_state
            .try_lock()
            .as_mut()
            .map(|x| {
                **x = state;
            })
            .ok();
    }
}

fn index_html() -> &'static str {
    r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>racegate</title>
    </head>
    <body>
        This is racegate
    </body>
</html>
"#
}
