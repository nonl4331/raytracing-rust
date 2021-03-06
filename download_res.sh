#!/bin/sh

mkdir res

curl https://eoimages.gsfc.nasa.gov/images/imagerecords/73000/73909/world.topo.bathy.200412.3x5400x2700.png --output res/earth.png

curl https://www.prinmath.com/csci5229/OBJ/bunny.zip --output res/bunny.unzip
unzip res/bunny -d res/
rm res/bunny.zip

curl http://www.mrbluesummers.com/wp-content/uploads/2010/07/stanford_dragon.zip --output res/stanford_dragon.zip
unzip res/stanford_dragon -d res/
rm res/dragon.ASE res/dragon.DWF res/dragon.max 'res/From MrBluesummers.com.txt' res/stanford_dragon.zip

echo "The dragon.obj does not have vertex normals generate them in blender or it cannot be used"
