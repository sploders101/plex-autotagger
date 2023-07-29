use std::future::Future;

use lazy_static::lazy_static;
use tokio::sync::oneshot;

use crate::task_queue::TaskQueue;


lazy_static! {
	static ref QUEUE: TaskQueue = TaskQueue::new();
}

/// This function allows concurrent async threads to share the console.
pub async fn interact<T: Send + Sync + 'static>(
	func: impl FnOnce() -> T + Send + Sync + 'static,
) -> T {
	let (sender, receiver) = oneshot::channel();
	QUEUE.add_task(async move {
		let _ = sender.send(tokio::task::spawn_blocking(|| func()).await.unwrap());
	});

	return receiver.await.unwrap();
}

pub async fn interact_async<T: Send + Sync + 'static>(
	func: impl Future<Output = T> + Send + 'static,
) -> T {
	let (sender, receiver) = oneshot::channel();
	QUEUE.add_task(async move {
		let _ = sender.send(func.await);
	});

	return receiver.await.unwrap();
}
