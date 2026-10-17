"""Original segmented stone arch. Run with Blender --background --factory-startup.

No cache models, textures, images or external asset libraries are used.
Blender units are tiles; export uses real evaluated and triangulated mesh geometry.
"""
import argparse
import hashlib
import json
import math
import sys
from pathlib import Path
import bpy
import bmesh
from mathutils import Vector


def main():
    args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args(args)
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    materials = []
    for name, color in [('Quarried stone', (.29, .265, .225, 1)),
                        ('Cut stone', (.41, .365, .28, 1)),
                        ('Forged iron', (.075, .085, .095, 1)),
                        ('Keystone ochre', (.43, .22, .075, 1))]:
        m = bpy.data.materials.new(name)
        m.diffuse_color = color
        m.use_nodes = True
        m.use_backface_culling = True
        bsdf = m.node_tree.nodes.get('Principled BSDF')
        bsdf.inputs['Base Color'].default_value = color
        bsdf.inputs['Roughness'].default_value = .87
        materials.append(m)
    pieces = []

    def finish(obj, material, bevel):
        obj.data.materials.append(materials[material])
        bpy.context.view_layer.objects.active = obj
        obj.select_set(True)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
        if bevel:
            modifier = obj.modifiers.new('Hand cut bevel', 'BEVEL')
            modifier.width = bevel
            modifier.segments = 1
            bpy.ops.object.modifier_apply(modifier=modifier.name)
        bm = bmesh.new()
        bm.from_mesh(obj.data)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        bm.to_mesh(obj.data)
        bm.free()
        for face in obj.data.polygons:
            face.use_smooth = False
        pieces.append(obj)
        obj.select_set(False)

    def block(name, location, dimensions, material, bevel=.027):
        bpy.ops.mesh.primitive_cube_add(size=1, location=location)
        obj = bpy.context.object
        obj.name = name
        obj.dimensions = dimensions
        finish(obj, material, bevel)

    for side, x in [('Left', -1.5), ('Right', 1.5)]:
        block(side + ' plinth', (x, 0, .12), (.98, 1, .24), 1)
        for course in range(4):
            block(f'{side} course {course}', (x, 0, .52 + course * .49),
                  (.83 if course % 2 else .87, .88, .47), course % 2)
        block(side + ' capital', (x, 0, 2.23), (.98, 1, .20), 1)
        for z in [.79, 1.77]:
            block(side + ' iron collar', (x, 0, z), (.91, .96, .085), 2, .012)
    for i in range(7):
        # Segmented, closed wedge; the small mortar gaps survive quantization.
        lo = i * math.pi / 7 + .008
        hi = (i + 1) * math.pi / 7 - .008
        radius = 1.70 if i != 3 else 1.76
        vertices = []
        for y in [-.44, .44]:
            for r, angle in [(1.01, lo), (radius, lo), (radius, hi), (1.01, hi)]:
                vertices.append((r * math.cos(angle), y, 2.22 + r * math.sin(angle)))
        faces = [(0, 1, 2, 3), (7, 6, 5, 4), (0, 4, 5, 1),
                 (1, 5, 6, 2), (2, 6, 7, 3), (3, 7, 4, 0)]
        mesh = bpy.data.meshes.new(f'Voussoir {i}')
        mesh.from_pydata(vertices, [], faces)
        mesh.update()
        obj = bpy.data.objects.new(f'Arch stone {i}', mesh)
        bpy.context.collection.objects.link(obj)
        finish(obj, 3 if i == 3 else i % 2, .018)
    bpy.ops.object.select_all(action='DESELECT')
    for obj in pieces:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = pieces[0]
    bpy.ops.object.join()
    arch = bpy.context.object
    arch.name = 'Original Cinderworks archway'
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    modifier = arch.modifiers.new('Export triangles', 'TRIANGULATE')
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    bpy.ops.export_scene.gltf(filepath=str(out / 'archway.glb'), export_format='GLB',
                              use_selection=True, export_materials='EXPORT',
                              export_animations=False, export_cameras=False,
                              export_lights=False, export_yup=True)
    bounds = [[min(v.co[i] for v in arch.data.vertices), max(v.co[i] for v in arch.data.vertices)] for i in range(3)]
    geometry = {'blender_version': bpy.app.version_string, 'origin': 'Fully original bpy geometry; no cache asset dependencies',
                'units': 'one Blender unit = one game tile = 128 native model units',
                'bounds_blender_xyz': bounds, 'vertices_blender': len(arch.data.vertices),
                'triangles_blender': len(arch.data.polygons),
                'materials': [{'name': m.name, 'linear_rgba': list(m.diffuse_color)} for m in arch.data.materials],
                'glb_sha256': hashlib.sha256((out / 'archway.glb').read_bytes()).hexdigest(),
                'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    (out / 'blender-manifest.json').write_text(json.dumps(geometry, indent=2) + '\n')
    # Preview-only ground and studio illumination are original and not exported.
    bpy.ops.mesh.primitive_plane_add(size=200)
    ground = bpy.context.object
    ground.name = 'Preview ground only'
    ground.location.z = -.025
    m = bpy.data.materials.new('Preview slate')
    m.diffuse_color = (.075, .09, .11, 1)
    ground.data.materials.append(m)
    bpy.ops.object.camera_add(location=(6.0, -8.0, 5.2))
    camera = bpy.context.object
    camera.rotation_euler = (Vector((0, 0, 1.85)) - camera.location).to_track_quat('-Z', 'Y').to_euler()
    camera.data.type = 'ORTHO'
    camera.data.ortho_scale = 6.9
    bpy.context.scene.camera = camera
    for location, energy, size in [((-4, -4, 7), 1000, 6), ((4, 2, 6), 750, 5)]:
        bpy.ops.object.light_add(type='AREA', location=location)
        lamp = bpy.context.object
        lamp.data.energy = energy
        lamp.data.shape = 'DISK'
        lamp.data.size = size
        lamp.rotation_euler = (Vector((0, 0, 2)) - lamp.location).to_track_quat('-Z', 'Y').to_euler()
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.samples = 24
    scene.render.resolution_x = 800
    scene.render.resolution_y = 800
    scene.render.resolution_percentage = 100
    scene.world.color = (.18, .18, .18)
    scene.render.filepath = str(out / 'BLENDER-preview.png')
    bpy.ops.wm.save_as_mainfile(filepath=str(out / 'archway.blend'))
    bpy.ops.render.render(write_still=True)
    print('ORIGINAL_ARCH_GENERATED', json.dumps(geometry))


if __name__ == '__main__':
    main()
