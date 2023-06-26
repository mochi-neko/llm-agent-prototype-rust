use anyhow::{Ok, Result};
use qdrant_client::prelude::{QdrantClient, QdrantClientConfig};

use super::database::DataBase;

pub(crate) struct VectorMemories<'a> {
    pub(crate) session: DataBase<'a>,
    // pub(crate) persistent: DataBase<'a>,
}

impl<'a> VectorMemories<'a> {
    pub(crate) async fn new() -> Result<VectorMemories<'a>> {
        let session = initialize_vector_db("session", true).await?;
        // let persistent = initialize_vector_db("persistent", true).await?;

        Ok(VectorMemories {
            session,
            // persistent,
        })
    }
}

async fn initialize_vector_db(name: &str, reset: bool) -> Result<DataBase> {
    let config = QdrantClientConfig::from_url("http://qdrant:6334");
    let client = QdrantClient::new(Some(config))?;

    let session = DataBase {
        client,
        name,
        index: 0,
    };
    if reset {
        session.reset().await?;
    }

    Ok(session)
}
