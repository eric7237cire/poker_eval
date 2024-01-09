import numpy as np
# python3 -m pip install scipy
# python3 -m pip install matplotlib --force-reinstall
from scipy.cluster.hierarchy import dendrogram, linkage
import matplotlib.pyplot as plt
from sklearn.decomposition import PCA

# Assuming 'equity_data' is a 2D array where each row represents the equity distribution
# of a pair of hole cards across the flops

import numpy as np
import pandas as pd
from sklearn.cluster import KMeans
from sklearn.preprocessing import LabelEncoder, StandardScaler
import matplotlib.pyplot as plt
from scipy.spatial.distance import cdist

def h_cluster():
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

def k_means():
        
    # Step 2: Read the CSV file
    df = pd.read_csv('/home/eric/git/poker_eval/data/hole_card_data.csv', header=None)

    card_labels = df.iloc[:, 1]  # Store the card labels
    features = df.iloc[:, 2:102]  # Select the columns with features


    # Step 2: Preprocess the data
    scaler = StandardScaler()
    scaled_features = scaler.fit_transform(features)

    num_clusters = 25

    # Step 3: Perform K-means clustering
    # Choose the number of clusters (k), for example, k=5
    kmeans = KMeans(n_clusters=num_clusters, random_state=42)
    kmeans.fit(scaled_features)

    # Adding cluster labels to the original data
    df['cluster'] = kmeans.labels_

    # Step 4: Analyze the Results
    print(df.head())

    pca = PCA(n_components=2)  # Reduce to 2 dimensions for plotting
    reduced_data = pca.fit_transform(scaled_features)

    # Step 5: Plot the Results
    plt.figure(figsize=(12, 8))
    colors = plt.cm.get_cmap('viridis', num_clusters)  # Color map for 5 clusters

    # plt.scatter(reduced_data[:, 0], reduced_data[:, 1], c=df['cluster'], cmap='viridis', marker='o')
    for i, point in enumerate(reduced_data):
        plt.scatter(point[0], point[1], color=colors(df['cluster'][i]))
        plt.text(point[0], point[1], card_labels[i], fontsize=9)

    plt.title('K-means Clustering with 2D PCA')
    plt.xlabel('PCA Feature 1')
    plt.ylabel('PCA Feature 2')
    plt.colorbar(label='Cluster')
    #plt.show()

    plt.savefig("/home/eric/git/poker_eval/data/cluster.png")

    features['cluster'] = df['cluster']
    cluster_means = features.groupby('cluster').mean()

    # Step 2: Sort Clusters by Mean
    sorted_cluster_indices = cluster_means.mean(axis=1).sort_values().index

    for cluster in sorted_cluster_indices:
        cluster_cards = df[df['cluster'] == cluster][1]
        print(f"Cluster {cluster}: {', '.join(cluster_cards)}")

# Function to calculate Chi-Square distance
def chi_square_distance(hist1, hist2):
    # Add a small number to avoid division by zero
    epsilon = 1e-10
    return np.sum((hist1 - hist2) ** 2 / (hist1 + hist2 + epsilon))

def produce_histograms():

    df = pd.read_csv('/home/eric/git/poker_eval/data/hole_card_data.csv', header=None)

    card_labels = df.iloc[:, 1]  # Store the card labels
    features = df.iloc[:, 2:]  # Select the columns with features
    
    fixed_card_labels = []
    for i in range(0, card_labels.size):
        orig_label = card_labels[i]
        if orig_label[0] == orig_label[2]:
            fixed_card_labels.append(f"{orig_label[0]}{orig_label[2]}")
        elif orig_label[1] == orig_label[3]:
            fixed_card_labels.append(f"{orig_label[0]}{orig_label[2]}s")
        else:
            fixed_card_labels.append(f"{orig_label[0]}{orig_label[2]}o")

        
    # Create histograms (assuming they are not already in histogram form)
    # histogram returns hist and bin_edges
    bin_edges = []
    histograms = []

    create_charts = True

    for index, row in features.iterrows():
        print(f"Line {index}: {card_labels[index]}")
        h, be = np.histogram(row, bins=40, range=(0, 1))
        
        histograms.append(h)
        bin_edges.append(be)

        if create_charts:
            plt.figure(figsize=(10, 6))

            bin_width = be[1] - be[0]

            plt.bar(be[:-1], h, width=bin_width, edgecolor='black')
            plt.title(f'Histogram {fixed_card_labels[index]}')
            plt.xlabel('Bins')
            plt.ylabel('Frequency')
            plt.savefig(f"/home/eric/git/poker_eval/data/histogram_{fixed_card_labels[index]}.png")
            plt.close()

    # Calculate the Chi-Square distance matrix
    #chi_square_matrix = cdist(histograms, histograms, metric=chi_square_distance)

    euclidean_distance_matrix = cdist(histograms, histograms, metric='euclidean')


    num_clusters = 25  # Define the number of clusters
    kmeans = KMeans(n_clusters=num_clusters, random_state=42)
    # clusters = kmeans.fit_predict(chi_square_matrix)
    clusters = kmeans.fit_predict(euclidean_distance_matrix)

    # Add cluster information to the DataFrame
    features['cluster'] = clusters

    cluster_means = features.groupby('cluster').mean()

    # Step 2: Sort Clusters by Mean
    sorted_cluster_indices = cluster_means.mean(axis=1).sort_values().index

    

    for cluster in sorted_cluster_indices:
        hc_indexes = features[features['cluster'] == cluster].index.tolist()
        print(f"*****Cluster {cluster}*****")
        for hc_index in hc_indexes:
            print(f"{fixed_card_labels[hc_index]}")
        #print(f"Histograms in Cluster {i}: {}")

if __name__ == "__main__":
    # h_cluster()
    # k_means()    
    produce_histograms()