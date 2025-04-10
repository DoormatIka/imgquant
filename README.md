
# imgquant
Simple image quantizer. CPU-bound.

`imgquant --input [file path] --color [color count] --dither [base, sierralite]`

`imgquant -i alice.png -c 256 -d base`

### features
- basic octree
- sierra lite and floyd-steinberg dithering
- fairly fast (80-200ms~ on a 2304 x 1792 image)

### plans
- refactor code
- switch out String with OsString for cli
- octree node pruning on YUV errors instead of in-order RGB errors
- 4x4 or 8x8 bayes' ordered dithering for gifs
- flattened octree using morton order to avoid indirection for every node
- parallelization of octrees (WHY ARE ALL THE PAPERS PAYWALLED??)
