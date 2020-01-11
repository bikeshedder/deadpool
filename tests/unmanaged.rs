use deadpool::unmanaged::Pool;
use std::sync::Arc;

#[tokio::test]
async fn test_unmanaged_basic() {
    let pool = Pool::new(vec![(), (), ()]);

    let status = pool.status();
    assert_eq!(status.size, 3);
    assert_eq!(status.available, 3);

    let _val0 = pool.get().await;

    let status = pool.status();
    assert_eq!(status.size, 3);
    assert_eq!(status.available, 2);

    let _val1 = pool.get().await;

    let status = pool.status();
    assert_eq!(status.size, 3);
    assert_eq!(status.available, 1);

    let _val2 = pool.get().await;

    let status = pool.status();
    assert_eq!(status.size, 3);
    assert_eq!(status.available, 0);
}

#[tokio::test(threaded_scheduler)]
async fn test_unmanaged_concurrent() {
    let pool = Arc::new(Pool::new(vec![0usize, 0, 0]));

    // Spawn tasks
    let futures = (0..100)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move {
                *pool.get().await += 1;
            })
        })
        .collect::<Vec<_>>();

    // Await tasks to finish
    for future in futures {
        future.await.unwrap();
    }

    // Verify
    let status = pool.status();
    assert_eq!(status.size, 3);
    assert_eq!(status.available, 3);

    let values = [pool.get().await, pool.get().await, pool.get().await];

    assert_eq!(values.iter().map(|obj| **obj).sum::<usize>(), 100);
}
