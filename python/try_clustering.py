import numpy as np
# python3 -m pip install scipy
# python3 -m pip install matplotlib --force-reinstall
from scipy.cluster.hierarchy import dendrogram, linkage
import matplotlib.pyplot as plt

# Assuming 'equity_data' is a 2D array where each row represents the equity distribution
# of a pair of hole cards across the flops

import numpy as np

# Generate random equity data for demonstration purposes
# 100 rows (hole card pairs) and 30 columns (different flop scenarios)

np.random.seed(0)  # Seed for reproducibility
equity_data = np.random.rand(100, 30)


# Perform hierarchical clustering
Z = linkage(equity_data, method='ward')

# # Plot the dendrogram
# plt.figure(figsize=(25, 10))
# dendrogram(Z)
# plt.show()


# Plot the dendrogram
plt.figure(figsize=(25, 10))
dendrogram(Z)
plt.title("Hierarchical Clustering Dendrogram")
plt.xlabel("Hole Card Pairs")
plt.ylabel("Distance")

# Save the plot to a PNG file
plt.savefig("hierarchical_clustering_dendrogram.png")

# Close the plot
plt.close()