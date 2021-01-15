use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Result, Server, StatusCode,
};
use log::{error, info};
use qstring::QString;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;

use std::{
    env::{set_var, var},
    time::Instant,
};

const PG: &'static str = include_str!("../data/pg.txt");
const NON_PG: &'static str = include_str!("../data/non-pg.txt");

lazy_static::lazy_static! {
    static ref PG_PROMPTS: Vec<&'static str> = {
        PG.lines().collect()
    };
    static ref NON_PG_PROMPTS: Vec<&'static str> = {
        let mut lines: Vec<_> = NON_PG.lines().collect();
        lines.extend_from_slice(&PG_PROMPTS);
        lines
    };
}

fn get_random_prompt(seed: Option<u64>, step: Option<u64>, allow_non_pg: bool) -> String {
    let mut rng = {
        if let Some(seed) = seed {
            ChaCha8Rng::seed_from_u64(seed)
        } else {
            ChaCha8Rng::from_entropy()
        }
    };
    if let Some(step) = step {
        rng.set_stream(step);
    }

    if allow_non_pg {
        NON_PG_PROMPTS
            .choose(&mut rng)
            .unwrap()
            .to_owned()
            .to_owned()
    } else {
        PG_PROMPTS.choose(&mut rng).unwrap().to_owned().to_owned()
    }
}

async fn request_handler(req: Request<Body>) -> Result<Response<Body>> {
    let method = req.method();
    let path = req.uri().path();
    let start = Instant::now();

    let response = match (method, path) {
        (&Method::GET, "/prompt") => {
            let query = req.uri().query();
            let (seed, step, allow_non_pg) = {
                if let Some(query) = query {
                    let query = QString::from(query);
                    (
                        query.get("seed").and_then(|s| s.parse().ok()),
                        query.get("step").and_then(|s| s.parse().ok()),
                        query
                            .get("allow_non_pg")
                            .map_or(false, |v| v.parse().unwrap_or(false)),
                    )
                } else {
                    (None, None, false)
                }
            };
            let prompt = get_random_prompt(seed, step, allow_non_pg);
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(prompt))
                .unwrap()
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    };

    let finish = Instant::now();
    info!(
        "{} {} {} (took {:?})",
        method,
        path,
        response.status(),
        finish - start
    );

    Ok(response)
}

#[tokio::main]
async fn main() {
    if var("RUST_LOG").is_err() {
        set_var("RUST_LOG", "info");
    }
    env_logger::builder().format_timestamp_millis().init();

    let port: u16 = var("PORT")
        .unwrap_or_else(|_| String::from("4000"))
        .parse()
        .unwrap();

    let addr = ([0, 0, 0, 0], port).into();

    let make_service = make_service_fn(move |_| async move {
        Ok::<_, hyper::Error>(service_fn(move |req| request_handler(req)))
    });

    let server = Server::bind(&addr).serve(make_service);

    info!("Listening on port {}", port);

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}
