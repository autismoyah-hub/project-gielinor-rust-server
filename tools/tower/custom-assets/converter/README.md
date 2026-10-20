# Static authoring tools

Selected original tools from server handoff
`f9180b23ae1e158a296489a3730a8ae697a71de2`, retained as standalone Python tools.
They do not change the game cache, mount resources, or run inside the server.

`convert_glb.py` converts a bounded static, opaque, untextured GLB triangle
subset to the legacy model format. Run `python convert_glb.py --help` for its
explicit input/output arguments. Use original assets you are entitled to export.
`calibration_codec.py` is the strict classic diagnostic encoder used by its tests.
Neither tool establishes native rendered acceptance or supports every model type.

Run the preserved synthetic tests from this directory:

```
python -B -m unittest discover -p "test_*.py" -v
```

The tests cover malformed GLB, smart limits, explicit quantization, invalid
triangles, opaque colors, deterministic output and refusal of unsupported skins,
weights and textures. No reference cache, private geometry or Blender is needed.

The historical package scripts are deliberately not adopted as current namespace
policy: their ranges predate the admitted Cinderworks package. Blender interchange,
extended model/animation tooling and native offline rendering remain preserved in
the audited recovery source and require their own dependency/acceptance review.
This selection is partial T-CORE adoption, not proof of full toolchain migration.
