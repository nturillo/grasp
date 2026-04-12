// Vertices. Each vertex holds a start index into the edge list (adjacency list)
@group(0) @binding(0)
var<storage, read> vert: array<u32>;
// Holds merged adjacency lists of vertices
@group(0) @binding(1)
var<storage, read> adj: array<u32>;
// Stores the color calculation
@group(0) @binding(2)
var<storage, read> color_in: array<u32>;
@group(0) @binding(3)
var<storage, read_write> color_out: array<u32>;

@compute @workgroup_size(64)
fn calculate_color(@builtin(global_invocation_id) gid: vec3<u32>) {
    let start = vert[gid.x];
    let end = vert[gid.x+1u];

    var sum = 0u;
    var xor = 0u;
    var sq_sum = 0u;

    var i = 0u;
    loop{
        if (i>=end) {break;}
        let u = adj[i];
        let c = color_in[u];

        sum += c;
        xor ^= c;
        sq_sum += c*c;

        i = i+1u;
    }
    color_out[gid.x] = combine(color_in[gid.x], end-start, sum, xor, sq_sum);
}

fn mix(x: u32) -> u32 {
    var v = x;
    v ^= v >> 16u;
    v *= 0x7feb352du;
    v ^= v >> 15u;
    v *= 0x846ca68bu;
    v ^= v >> 16u;
    return v;
}

fn rotl(x: u32, r: u32) -> u32 {
    return (x << r) | (x >> (32u - r));
}

fn combine(color: u32, degree: u32, sum: u32, xor: u32, sq_sum: u32) -> u32 {
    return
        mix(color) ^ mix(degree) ^
        rotl(mix(sum),7u) ^
        rotl(mix(xor),13u) ^
        rotl(mix(sq_sum),21u);
}
