<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.1" name="vikings_of_midgard" tilewidth="20" tileheight="20" tilecount="160" columns="16">
 <properties>
  <property name="best_color" type="color" value="#ff83947b"/>
 </properties>
 <image source="vikings_of_midgard.png" trans="ff00ff" width="320" height="200"/>
 <tile id="0">
  <properties>
   <property name="is_steve" type="bool" value="true"/>
  </properties>
 </tile>
 <tile id="1">
  <properties>
   <property name="is_steve" type="bool" value="false"/>
  </properties>
 </tile>
 <tile id="22">
  <properties>
   <property name="is_jerry" type="bool" value="true"/>
  </properties>
 </tile>
 <tile id="144">
  <animation>
   <frame tileid="144" duration="100"/>
   <frame tileid="145" duration="100"/>
   <frame tileid="146" duration="100"/>
   <frame tileid="147" duration="100"/>
  </animation>
 </tile>
 <wangsets>
  <wangset name="grass" type="corner" tile="-1"/>
 </wangsets>
</tileset>
