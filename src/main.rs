extern crate actix_web;


use actix_web::actix::{Actor, Addr, Context, Handler, Message};
use actix_web::{server, App, HttpRequest, Responder};
use futures::future::Future;
pub struct AppState {
    pub example: Addr<ExampleStruct>,
}

pub struct ExampleStruct {
    pub count: u64,
}

impl Actor for ExampleStruct {
    type Context = Context<Self>;
}

pub struct Incrementer {
    pub by: u64,
}

impl Message for Incrementer {
    type Result = u64;
}

impl Handler<Incrementer> for ExampleStruct {
    type Result = u64;

    fn handle(&mut self, incrementer: Incrementer, _: &mut Self::Context) -> Self::Result {
        self.count += incrementer.by;

        self.count
    }
}

fn increment(req: &HttpRequest<AppState>) -> impl Responder {
    let ex: &Addr<ExampleStruct> = &req.state().example;

    println!("Sending request to actor");
    ex.send(Incrementer { by: 1 })
        .map(|int| {
            println!("Received response from actor");
            int.to_string()
        })
        .wait()
}

fn main() {
    let addr = ExampleStruct { count: 0 }.start();

    server::new(move || {
        App::with_state(AppState {
            example: addr.clone(),
        })
        .resource("/", |r| r.f(increment))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestServer;
    use actix_web::{http, HttpMessage};
    use std::str;

    fn test_server() -> TestServer {
        TestServer::build_with_state(|| {
            let addr = ExampleStruct { count: 0 }.start();
            AppState {
                example: addr.clone(),
            }
        })
        .start(|app| {
            app.resource("/", |r| r.f(increment));
        })
    }

    #[test]
    fn increment_counter() {
        let mut srv = test_server();

        let request = srv.client(http::Method::GET, &"/").finish().unwrap();

        let response = srv.execute(request.send()).unwrap();
        assert_eq!(response.status(), 200);

        let bytes = srv.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        assert_eq!(body, "1");
    }
}
