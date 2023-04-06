use embedded_svc::http::Method;
use embedded_svc::io::Write;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};

pub struct HttpServer {
    #[allow(dead_code)]
    esp_http_server: EspHttpServer,
}

fn add_handlers(server: &mut EspHttpServer) -> anyhow::Result<()> {
    server.fn_handler("/", Method::Get, |request| {
        let html = index_html();
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    Ok(())
}

impl HttpServer {
    pub fn new() -> anyhow::Result<Self> {
        let conf = Configuration::default();
        let mut esp_http_server = EspHttpServer::new(&conf)?;
        add_handlers(&mut esp_http_server)?;
        Ok(HttpServer { esp_http_server })
    }
}

impl crate::svc::HttpServer for HttpServer {}

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
