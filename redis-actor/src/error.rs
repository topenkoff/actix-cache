use redis::RedisError;

#[derive(Debug)]
pub enum Error {
    Redis(RedisError),
    Connection,
}

impl From<RedisError> for Error {
    fn from(error: RedisError) -> Self {
        Error::Redis(error)
    }
}
