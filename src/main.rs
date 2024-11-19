#![feature(unwrap_infallible)]

use std::fs::File;
use std::io::BufReader;
use serde_json::{Map, Value};
use websocket::{ClientBuilder, OwnedMessage};
use finalfusion::prelude::*;
use finalfusion::storage::NdArray;
use finalfusion::vocab::FastTextSubwordVocab;
use qdrant_client::{Payload, Qdrant};
use qdrant_client::qdrant::{Distance, PointStruct, CreateCollectionBuilder, ScalarQuantizationBuilder, UpsertPointsBuilder, VectorParamsBuilder};
use uuid::Uuid;
use vader_sentiment::SentimentIntensityAnalyzer;

struct Processor {
    model: Embeddings<FastTextSubwordVocab, NdArray>,
    qdrant: Qdrant,
    vader: SentimentIntensityAnalyzer<'static>,
}

const MODEL_FILE: &str = "/Users/jstanbrough/Downloads/cc.en.300.bin";
impl Processor {
    async fn new() -> Self {
        let vader = vader_sentiment::SentimentIntensityAnalyzer::new();
        let qdrant = Qdrant::from_url("http://localhost:6334").build().expect("failed to connect to qdrant");
        let _ = qdrant.create_collection(
            CreateCollectionBuilder::new("bluesky")
                .vectors_config(VectorParamsBuilder::new(300, Distance::Cosine))
                .quantization_config(ScalarQuantizationBuilder::default()),
        )
            .await;

        let mut reader = BufReader::new(File::open(MODEL_FILE).expect("failed to open embeddings"));

        Processor {
            model: Embeddings::read_fasttext(&mut reader).unwrap(),
            qdrant,
            vader,
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
        let mut v: &Map<String, Value> = v.as_object().unwrap();
        let sentiment_map = self.vader.polarity_scores(text_str);

        let mut cv = Map::<String, Value>::new();

        let compound = sentiment_map.get("compound").unwrap();
        let mut sentiment = 0.0;
        if compound.abs() > 0.000001 {
            sentiment = if *compound > 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        cv.insert("sentiment".to_string(), sentiment.try_into()?);
        for (k, v) in v {
            cv.insert(k.clone(), v.clone());
        }
        for (k, v) in sentiment_map {
            cv.insert(k.to_string(), v.try_into()?);
        }
        let payload: Payload = cv.try_into()?;

        let embeds = self.model.embedding(text_str);
        match embeds {
            Some(embeds) => {
                let embeds = embeds.to_vec();
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
fn test_build() {}
