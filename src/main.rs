use serde_json::Value;
use websocket::{ClientBuilder, OwnedMessage};
use websocket::futures::{Future, IntoFuture, Stream};

#[derive(Clone, Default)]
struct Processor {}

impl Processor {
    fn process(&mut self, text: String) -> Result<(), anyhow::Error> {
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
        
        println!("{}", text);
        Ok(())
    }
}
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let url = "ws://localhost:6008/subscribe?wantedCollections=app.bsky.feed.post";
    let client = ClientBuilder::new(url)?
        .add_protocol("rust-websocket")
        .async_connect_insecure();

    let mut processor = Processor::default();

    let _ = client.and_then(|(duplex, _)| {
        let (sink, stream) = duplex.split();
        stream.filter_map(|m| {
            match m {
                OwnedMessage::Text(t) => {
                    if processor.process(t).is_err() {
                        println!("Error deserializing")
                    }
                    None
                }
                _ => None
            }
        })
            .forward(sink)
    }).wait()?;
    Ok(())
}
