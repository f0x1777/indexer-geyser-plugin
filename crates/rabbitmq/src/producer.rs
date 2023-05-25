//! An AMQP producer configured from a [`QueueType`]

use lapin::{BasicProperties, Channel, Connection};

use crate::{QueueType, Result};

#[cfg(feature = "produce-json")]
use crate::serialize::json;
#[cfg(not(feature = "produce-json"))]
use crate::serialize::serialize;

/// A producer consisting of a configured channel and additional queue config
#[derive(Debug)]
pub struct Producer<Q> {
    chan: Channel,
    ty: Q,
}

impl<Q: QueueType> Producer<Q>
where
    Q::Message: serde::Serialize,
{
    /// Construct a new producer from a [`QueueType`]
    ///
    /// # Errors
    /// This function fails if the channel cannot be created and configured
    /// successfully.
    pub async fn new(conn: &Connection, ty: Q) -> Result<Self> {
        let chan = conn.create_channel().await?;

        ty.info().init_producer(&chan).await?;

        Ok(Self { chan, ty })
    }

    /// Write a single message to this producer
    ///
    /// # Errors
    /// This function fails if the value cannot be serialized or the serialized
    /// payload cannot be transmitted.
    pub async fn write(
        &self,
        val: impl std::borrow::Borrow<Q::Message>,
        routing_key: Option<&str>,
    ) -> Result<()> {
        let _ = &self.write_with_props(val, routing_key, None).await;
        Ok(())
    }

    /// Write a single message to this producer with additional rabbit properties
    ///
    /// # Errors
    /// This function fails if the value cannot be serialized or the serialized
    /// payload cannot be transmitted.
    pub async fn write_with_props(
        &self,
        val: impl std::borrow::Borrow<Q::Message>,
        routing_key: Option<&str>,
        rabbit_props: Option<&BasicProperties>,
    ) -> Result<()> {
        let val = val.borrow();

        let mut vec = Vec::new();

        #[cfg(feature = "produce-json")]
        json(&mut vec, val)?;
        #[cfg(not(feature = "produce-json"))]
        serialize(&mut vec, val)?;

        self.ty
            .info()
            .publish(&self.chan, &vec, routing_key, rabbit_props)
            .await?
            .await?;

        Ok(())
    }
}
