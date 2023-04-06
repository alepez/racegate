use esp_idf_svc::http::server::{Configuration, EspHttpServer};

pub struct HttpServer {
    #[allow(dead_code)]
    esp_http_server: EspHttpServer,
}

impl HttpServer {
    pub fn new() -> anyhow::Result<Self> {
        let conf = Configuration::default();
        let esp_http_server = EspHttpServer::new(&conf)?;
        Ok(HttpServer { esp_http_server })
    }
}

impl crate::svc::HttpServer for HttpServer {}
