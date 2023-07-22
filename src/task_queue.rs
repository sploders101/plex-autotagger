use std::{future::Future, pin::Pin};
use tokio::{
	sync::{
		mpsc::{unbounded_channel, UnboundedSender},
		oneshot,
	},
	task,
};

/// Creates a queue of tasks that should run in sequence, but shouldn't block other
/// operations.
///
/// This is most useful for parallelizing tasks that perform an I/O-bound operation,
/// then a CPU-bound operation in sequence. The I/O operation can be performed in a
/// loop, one after the other, and instead of executing the CPU-bound task directly,
/// it is pushed to this queue to be executed once the previous CPU-bound task is
/// finished.
///
/// One example of this is extracting subtitles, which is an I/O-bound operation,
/// then spawning vobsubocr to run OCR on the extracted subtitles while the next
/// subtitle track is extracted. Because vobsubocr is already highly parallelized,
/// the tasks that run it should not be, but it doesn't do a lot of I/O, so extracting
/// subtitles from the next file won't impact it much.
pub struct TaskQueue {
	sender: UnboundedSender<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
}
impl TaskQueue {
	pub fn new() -> Self {
		let (sender, mut receiver) = unbounded_channel();
		task::spawn(async move {
			while let Some(task) = receiver.recv().await {
				task.await;
			}
		});
		return Self { sender };
	}

	pub async fn add_task(&self, task: impl Future<Output = ()> + Send + 'static) {
		self.sender.send(Box::pin(task)).unwrap();
	}

	pub async fn wait_for_queued_tasks(&self) {
		let (sender, receiver) = oneshot::channel::<()>();
		self.add_task(async move {
			let _ = sender.send(());
		})
		.await;
		receiver.await.unwrap();
	}
}
