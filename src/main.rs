#![feature(core, collections, env, old_io, old_path)]

extern crate getopts;
extern crate http;
extern crate nickel;
extern crate regex;
extern crate "rustc-serialize" as rustc_serialize;

use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::old_io::File;
use std::old_io::net::ip::Ipv4Addr;

use getopts::Options;
use http::status::NotFound;
use nickel::{
    Nickel, NickelError, ErrorWithStatusCode, Continue, Halt, Request, Response,
    StaticFilesHandler, MiddlewareResult, HttpRouter, Action, Middleware
};
use nickel::mimes::MediaType;

struct PageHandler {
    path: Path,
}

impl Middleware for PageHandler {
    fn invoke(&self, _req: &mut Request, res: &mut Response) -> Result<Action, NickelError> {
        match File::open(&self.path) {
            Ok(mut file) => {
                match file.read_to_string() {
                    Ok(content) => {
                        res.content_type(MediaType::Html);
                        res.send(content)
                    },
                    Err(why) => panic!("couldn't read : {}", why.desc),
                }
            },
            Err(_) => res.send("Something went wrong"),
        }
        Ok(Halt)
    }
}

fn logger(request: &Request, _response: &mut Response) -> MiddlewareResult {
    println!("logging request: {}", request.origin.request_uri);
    Ok(Continue)
}

fn custom_404(err: &NickelError, _req: &Request, response: &mut Response) -> MiddlewareResult {
    match err.kind {
        ErrorWithStatusCode(NotFound) => {
            response.content_type(MediaType::Html)
                    .status_code(NotFound)
                    .send("<h1>Oops, not found!<h1>");
            Ok(Halt)
        },
        _ => Ok(Continue)
    }
}

fn usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] <routes-cfg>", program);
    print!("{}", opts.usage(brief.as_slice()));
}

fn run(assests_path: &str, port: u16, routes: HashMap<String, String>) {
    let mut server = Nickel::new();
    let mut router = Nickel::router();

    for (route, html_path) in routes.iter() {
        let handler = PageHandler { path: Path::new(&html_path[]) };
        router.get(&route[], handler);
    }

    server.utilize(router);
    server.utilize(StaticFilesHandler::new(assests_path));
    server.utilize(logger as fn(&Request, &mut Response) -> MiddlewareResult);
    server.handle_error(custom_404 as fn(&NickelError, &Request, &mut Response) -> MiddlewareResult);
    server.listen(Ipv4Addr(0, 0, 0, 0), port);
}

fn parse_routes(routes_cfg: String) -> HashMap<String, String> {
    let mut routes = HashMap::new();
    let path = Path::new(&routes_cfg[]);
    let comment = Regex::new(r"^#").unwrap();

    match File::open(&path) {
        Ok(mut file) => {
            match file.read_to_string() {
                Ok(content) => {
                    let lines = content.split_str("\n");
                    for mut line in lines {
                        line = line.trim_matches(' ');
                        if line.is_empty() || comment.is_match(line) {
                            continue;
                        }

                        let mut parts = line.split_str(" ");
                        let mut added = false;
                        match parts.next() {
                            Some(route) => {
                                match parts.next() {
                                    Some(html_file) => {
                                        println!("adding route {} -> {}", route, html_file);
                                        routes.insert(route.to_string(), html_file.to_string());
                                        added = true;
                                    },
                                    _ => {},
                                }
                            },
                            _ => {},
                        }

                        if !added {
                            println!("skipping bad line: {}", line);
                        }
                    }
                },
                Err(why) => panic!("couldn't read: {}", why.desc),
            }
        },
        Err(e) => panic!("couldn't open routes cfg: {}", e),
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

    let matches = match opts.parse(args.tail()) {
        Ok(m) => { m }
        Err(_) => {
            usage(program.as_slice(), opts);
            return;
        }
    };

    if matches.opt_present("h") {
        usage(program.as_slice(), opts);
        return;
    }

    if matches.opt_present("port") {
        let port_s = &matches.opt_str("port").unwrap()[];
        match port_s.parse::<u16>() {
            Ok(p) => port = p,
            Err(_) => {
                usage(program.as_slice(), opts);
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
        usage(program.as_slice(), opts);
        return;
    };

    let routes = parse_routes(routes_cfg);

    run(&assets_path[], port, routes);
}
