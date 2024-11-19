# Famous Blue Raincoat (FBR)

- Load skeets from the Jetstream
- Perform basic VADER-ish sentiment analysis on the skeet text
- Generate embeddings using a fasttext model
- Save into Qdrant
- Visualize with t-SNE

Very WIP

# Models

I am using https://dl.fbaipublicfiles.com/fasttext/vectors-crawl/cc.en.300.bin.gz

It's almost 7 gigs uncompressed. The embedding library used, `finalfussion`, also supports word2vec and GloVE binary models.
