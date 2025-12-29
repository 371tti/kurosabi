use core::future::Future;
use core::time::Duration;
use std::io;
use std::net::SocketAddr;
#[cfg(feature = "compio-runtime")]
use std::{any::Any, pin::Pin};

use futures::io::{AsyncRead, AsyncWrite};

pub trait KurosabiRuntime: Clone + Send + Sync + 'static {
    type JoinError: core::fmt::Debug + Send + 'static;

    type JoinHandle<T>: Send + 'static
    where
        T: Send + 'static;

    type JoinFuture<'a, T>: Future<Output = Result<T, Self::JoinError>> + Send + 'a
    where
        Self: 'a,
        T: Send + 'static;

    fn spawn<F, T>(&self, fut: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;

    fn join<'a, T>(&'a self, handle: &'a mut Self::JoinHandle<T>) -> Self::JoinFuture<'a, T>
    where
        T: Send + 'static;

    type SleepFuture<'a>: Future<Output = ()> + Send + 'a
    where
        Self: 'a;

    fn sleep<'a>(&'a self, dur: Duration) -> Self::SleepFuture<'a>;

    type TcpStream: AsyncRead + AsyncWrite + Unpin + Send + 'static;
    type TcpListener: Send + 'static;

    type TcpConnectFuture<'a>: Future<Output = io::Result<Self::TcpStream>> + Send + 'a
    where
        Self: 'a;

    type TcpBindFuture<'a>: Future<Output = io::Result<Self::TcpListener>> + Send + 'a
    where
        Self: 'a;

    type TcpAcceptFuture<'a>: Future<Output = io::Result<(Self::TcpStream, SocketAddr)>> + Send + 'a
    where
        Self: 'a;

    fn tcp_connect<'a>(&'a self, addr: SocketAddr) -> Self::TcpConnectFuture<'a>;
    fn tcp_bind<'a>(&'a self, addr: SocketAddr) -> Self::TcpBindFuture<'a>;
    fn tcp_accept<'a>(&'a self, listener: &'a Self::TcpListener) -> Self::TcpAcceptFuture<'a>;
}

#[derive(Clone)]
pub struct DefaultRuntime;

#[cfg(feature = "tokio-runtime")]
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

#[cfg(feature = "tokio-runtime")]
impl KurosabiRuntime for DefaultRuntime {
    type JoinHandle<T> = tokio::task::JoinHandle<T> where T: Send + 'static;
    type JoinError = tokio::task::JoinError;
    type Sleep = tokio::time::Sleep;

    fn spawn<F, T>(&self, fut: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        tokio::spawn(fut)
    }

    fn sleep(&self, dur: Duration) -> Self::Sleep {
        tokio::time::sleep(dur)
    }

    type TcpStream = Compat<tokio::net::TcpStream>; // ここがポイント
    type TcpListener = tokio::net::TcpListener;
        
    fn tcp_connect(
        &self,
        addr: SocketAddr,
    ) -> impl Future<Output = io::Result<Self::TcpStream>> + Send + 'static {
        async move {
            let s = tokio::net::TcpStream::connect(addr).await?;
            Ok(s.compat()) // tokio -> futures 互換
        }
    }

    fn tcp_bind(
        &self,
        addr: SocketAddr,
    ) -> impl Future<Output = io::Result<Self::TcpListener>> + Send + 'static {
        async move { tokio::net::TcpListener::bind(addr).await }
    }

    fn tcp_accept<'a>(
        &'a self,
        listener: &'a Self::TcpListener,
    ) -> impl Future<Output = io::Result<(Self::TcpStream, SocketAddr)>> + Send + 'a {
        async move {
            let (s, peer) = listener.accept().await?;
            Ok((s.compat(), peer))
        }
    }
}

#[cfg(feature = "compio-runtime")]

impl KurosabiRuntime for DefaultRuntime {

    type JoinError = Box<dyn Any + Send>;

    type JoinHandle<T> = compio::runtime::JoinHandle<T>
    where
        T: Send + 'static;

    type JoinFuture<'a, T> =
        impl core::future::Future<Output = Result<T, Self::JoinError>> + Send + 'a
    where
        Self: 'a,
        T: Send + 'static;

    fn spawn<F, T>(&self, fut: F) -> Self::JoinHandle<T>
    where
        F: core::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        compio::runtime::spawn(fut)
    }

    fn join<'a, T>(&'a self, handle: &'a mut Self::JoinHandle<T>) -> Self::JoinFuture<'a, T>
    where
        T: Send + 'static,
    {
        async move {
            handle.await
        }
    }


    type SleepFuture<'a> =
        impl core::future::Future<Output = ()> + Send + 'a
    where
        Self: 'a;

    fn sleep<'a>(&'a self, dur: Duration) -> Self::SleepFuture<'a> {
        async move {
            compio::runtime::time::sleep(dur).await;
        }
    }

    type TcpStream = compio::net::TcpStream;
    type TcpListener = compio::net::TcpListener;

    type TcpConnectFuture<'a> =
        impl core::future::Future<Output = io::Result<Self::TcpStream>> + Send + 'a
    where
        Self: 'a;

    type TcpBindFuture<'a> =
        impl core::future::Future<Output = io::Result<Self::TcpListener>> + Send + 'a
    where
        Self: 'a;

    type TcpAcceptFuture<'a> =
        impl core::future::Future<Output = io::Result<(Self::TcpStream, SocketAddr)>> + Send + 'a
    where
        Self: 'a;

    fn tcp_connect<'a>(&'a self, addr: SocketAddr) -> Self::TcpConnectFuture<'a> {
        async move {
            compio::net::TcpStream::connect(addr)
                .await
                .map_err(Into::into)
        }
    }

    fn tcp_bind<'a>(&'a self, addr: SocketAddr) -> Self::TcpBindFuture<'a> {
        async move {
            compio::net::TcpListener::bind(addr)
                .await
                .map_err(Into::into)
        }
    }

    fn tcp_accept<'a>(&'a self, listener: &'a Self::TcpListener) -> Self::TcpAcceptFuture<'a> {
        async move {
            listener
                .accept()
                .await
                .map_err(Into::into)
        }
    }
}