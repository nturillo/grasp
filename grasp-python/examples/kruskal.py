import grasp
import gdraw

# -----------------------
# Build original graph
# -----------------------
g = grasp.SparseGraph()

edges = [
    (0, 1),
    (0, 2),
    (1, 3),
    (2, 3),
    (2, 4),
    (3, 5),
    (4, 5),
    (4, 6),
    (5, 7),
    (6, 7),
]

for u, v in edges:
    g.add_edge(u, v)

weights = {
    (0, 1): 4.0,
    (0, 2): 2.0,
    (1, 3): 5.0,
    (2, 3): 1.0,
    (2, 4): 7.0,
    (3, 5): 3.0,
    (4, 5): 2.0,
    (4, 6): 3.0,
    (5, 7): 1.0,
    (6, 7): 5.0,
}

# -----------------------
# Show original graph
# -----------------------
print("\nOpening ORIGINAL graph...")
gdraw.open_app_with_graph(g)

# -----------------------
# Run Kruskal
# -----------------------
mst_edges, total_weight = g.kruskal(weights)

print("\n=== KRUSKAL MST ===")
for u, v in mst_edges:
    w = weights.get((u, v), weights.get((v, u), None))
    print(f"{u} -- {v} (weight={w})")
print(f"Total weight: {total_weight}")

# -----------------------
# Build MST graph
# -----------------------
mst_graph = grasp.SparseGraph()

# IMPORTANT: add all vertices (so layout stays consistent)
for v in g.vertices():
    mst_graph.add_vertex(v)

# Add only MST edges
for u, v in mst_edges:
    mst_graph.add_edge(u, v)

# -----------------------
# Show MST graph
# -----------------------
print("\nOpening MST graph (after Kruskal)...")
gdraw.open_app_with_graph(mst_graph)

print("Done.")