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

fn index(req: &HttpRequest<AppState>) -> impl Responder {
    let ex: &Addr<ExampleStruct> = &req.state().example;

    ex.send(Incrementer { by: 1 })
        .map(|int| int.to_string())
        .wait()
}

fn main() {
    let addr = ExampleStruct { count: 0 }.start();

    server::new(move || {
        App::with_state(AppState {
            example: addr.clone(),
        })
        .resource("/", |r| r.f(index))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run();
}
