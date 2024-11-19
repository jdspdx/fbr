# Famous Blue Raincoat (FBR)

- Load skeets from the Jetstream
- Perform basic VADER-ish sentiment analysis on the skeet text
- Generate embeddings using a fasttext model
- Save into Qdrant
- Visualize with t-SNE

Very WIP

This project runs both Jetstream and Qdrant in Docker.


# Running Jetstream

To run jetstream
- `git submodule update --init`
- Run `make up` in the `jetstream` directory.

You can also use the public Jetstream servers, just change the URL in the code. I'll make this configurable later.

# Running Qdrant
- `cd qdrant`
- `./run.sh

# Models

I am using https://dl.fbaipublicfiles.com/fasttext/vectors-crawl/cc.en.300.bin.gz

It's almost 7 gigs uncompressed. The embedding library used, `finalfussion`, also supports word2vec and GloVE binary models.

Set src/main.rs `MODEL_FILE` path name to your download location
