use actix::prelude::*;
use actix_cache::dev::{
    backend::{GetMessages, MockBackend, MockMessage},
    Get,
};
use actix_cache::{CacheError, Cacheable};
use serde::{Deserialize, Serialize};
use actix_cache::CacheActor;

struct UpstreamActor;

impl Actor for UpstreamActor {
    type Context = Context<Self>;
}

#[derive(MessageResponse, Deserialize, Serialize, Debug)]
struct Pong {
    id: i32,
}

#[derive(Message, Serialize)]
#[rtype(result = "Pong")]
struct Ping {
    pub id: i32,
}

impl Cacheable for Ping {
    fn cache_message_key(&self) -> Result<String, CacheError> {
        Ok(format!("{}::{}", self.cache_key_prefix(), self.id))
    }
    fn cache_key_prefix(&self) -> String {
        "Ping".to_owned()
    }
}

impl Handler<Ping> for UpstreamActor {
    type Result = <Ping as Message>::Result;

    fn handle(&mut self, msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        Pong { id: msg.id }
    }
}

#[actix_rt::test]
async fn test_mock_backend() {
    let backend = MockBackend::new().start();
    let cache = CacheActor::builder().build(backend.clone()).start();
    let upstream = UpstreamActor.start();
    let msg = Ping { id: 42 };
    cache
        .send(msg.into_cache(&upstream))
        .await
        .unwrap()
        .unwrap();
    let messages = backend.send(GetMessages).await.unwrap().0;
    assert_eq!(
        messages[..1],
        [MockMessage::Get(Get {
            key: "UpstreamActor::Ping::42".to_owned()
        }),]
    );
}
