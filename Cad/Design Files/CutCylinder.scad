 $fa = 1;
 $fs = 0.1;
 
 height = 2.5; // Height of the cylinder
 internal_cylinder = 3; // Diameter of the internal cylinder (cavity)
 external_cylinder = 6; // Diamter of the external cyclinder
 
 difference() {
    cylinder( height, external_cylinder / 2, external_cylinder / 2, true);
    cylinder( height + 1, internal_cylinder / 2, internal_cylinder / 2, true);
 }
 
 