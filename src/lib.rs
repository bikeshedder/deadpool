use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[cfg(feature = "postgres")]
pub mod postgres;

#[async_trait]
pub trait Manager<T, E> {
    async fn create(&self) -> Result<T, E>;
    async fn recycle(&self, obj: T) -> Result<T, E>;
}

pub struct Object<T, E, M> where M: Manager<T, E> + Sized {
    obj: Option<T>,
    pool: Weak<PoolInner<T, E, M>>
}

impl<T, E, M: Manager<T, E>> Object<T, E, M> {
    fn new(pool: &Pool<T, E, M>, obj: T) -> Object<T, E, M> {
        Object {
            obj: Some(obj),
            pool: Arc::downgrade(&pool.inner),
        }
    }
}

impl<T, E, M: Manager<T, E>> Drop for Object<T, E, M> {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.upgrade() {
            pool.return_obj(self.obj.take().unwrap());
        }
    }
}

impl<T, E, M: Manager<T, E>> Deref for Object<T, E, M> {
    type Target = T;
    fn deref(&self) -> &T {
        self.obj.as_ref().unwrap()
    }
}

impl<T, E, M: Manager<T, E>> DerefMut for Object<T, E, M> {
    fn deref_mut(&mut self) -> &mut T {
        self.obj.as_mut().unwrap()
    }
}

#[derive(Default)]
pub struct PoolSize {
    current: AtomicUsize,
    available: AtomicIsize,
}

pub struct PoolInner<T, E, M: Manager<T, E> + Sized> {
    manager: M,
    max_size: usize,
    obj_sender: Sender<T>,
    obj_receiver: Mutex<Receiver<T>>,
    size: PoolSize,
    //
    _error: PhantomData<E>
}

impl<T, E, M: Manager<T, E>> PoolInner<T, E, M> {
    fn return_obj(&self, obj: T) {
        self.size.available.fetch_add(1, Ordering::SeqCst);
        self.obj_sender.clone().try_send(obj).map_err(|_| ()).unwrap();
    }
}

#[derive(Clone)]
pub struct Pool<T, E, M: Manager<T, E> + Sized> {
    inner: Arc<PoolInner<T, E, M>>
}

impl<T, E, M: Manager<T, E>> Pool<T, E, M> {
    pub fn new(manager: M, max_size: usize) -> Pool<T, E, M> {
        let (obj_sender, obj_receiver) = channel::<T>(max_size);
        Pool {
            inner: Arc::new(PoolInner {
                max_size: max_size,
                manager: manager,
                obj_sender: obj_sender,
                obj_receiver: Mutex::new(obj_receiver),
                size: PoolSize::default(),
                _error: PhantomData
            })
        }
    }

    pub async fn get(&self) -> Result<Object<T, E, M>, E> {
        let available = self.inner.size.available.fetch_sub(1, Ordering::SeqCst);
        if available <= 0 && self.inner.size.current.load(Ordering::SeqCst) < self.inner.max_size {
            let current = self.inner.size.current.fetch_add(1, Ordering::SeqCst);
            if current < self.inner.max_size {
                self.inner.size.available.fetch_add(1, Ordering::SeqCst);
                let obj = self.inner.manager.create().await?;
                return Ok(Object::new(&self, obj))
            }
        }
        let obj = self.inner.obj_receiver.lock().await.recv().await.unwrap();
        let obj = self.inner.manager.recycle(obj).await?;
        return Ok(Object::new(&self, obj));
    }
}
