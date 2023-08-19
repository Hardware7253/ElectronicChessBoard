 $fa = 1;
 $fs = 0.1;
 
 // Chess magnet holder pieces
 // These pieces are glued to the bottom of normal chess pieces
 //  Recommended to be printed with a clear filament to diffuse the LED light from the PCB
 
piece_base_diameter = 20; // Diameter of the chess piece

magnet_diameter = 12; // Diameter of the magnet
magnet_height = 1; // Height of the magnet

height_from_board = 5; // Total distance to raise the attached piece from the board

difference() {
    
    // Main base piece cylinder
    cylinder( height_from_board, piece_base_diameter / 2,  piece_base_diameter / 2, false);
    
    // Magnet pocket cut cylinder
    translate([0, 0, height_from_board - magnet_height])
        cylinder(magnet_height + 0.01 , (magnet_diameter / 2), magnet_diameter / 2, false);
}
 