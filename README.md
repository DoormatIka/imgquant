
# imgquant
Simple image quantizer. CPU-bound.

`imgquant -h`

### features
- basic octree
- sierra lite and floyd-steinberg dithering

### plans
- clean everything up!
- switch out String with OsString for cli.
- octree with imagemagick's error pruning with YUV
- flattened octree using morton order to avoid indirection for every node.
- option to turn off diffusion based dithering & switch to 4x4 or 8x8 bayes' ordered dithering
- out of core implementation for massive images: [related issue](https://github.com/DoormatIka/imgquant/issues/1)
- parallelization of octrees (WHY ARE ALL THE PAPERS PAYWALLED??)

### informal benchmarks
all using floydsteinberg dithering.
- 1381 x 1381 "Kaguya.png" | 120.1192ms (init), 86.8429ms (quant), 256 colors, depth 6
- 1381 x 1381 "Kaguya.png" | 137.909ms (init), 81.3696ms (quant), 256 colors, depth 8
- 600 x 546 "elonma.jpg" | 18.5035ms (init), 13.6052ms (quant), 256 colors, depth 6
- 600 x 546 "elonma.jpg" | 38.0389ms (init), 15.8098ms (quant), 256 colors, depth 8
- 4288 x 2848 "big_sky.jpg" | 551.3375ms (init), 529.0182ms (quant), 256 colors, depth 6
- 4288 x 2848 "big_sky.jpg" | 889.7646ms (init), 535.0434ms (quant), 256 colors, depth 8
