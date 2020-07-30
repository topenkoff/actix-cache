use actix::prelude::*;
use actix_cache::dev::{
    backend::{GetMessages, MockBackend, MockMessage},
    Get,
};
use actix_cache::{Cache, CacheError, Cacheable};
use serde::{Deserialize, Serialize};

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
    fn cache_key(&self) -> Result<String, CacheError> {
        Ok(format!("Ping::{}", self.id))
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
    let cache = Cache::builder().build(backend.clone()).start();
    let upstream = UpstreamActor.start();
    let msg = Ping { id: 42 };
    cache.send(msg.into_cache(upstream)).await.unwrap().unwrap();
    let messages = backend.send(GetMessages).await.unwrap().0;
    assert_eq!(
        messages[..1],
        [MockMessage::Get(Get {
            key: "Ping::42".to_owned()
        }),]
    );
}