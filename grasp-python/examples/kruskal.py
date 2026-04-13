import grasp

g = grasp.SparseGraph()

# Build graph (same structure as before)
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

# Assign weights
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

# Pretty print MST
def print_mst(title, mst_edges, total_weight):
    print(f"\n=== {title} ===")
    print("Edges in MST:")
    for u, v in mst_edges:
        w = weights.get((u, v), weights.get((v, u), None))
        print(f"{u} -- {v} (weight={w})")
    print(f"Total weight: {total_weight}")


# Run Kruskal
mst_edges, total_weight = g.kruskal(weights)

print_mst("KRUSKAL MST", mst_edges, total_weight)