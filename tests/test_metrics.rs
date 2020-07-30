#[cfg(feature = "metrics")]
mod tests {
    use actix::prelude::*;
    use actix_cache::metrics::CACHE_MISS_COUNTER;
    use actix_cache::{dev::backend::MockBackend, Cache, CacheError, Cacheable};

    pub struct Upstream;

    impl Actor for Upstream {
        type Context = Context<Self>;
    }

    #[derive(Message)]
    #[rtype(result = "Result<i32, ()>")]
    pub struct Ping(i32);

    impl Cacheable for Ping {
        fn cache_key(&self) -> Result<String, CacheError> {
            Ok(format!("Ping::{}", self.0))
        }
    }

    impl Handler<Ping> for Upstream {
        type Result = Result<i32, ()>;

        fn handle(&mut self, msg: Ping, _: &mut Self::Context) -> Self::Result {
            if msg.0 > 0 {
                Ok(msg.0)
            } else {
                Err(())
            }
        }
    }

    #[derive(Message)]
    #[rtype(result = "i32")]
    pub struct Pong;

    impl Cacheable for Pong {
        fn cache_key(&self) -> Result<String, CacheError> {
            Ok("Pong::".to_owned())
        }
    }

    impl Handler<Pong> for Upstream {
        type Result = i32;

        fn handle(&mut self, _msg: Pong, _: &mut Self::Context) -> Self::Result {
            42
        }
    }

    #[actix_rt::test]
    async fn test_miss_counter_metric() {
        let backend = MockBackend::new().start();
        let cache = Cache::builder().build(backend).start();
        let upstream = Upstream {}.start();
        let res = cache
            .send(Ping(8).into_cache(upstream.clone()))
            .await
            .unwrap();
        assert_eq!(res.unwrap(), Ok(8));
        let res = cache
            .send(Ping(-42).into_cache(upstream.clone()))
            .await
            .unwrap();
        assert_eq!(res.unwrap(), Err(()));
        let res = cache.send(Pong.into_cache(upstream.clone())).await.unwrap();
        assert_eq!(res.unwrap(), 42);

        assert_eq!(
            2,
            CACHE_MISS_COUNTER
                .with_label_values(&["test_metrics::tests::Ping", "test_metrics::tests::Upstream"])
                .get()
        );
        assert_eq!(
            1,
            CACHE_MISS_COUNTER
                .with_label_values(&["test_metrics::tests::Pong", "test_metrics::tests::Upstream"])
                .get()
        );
    }
}