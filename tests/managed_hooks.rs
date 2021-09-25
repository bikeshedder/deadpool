#![cfg(feature = "managed")]

use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use deadpool::managed::{
    hooks::{HookCallback, HookError, HookErrorCause},
    Manager, Metrics, Pool, PoolError, RecycleResult,
};

struct Computer {
    next_id: AtomicUsize,
}

impl Computer {
    pub fn new(start: usize) -> Self {
        Self {
            next_id: AtomicUsize::new(start),
        }
    }
}

#[async_trait]
impl Manager for Computer {
    type Type = usize;
    type Error = ();

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    async fn recycle(&self, _: &mut Self::Type) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

/// This hook fails every other object creation
/// with a [`HookError::Continue`]
struct CreateErrContinueHook {}

#[async_trait]
impl HookCallback<usize, ()> for CreateErrContinueHook {
    async fn call(&self, obj: &mut usize, _: &Metrics) -> Result<(), HookError<()>> {
        if *obj % 2 != 0 {
            Err(HookError::Continue(None))
        } else {
            Ok(())
        }
    }
}

/// This hook fails every other object creation
/// with a [`HookError::Abort`]
struct CreateErrAbortHook {}

#[async_trait]
impl HookCallback<usize, ()> for CreateErrAbortHook {
    async fn call(&self, obj: &mut usize, _: &Metrics) -> Result<(), HookError<()>> {
        if *obj % 2 != 0 {
            Err(HookError::Abort(HookErrorCause::StaticMessage("fail!")))
        } else {
            Ok(())
        }
    }
}

struct IncrementHook {}

#[async_trait]
impl HookCallback<usize, ()> for IncrementHook {
    async fn call(&self, obj: &mut usize, _: &Metrics) -> Result<(), HookError<()>> {
        *obj += 1;
        Ok(())
    }
}

/// This hook fails after the object was recycled once
struct RecycleErrContinueHook {}

#[async_trait]
impl HookCallback<usize, ()> for RecycleErrContinueHook {
    async fn call(&self, _: &mut usize, metrics: &Metrics) -> Result<(), HookError<()>> {
        if metrics.recycle_count > 0 {
            return Err(HookError::Continue(None));
        } else {
            Ok(())
        }
    }
}

/// This hook fails after the second recycle operation
/// with a [`HookError::Error`]
struct RecycleErrAbortHook {}

#[async_trait]
impl HookCallback<usize, ()> for RecycleErrAbortHook {
    async fn call(&self, _: &mut usize, metrics: &Metrics) -> Result<(), HookError<()>> {
        if metrics.recycle_count > 0 {
            return Err(HookError::Abort(HookErrorCause::StaticMessage("fail!")));
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn post_create_ok() {
    let manager = Computer::new(42);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .post_create(IncrementHook {})
        .build()
        .unwrap();
    assert!(*pool.get().await.unwrap() == 43);
}

#[tokio::test]
async fn post_create_err_continue() {
    let manager = Computer::new(0);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(3)
        .post_create(CreateErrContinueHook {})
        .build()
        .unwrap();
    let obj1 = pool.get().await.unwrap();
    assert_eq!(*obj1, 0);
    let obj2 = pool.get().await.unwrap();
    assert_eq!(*obj2, 2);
    let obj3 = pool.get().await.unwrap();
    assert_eq!(*obj3, 4);
}

#[tokio::test]
async fn post_create_err_abort() {
    let manager = Computer::new(0);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(3)
        .post_create(CreateErrAbortHook {})
        .build()
        .unwrap();
    let obj1 = pool.get().await.unwrap();
    assert_eq!(*obj1, 0);
    assert!(pool.get().await.is_err());
    let obj2 = pool.get().await.unwrap();
    assert_eq!(*obj2, 2);
    assert!(pool.get().await.is_err());
    let obj2 = pool.get().await.unwrap();
    assert_eq!(*obj2, 4);
}

#[tokio::test]
async fn pre_recycle_ok() {
    let manager = Computer::new(42);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .pre_recycle(IncrementHook {})
        .build()
        .unwrap();
    assert!(*pool.get().await.unwrap() == 42);
    assert!(*pool.get().await.unwrap() == 43);
    assert!(*pool.get().await.unwrap() == 44);
    assert!(*pool.get().await.unwrap() == 45);
}

#[tokio::test]
async fn pre_recycle_err_continue() {
    let manager = Computer::new(0);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .pre_recycle(RecycleErrContinueHook {})
        .build()
        .unwrap();
    assert_eq!(*pool.get().await.unwrap(), 0);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 0);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 1);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 1);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 2);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 2);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
}

#[tokio::test]
async fn pre_recycle_err_abort() {
    let manager = Computer::new(0);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .pre_recycle(RecycleErrAbortHook {})
        .build()
        .unwrap();
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().size, 0);
    assert!(matches!(pool.get().await, Ok(x) if *x == 0));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(pool.get().await, Ok(x) if *x == 0));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(
        pool.get().await,
        Err(PoolError::PreRecycleHook(HookError::Abort(_)))
    ));
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().size, 0);
    assert!(matches!(pool.get().await, Ok(x) if *x == 1));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(pool.get().await, Ok(x) if *x == 1));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(
        pool.get().await,
        Err(PoolError::PreRecycleHook(HookError::Abort(_)))
    ));
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().size, 0);
    assert!(matches!(pool.get().await, Ok(x) if *x == 2));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(pool.get().await, Ok(x) if *x == 2));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(
        pool.get().await,
        Err(PoolError::PreRecycleHook(HookError::Abort(_)))
    ));
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().size, 0);
}

#[tokio::test]
async fn post_recycle_ok() {
    let manager = Computer::new(42);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .post_recycle(IncrementHook {})
        .build()
        .unwrap();
    assert!(*pool.get().await.unwrap() == 42);
    assert!(*pool.get().await.unwrap() == 43);
    assert!(*pool.get().await.unwrap() == 44);
    assert!(*pool.get().await.unwrap() == 45);
}

#[tokio::test]
async fn post_recycle_err_continue() {
    let manager = Computer::new(0);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .post_recycle(RecycleErrContinueHook {})
        .build()
        .unwrap();
    assert_eq!(*pool.get().await.unwrap(), 0);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 0);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 1);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 1);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 2);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(*pool.get().await.unwrap(), 2);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
}

#[tokio::test]
async fn post_recycle_err_abort() {
    let manager = Computer::new(0);
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .post_recycle(RecycleErrAbortHook {})
        .build()
        .unwrap();
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().size, 0);
    assert!(matches!(pool.get().await, Ok(x) if *x == 0));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(pool.get().await, Ok(x) if *x == 0));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(
        pool.get().await,
        Err(PoolError::PostRecycleHook(HookError::Abort(_)))
    ));
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().size, 0);
    assert!(matches!(pool.get().await, Ok(x) if *x == 1));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(pool.get().await, Ok(x) if *x == 1));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(
        pool.get().await,
        Err(PoolError::PostRecycleHook(HookError::Abort(_)))
    ));
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().size, 0);
    assert!(matches!(pool.get().await, Ok(x) if *x == 2));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(pool.get().await, Ok(x) if *x == 2));
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().size, 1);
    assert!(matches!(
        pool.get().await,
        Err(PoolError::PostRecycleHook(HookError::Abort(_)))
    ));
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().size, 0);
}
