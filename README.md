# Image Grouper
An experimental cli tool for unix systems that intends to group images together using perceptual hashing functions.

# How it works
Basically works by running a perceptual hash through all the images. Each hash is treated like a vertex  edges are weighted by the hamming distance between two hashes. The graph is also assumed to be connected. 

The program "groups" by finding a low cost hamiltonian circuit about the connected graph.

# How to use 
try

```
cargo run --release --  ~/Pictures -o ~/Pictures/sorted 
```
