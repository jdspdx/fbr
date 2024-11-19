#%%
import matplotlib.pyplot as plt
from sklearn.manifold import TSNE
from qdrant_client import QdrantClient
import numpy as np

# Step 1: Connect to Qdrant
client = QdrantClient("http://localhost:6333")  # Adjust the host and port as needed

# Specify the collection from which to fetch vectors
collection_name = "bluesky"

# Fetch all vectors and their "compound" property from the specified Qdrant collection
def fetch_vectors_and_properties(client, collection_name, limit=1000):
    """Fetch vectors and their 'compound' property from Qdrant collection."""
    response = client.scroll(
        collection_name=collection_name,
        limit=limit,
        with_vectors=True,
        with_payload=True
    )
    vectors = []
    compounds = []
    for point in response[0]:
        vectors.append(point.vector)
        compounds.append(point.payload.get("compound", 0))  # Default to 0 if "compound" is missing
    return np.array(vectors), np.array(compounds)

# Step 2: Fetch vectors and their properties
print("Fetching vectors and properties from Qdrant...")
vectors, compounds = fetch_vectors_and_properties(client, collection_name)

# Step 3: Perform t-SNE dimensionality reduction to 2D
print("Performing t-SNE dimensionality reduction to 2D...")
tsne = TSNE(n_components=2, random_state=42, perplexity=30)
tsne_results = tsne.fit_transform(vectors)

# Step 4: Normalize the "compound" values for coloring
# Normalize compound values to [0, 1] for color mapping
compound_norm = (compounds - compounds.min()) / (compounds.max() - compounds.min())

# Step 5: Plot the results in 2D using Matplotlib with color mapping
print("Visualizing results in 2D with color mapping...")
plt.figure(figsize=(10, 7))
sc = plt.scatter(
    tsne_results[:, 0],
    tsne_results[:, 1],
    c=compound_norm,  # Color based on normalized compound values
    cmap='viridis',   # Choose a colormap, e.g., 'viridis', 'plasma', 'coolwarm'
    alpha=0.8,
    edgecolor='k',
    s=50
)

# Add a color bar to indicate the "compound" value range
cb = plt.colorbar(sc)
cb.set_label("Compound Value (Normalized)", fontsize=12)

plt.title("2D t-SNE Visualization of Qdrant Vectors", fontsize=16)
plt.xlabel("t-SNE Component 1", fontsize=12)
plt.ylabel("t-SNE Component 2", fontsize=12)
plt.grid(True)
plt.show()
