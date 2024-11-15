# voxeltorus

A raycasting voxel engine written in Rust using [Macroquad](https://github.com/not-fl3/macroquad). It uses the Fast Voxel Traversal Algorithm on a linked list resulting in non-Euclidean space. For example, you can have space that loops around in all 3 dimensions (aka the 3-torus) as well as seamless portals.

To use it, clone the repository with git, then build and run:

```sh
git clone https://github.com/mdnestor/voxeltorus.git
cd voxeltorus
cargo build
cargo run --release
```

![](image.png)
