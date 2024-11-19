#![feature(unwrap_infallible)]

use std::fs::File;
use std::io::BufReader;
use serde_json::Value;
use websocket::{ClientBuilder, OwnedMessage};
use finalfusion::prelude::*;
use finalfusion::storage::NdArray;
use finalfusion::vocab::FastTextSubwordVocab;
use qdrant_client::{Payload, Qdrant};
use qdrant_client::qdrant::{Distance, PointStruct, CreateCollectionBuilder, ScalarQuantizationBuilder, UpsertPointsBuilder, VectorParamsBuilder};
use uuid::Uuid;

struct Processor {
    model: Embeddings<FastTextSubwordVocab, NdArray>,
    qdrant: Qdrant,
}


impl Processor {
    async fn new() -> Self {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().expect("failed to connect to qdrant");
        let _ = qdrant.create_collection(
            CreateCollectionBuilder::new("bluesky")
                .vectors_config(VectorParamsBuilder::new(300, Distance::Cosine))
                .quantization_config(ScalarQuantizationBuilder::default()),
        )
            .await;

        let file = "/Users/jstanbrough/Downloads/cc.en.300.bin";
        let mut reader = BufReader::new(File::open(file).expect("failed to open embeddings"));

        Processor {
            model: Embeddings::read_fasttext(&mut reader).unwrap(),
            qdrant,
        }
    }
    async fn process(&mut self, text: String) -> Result<(), anyhow::Error> {
        let v: Value = serde_json::from_str(text.as_str())?;
        let kind = v["kind"].clone();

        if kind != "commit" {
            return Ok(());
        }

        let commit = v["commit"].clone();
        if commit["operation"] != "create" {
            return Ok(());
        }

        let record = commit["record"].clone();
        let langs = record["langs"].clone();
        let text = record["text"].clone();

        let langs = langs.as_array();
        if langs.is_none() {
            return Ok(());
        }

        let langs = langs.unwrap();
        if langs.len() != 1 {
            return Ok(());
        }

        if langs[0] != "en" {
            return Ok(());
        }

        if text == "" {
            return Ok(());
        }

        let text_str = text.as_str().unwrap();
        let embeds = self.model.embedding(text_str);
        match embeds {
            Some(embeds) => {
                let embeds = embeds.to_vec();
                let payload: Payload = v.try_into()?;
                let points = vec![PointStruct::new(Uuid::new_v4().to_string(), embeds, payload)];
                self.qdrant
                    .upsert_points(UpsertPointsBuilder::new("bluesky", points))
                    .await?;
                println!("{}", text_str)
            }
            _ => println!("failed to embed {}", text)
        }
        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let url = "ws://localhost:6008/subscribe?wantedCollections=app.bsky.feed.post";
    let client = ClientBuilder::new(url)?
        .add_protocol("rust-websocket")
        .connect_insecure()?;

    let mut processor = Processor::new().await;
    let (mut reader, _) = client.split()?;
    loop {
        let message = reader.recv_message()?;
        match message {
            OwnedMessage::Text(text) => {
                processor.process(text).await?
            }
            _ => {}
        }
    }
}

#[test]
fn test_proc() {}