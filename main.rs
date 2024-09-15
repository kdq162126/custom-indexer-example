use tokio::signal;
use tokio::sync::oneshot;
use anyhow::Result;
use async_trait::async_trait;
use sui_types::{event, full_checkpoint_content::CheckpointData};
use sui_data_ingestion_core::{Worker, setup_single_workflow};
use bcs::{from_bytes, to_bytes};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketBought {
    receiver: [u8; 32], 
    spin_id: u64,
    price: u64,
    amount: u64,
    coin_type: String,
}

fn deserialize_ticket_bought(data: &[u8]) -> Result<TicketBought, Box<dyn Error>> {
    println!("Raw BCS Data: {:?}", data);
    
    match from_bytes(data) {
        Ok(decoded) => Ok(decoded),
        Err(e) => {
            eprintln!("Failed to decode BCS data: {}", e);
            Err(Box::new(e))
        }
    }
}

struct CustomWorker;

#[async_trait]
impl Worker for CustomWorker {
    async fn process_checkpoint(&self, checkpoint: CheckpointData) -> Result<()> {
        println!("Processing checkpoint: {}", checkpoint.checkpoint_summary.to_string());

        for tx in checkpoint.transactions.iter() {
            let events = &tx.events;
            for event in events.iter() {
                let data = &event.data;
                for _data in data.iter() {
                    if _data.type_.name.as_str() == "TicketBought" {
                        println!(">>>>>>>>>>>>>>>>>>>>>>>>>");
                        println!("Event: {:#?}", event);

                        match deserialize_ticket_bought(&_data.contents) {
                            Ok(ticket_bought) => println!("Decoded Content: {:?}", ticket_bought),
                            Err(e) => eprintln!("Failed to decode BCS data: {}", e),
                        }
                        println!("<<<<<<<<<<<<<<<<<<<<<<<<");
                    }
                }
            }
        }

        println!("-----------------------------------------------------------------");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let (executor, term_sender) = setup_single_workflow(
        CustomWorker,
        "https://checkpoints.mainnet.sui.io".to_string(),
        58_542_633, /* initial checkpoint number */
        50, /* concurrency */
        None, /* extra reader options */
    ).await?;
    executor.await?;
    Ok(())
}
