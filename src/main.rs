extern crate actix_web;
use actix_web::actix::{Actor, Addr, Context, Handler, MailboxError, Message};
use actix_web::{server, App, AsyncResponder, HttpRequest};
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

fn increment(req: &HttpRequest<AppState>) -> Box<Future<Item = String, Error = MailboxError>> {
    let ex: &Addr<ExampleStruct> = &req.state().example;

    println!("Sending request to actor");
    ex.send(Incrementer { by: 1 })
        .map(|int| {
            println!("Received response from actor");
            int.to_string()
        })
        .responder()
}

fn main() {
    server::new(|| {
        let addr = ExampleStruct { count: 0 }.start();
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

    macro_rules! expect_increment_to_ret {
        ($srv:ident, $expected_status:expr, $expected_body:expr) => {
            {
                let request = $srv.client(http::Method::GET, &"/").finish().unwrap();
                let response = $srv.execute(request.send()).unwrap();
                let bytes = $srv.execute(response.body()).unwrap();
                let body = str::from_utf8(&bytes).unwrap();

                assert_eq!(response.status(), $expected_status);
                assert_eq!(body, $expected_body);
            }
        };
    }

    #[test]
    fn test_increment_counter() {
        let mut srv = test_server();

        expect_increment_to_ret!(srv, 200, "1");
        expect_increment_to_ret!(srv, 200, "2");
        expect_increment_to_ret!(srv, 200, "3");
    }
}
