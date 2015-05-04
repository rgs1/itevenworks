extern crate getopts;
extern crate nickel;
#[macro_use] extern crate nickel_macros;
extern crate regex;
extern crate rustc_serialize;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use getopts::Options;
use nickel::{
    Nickel, NickelError, Continue, Halt, Request, Response,
    StaticFilesHandler, MiddlewareResult, HttpRouter, Action, Middleware
};
use nickel::status::StatusCode::NotFound;
use regex::Regex;

struct PageHandler {
    path: String,
}

impl PageHandler {
    pub fn new (p: String) -> PageHandler {
        PageHandler {
            path: p,
        }
    }
}

impl Middleware for PageHandler {
    fn invoke<'a>(&self, _: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {
        res.send_file(Path::new(&self.path[..]))
    }
}

fn custom_404<'a>(err: &mut NickelError, _req: &mut Request) -> Action {
    if let Some(ref mut res) = err.stream {
        if res.status() == NotFound {
            let _ = res.write_all(b"<h1>Oops, not found!<h1>");
            return Halt(())
        }
    }

    Continue(())
}

fn usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] <routes-cfg>", program);
    print!("{}", opts.usage(&brief[..]));
}

fn run(assests_path: &str, port: u16, routes: HashMap<String, String>) {
    let mut server = Nickel::new();
    let mut router = Nickel::router();

    for (route, html_path) in routes.iter() {
        router.get(&route[..], PageHandler::new(html_path.clone()));
    }

    server.utilize(router);
    server.utilize(StaticFilesHandler::new(assests_path));
    server.utilize(middleware! { |request|
      println!("logging request: {:?}", request.origin.uri);
    });
    server.handle_error(custom_404 as fn(&mut NickelError, &mut Request) -> Action);
    let addr = format!("0.0.0.0:{}", port);
    server.listen(&addr[..]);
}

fn parse_routes(routes_cfg: String) -> HashMap<String, String> {
    let mut routes = HashMap::new();
    let path = Path::new(&*routes_cfg);
    let comment = Regex::new(r"^#").unwrap();

    let mut file = File::open(&path).ok().expect("couldn't open routes cfg");
    let mut content = String::new();
    let _ = file.read_to_string(&mut content).ok().expect("couldn't read routes");

    let lines = content.split("\n");
    for mut line in lines {
        line = line.trim_matches(' ');
        if line.is_empty() || comment.is_match(line) {
            continue;
        }

        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() != 2 {
            println!("skipping bad line: {}", line);
            continue;
        }

        println!("adding route {} -> {}", parts[0], parts[1]);
        routes.insert(parts[0].to_string(), parts[1].to_string());
    }

    routes
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut assets_path = "".to_string();
    let mut port: u16 = 7000;
    let mut opts = Options::new();

    opts.optopt("", "port", "port to listen", "PORT");
    opts.optopt("", "assets-path", "path for static assets", "ASSETS");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(_) => {
            usage(&program[..], opts);
            return;
        }
    };

    if matches.opt_present("h") {
        usage(&program[..], opts);
        return;
    }

    if matches.opt_present("port") {
        let port_s = &*matches.opt_str("port").unwrap();
        match port_s.parse::<u16>() {
            Ok(p) => port = p,
            Err(_) => {
                usage(&program[..], opts);
                return;
            }
        }
    }

    if matches.opt_present("assets-path") {
        assets_path = matches.opt_str("assets-path").unwrap();
    }

    let routes_cfg = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        usage(&program[..], opts);
        return;
    };

    let routes = parse_routes(routes_cfg);

    run(&*assets_path, port, routes);
}
