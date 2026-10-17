# Original arch model conversion contract

The experimental converter imports an embedded glTF2 GLB, not a rendered image.
It applies scene node affine transforms and supports static indexed triangles,
opaque constant baseColorFactor materials and finite coordinates within64 tiles.
It rejects skins, animations, textures, morph targets, required extensions,
singular transforms, malformed buffer accessors, count/section overflow and
triangles that collapse after quantization. Closed edges must appear exactly
twice with opposite directions. This proves local winding consistency; it does
not prove arbitrary meshes are free of self-intersections or outward globally.
The original Blender generator recalculates outward normals on each closed part.

One Blender unit equals one tile. Blender exports glTF Y-up. The importer maps
glTF XYZ to native `[x,-y,-z]*128`, rounds to integer model units and deduplicates
positions. Negative native Y is up. The coordinate transform has positive
determinant; a reflected node reverses triangle winding explicitly.

The existing native decoder supports four formats. This POC writes its smallest
untextured Legacy format:18-byte footer, u16 vertex/face counts, signed-smart
vertex deltas, type1 triangle-index deltas, flat render type1, u16 face colors,
no texture/skin/animation/alpha sections. Delta bounds are -16384..16383;
section lengths fit u16. The encoder emits the actual imported triangles.
The independent Rust oracle decodes with the existing Model::decode and compares
every vertex, triangle index, color and render type with the importer output.

Linear material RGB becomes sRGB, then packed HSL6/3/7. Native scene normals,
directional illumination, palette brightness and rasterization are unchanged.
Metallic/roughness factors have no equivalent in this format and are discarded.
Blender's preview lighting is illustrative and is not expected to match pixels.

Model1000000 is referenced by object65000 through opcode7 with a32-bit model ID.
The normal wire location ID is u16, so object IDs65000 and65001 remain below65536.
The visual shape10 object has a4×1 footprint and no rectangular collision.
Two ordinary shape22, model-free footing locations, each with block_walk1,
provide per-column collision. The resulting two-tile central passage is open.
Rotation1 uses the normal client model rotation and swaps the footprint axes.
This is authored compound tile collision, not triangle-mesh physics.
