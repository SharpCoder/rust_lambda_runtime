#![allow(dead_code)]

use async_trait::async_trait;
use std::collections::BTreeMap;
use std::{env};
use http_req::request;

pub type LambdaStatus = Result<String, String>;

const API_VERSION: &str = "2018-06-01";
const REQUEST_ID_HEADER: &str = "Lambda-Runtime-Aws-Request-Id";
const TRACE_ID_HEADER: &str = "Lambda-Runtime-Trace-Id";
const FUNCTION_ARN_HEADER: &str = "Lambda-Runtime-Invoked-Function-Arn";

fn get_env() -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for (key, value) in env::vars() {
        result.insert(key, value);
    }
    return result;
}

#[async_trait]
pub trait LambdaHandler<T> {
    async fn handle(&self, payload: T) -> LambdaStatus;
}

/// This is the lambda runtime helper. It has the ability to
/// execute specific functions within the lambda ecosystem.
pub mod runtime {
    use super::*;

    struct InvocationRequest<T> {
        pub payload: T,
        request_id: String,
        trace_id: String,
        function_arn: String,
    }

    fn build_uri(path: &str) -> String {
        let env = get_env();
        let api = if env.contains_key("AWS_LAMBDA_RUNTIME_API") { env.get("AWS_LAMBDA_RUNTIME_API").unwrap() } else { "127.0.0.1" };
        return format!("http://{}/{}/runtime{}", api, API_VERSION, path).to_string();
    }

    fn headers_to_map(headers: &http_req::response::Headers) -> BTreeMap<String, String> {
        return headers.iter()
            .fold(BTreeMap::new(), |mut acc, header| {
                let key = header.0.to_string();
                let val = header.1.to_string();
                acc.insert(key, val);
                return acc;
            });
    }

    async fn next_invocation<R : serde::de::DeserializeOwned>() -> Option<InvocationRequest<R>> {
        let mut stream: Vec<u8> = Vec::new();    
        let res = request::get(build_uri("/invocation/next"), &mut stream);
        
        if res.is_ok() {
            println!("payload {}", String::from_utf8_lossy(&stream));
            let content = String::from_utf8_lossy(&stream).to_string();
            let payload: R = serde_json::from_str(content.as_str().clone()).unwrap();
            let ok_val = res.ok();
            let headers = headers_to_map(ok_val.as_ref().unwrap().headers());

            if headers.contains_key(REQUEST_ID_HEADER) && headers.contains_key(TRACE_ID_HEADER) && headers.contains_key(FUNCTION_ARN_HEADER) {
                return Some(InvocationRequest {
                    request_id: headers.get(REQUEST_ID_HEADER).unwrap().to_string(),
                    trace_id: headers.get(TRACE_ID_HEADER).unwrap().to_string(),
                    function_arn: headers.get(FUNCTION_ARN_HEADER).unwrap().to_string(),
                    payload: payload,
                });
            }
        } 

        return None;
    }

    async fn send_response<R>(request: &InvocationRequest<R>, result: LambdaStatus) {
        let payload: (String, String) = match result {
            Ok(code) => (
                build_uri(format!("/invocation/{}/response", request.request_id.as_str()).as_str()),
                code.to_string()
            ),
            Err(err) => (
                build_uri(format!("/invocation/{}/error", request.request_id.as_str()).as_str()), 
                err.to_string()
            ),
        };

        request::post(payload.0.as_str(), payload.1.as_bytes(), &mut Vec::new()).unwrap();
    }

    pub async fn process<T, R: serde::de::DeserializeOwned>(app: T)
    where T : LambdaHandler<R>,
          R : Clone 
    {   
        println!("fetching initial invocation request");
        let mut current_invocation: Option<InvocationRequest<R>> = runtime::next_invocation().await;
        while current_invocation.is_some()  {
            let context = current_invocation.unwrap();
            println!("processing invocation request");
            let outcome = app.handle(context.payload.clone()).await;
            runtime::send_response(&context, outcome).await;
            println!("checking for more invocation requests");
            current_invocation = runtime::next_invocation().await;
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_lambda() {

    }
}