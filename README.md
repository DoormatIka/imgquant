
# imgquant
Simple image quantizer.

`imgquant -- -i alice.png -c 256`

### features
- basic octree
- sierra lite dithering
- fairly fast (500ms~ on a 2304 x 1792 image)

### plans
- clean everything up!
- switch out String with OsString for cli.
- octree with imagemagick's error pruning with YUV
- flattened octree using morton order to avoid indirection for every node.
- option to turn off sierra lite dithering & switch to 4x4 or 8x8 bayes' ordered dithering
