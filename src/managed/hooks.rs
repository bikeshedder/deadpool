pub enum CustomizerError<E> {
    Backend(E),
    Message(String),
}

#[async_trait::async_trait]
pub trait PostCreate: Sync + Send {
    type Type;
    type Error;
    async fn post_create(&self, obj: Self::Type) -> Result<Self::Type, CustomizerError<Self::Error>>;
}
