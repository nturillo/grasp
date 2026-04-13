import grasp

g = grasp.SparseGraph()

# Build graph
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

# Weight function (handles undirected edges)
def weight_fn(u, v):
    return weights.get((u, v), weights.get((v, u), 1.0))

# Simple heuristic (admissible-ish for demo)
def heuristic(v):
    # crude estimate: distance to target node 7
    h = {
        0: 3, 1: 3, 2: 2, 3: 2,
        4: 2, 5: 1, 6: 1, 7: 0
    }
    return h.get(v, 0)


def reconstruct_path(predecessors, source, target):
    path = []
    cur = target
    while cur in predecessors:
        path.append(cur)
        cur = predecessors[cur]
    path.append(source)
    path.reverse()
    return path


def print_all_paths_from_predecessors(title, distances, predecessors, source):
    print(f"\n=== {title} ===")
    for v in sorted(distances):
        path = reconstruct_path(predecessors, source, v)
        print(f"{source} -> {v}: path={path}, cost={distances[v]}")


def print_all_paths_astar(title, source, g, weighted=False):
    print(f"\n=== {title} ===")
    for v in sorted(g.vertices()):
        if weighted:
            path, cost = g.astar_weighted(source, v, weight_fn, heuristic)
        else:
            path, cost = g.astar_unweighted(source, v)

        print(f"{source} -> {v}: path={path}, cost={cost}")

source = 0
target = 7

dist_u, prev_u = g.dijkstra_unweighted(source)
print_all_paths_from_predecessors(
    "UNWEIGHTED DIJKSTRA",
    dist_u,
    prev_u,
    source
)

dist_w, prev_w = g.dijkstra_weighted(source, weight_fn)
print_all_paths_from_predecessors(
    "WEIGHTED DIJKSTRA",
    dist_w,
    prev_w,
    source
)

path_u_astar, cost_u_astar = g.astar_unweighted(source, target)
print_all_paths_astar(
    "UNWEIGHTED A*",
    source,
    g,
    weighted=False
)

path_w_astar, cost_w_astar = g.astar_weighted(source, target, weight_fn, heuristic)
print_all_paths_astar(
    "WEIGHTED A*",
    source,
    g,
    weighted=True
)