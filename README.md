
# imgquant
incomplete.

### plans
- octree with imagemagick's error pruning with YUV
- dithering with sierra lite

possible optimizations:
- flattened octree to a vector to avoid indirection for every node.
- option to turn off imagemagick's error pruning & switch to 4x4 8x8 bayes' ordered dithering

### notes
this is also a bet to see if array indexing is better than pointers for octrees.
