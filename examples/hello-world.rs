extern crate tokio;

use std::{env};
use async_trait::async_trait;
use serde::{Deserialize};
use rust_lambda_runtime::{ runtime, LambdaHandler, LambdaStatus};

#[derive(Deserialize, Clone)]
struct TestEvent {
    pub key1: String,
    pub key2: String,
    pub key3: String,
}

type InvocationType = TestEvent;

struct App { }

#[async_trait]
impl LambdaHandler<InvocationType> for App {
    async fn handle(&self, context: InvocationType) -> LambdaStatus {
        let args: Vec<String> = env::args().collect();
        args.iter()
            .for_each(|arg| {
                println!("{}", arg);
            });


        println!("hello, lambda world!");
        println!("Your event has the following values: {}, {}, {}", context.key1, context.key2, context.key3);
        return Ok("okay".to_string());
    }
}
#[tokio::main]
async fn main() {
    let app = App { };
    runtime::process::<App, InvocationType>(app).await;
    std::process::exit(0);    
}