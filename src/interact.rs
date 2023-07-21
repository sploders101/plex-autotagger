/// This function allows concurrent async threads to share the console.
pub async fn interact<T: Send + Sync + 'static>(func: impl FnOnce() -> T + Send + Sync + 'static) -> T {
	return tokio::task::spawn_blocking(|| func()).await.unwrap();
}
