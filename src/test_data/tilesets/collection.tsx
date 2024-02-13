<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.1" name="test_tileset" tilewidth="320" tileheight="200" tilecount="4" columns="0">
 <grid orientation="orthogonal" width="1" height="1"/>
 <tile id="0" x="1" y="2" width="3" height="4">
  <properties>
   <property name="is_steve" type="bool" value="true"/>
  </properties>
  <image width="320" height="200" source="vikings_of_midgard.png"/>
 </tile>
 <tile id="1">
  <properties>
   <property name="is_steve" type="bool" value="false"/>
  </properties>
  <image width="16" height="16" source="tile_16x16.png"/>
 </tile>
 <tile id="2">
  <image width="32" height="32" source="tile_32x32.png"/>
 </tile>
 <tile id="3">
  <image width="320" height="200" source="vikings_of_midgard_alt.png"/>
 </tile>
</tileset>
