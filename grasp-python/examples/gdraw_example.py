import grasp
import gdraw

g = grasp.SparseGraph()

g.add_edge(1, 2)
g.add_edge(2, 3)
g.add_edge(1, 3)

print("Edges:", g.edges())

gdraw.open_app_with_graph(g)

print("Returned from drawing app")