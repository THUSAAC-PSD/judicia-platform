use anyhow::Result;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties,
};
use shared::JudgingJob;

pub struct Queue {
    connection: Connection,
}

impl Queue {
    pub async fn new(rabbitmq_url: &str) -> Result<Self> {
        let connection = Connection::connect(rabbitmq_url, ConnectionProperties::default()).await?;
        
        // Create channel and declare queue
        let channel = connection.create_channel().await?;
        channel
            .queue_declare(
                "judging_jobs",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(Queue { connection })
    }

    pub async fn publish_judging_job(&self, job: &JudgingJob) -> Result<()> {
        let channel = self.connection.create_channel().await?;
        
        let payload = serde_json::to_vec(job)?;
        
        let confirm = channel
            .basic_publish(
                "",
                "judging_jobs",
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await?
            .await?;

        match confirm {
            Confirmation::Ack(_) => Ok(()),
            Confirmation::Nack(_) => Err(anyhow::anyhow!("Failed to publish message")),
            Confirmation::NotRequested => {
                // Handle the NotRequested case appropriately
                todo!()
            }
        }
    }
}