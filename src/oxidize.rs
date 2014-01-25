extern mod http;
extern mod extra;

use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{Star, AbsoluteUri, AbsolutePath, Authority};
use http::headers;
use http::status;
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use extra::time;
use std::io::File;
use std::str;

use router::Router;
use renderer::Renderer;

mod router;
mod renderer;

#[deriving(Clone)]
struct OxidizeServer;

struct OxidizeRouter;

struct OxidizeRenderer;

impl Renderer for OxidizeRenderer {
  fn render(&self, file_name: &str) -> ~str {
    let contents = File::open(&Path::new("views/"+file_name+".html")).read_to_end();
    return str::from_utf8(contents).to_owned();
  }
}

impl Router for OxidizeRouter {
  // should probably return a result object
  // containing the body and the status
  fn route(&self, path: &str, response: &mut ResponseWriter) -> ~str {
    if(path == "/"){
      return OxidizeRenderer.render("index");
    }
    else {
      response.status = status::NotFound;
      return OxidizeRenderer.render("404");
    }
  }
}

impl Server for OxidizeServer {
  fn get_config(&self) -> Config {
    Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 } }
  }

  fn handle_request(&self, req: &Request, res: &mut ResponseWriter) {
    res.headers.date = Some(time::now_utc());
    res.headers.server = Some(~"Oxidize/0.0.0 (Ubuntu)");

    let path = match req.request_uri{
      AbsolutePath(ref i) => i.to_str(),
      AbsoluteUri(ref i) => i.to_str(),
      Authority(ref i) => i.to_str(),
      Star => ~"error" // ?
    };

    let response_body = OxidizeRouter.route(path,res);

    res.headers.content_type = Some(headers::content_type::MediaType {
      type_: ~"text",
      subtype: ~"html",
      parameters: ~[]
    });

    res.headers.content_length = Some(response_body.len());

    res.write(response_body.as_bytes());
  }
}

fn main(){
  OxidizeServer.serve_forever();
}