"""Replay a trusted local authoring-engine snapshot with its original recipe.

Run inside Blender: replay.py -- SNAPSHOT SOURCE_BLEND RECIPE OUTPUT_DIRECTORY.
The snapshot's dependency root stays pinned to this workbench, not its evidence
directory. This preserves older recipe semantics across tool improvements.
"""
import sys
from pathlib import Path
snapshot,source,recipe,out=sys.argv[sys.argv.index('--')+1:]
tool=Path(__file__).resolve().parent/'surface.py';sys.argv=['surface.py','--',source,recipe,out]
exec(compile(Path(snapshot).read_text(),str(snapshot),'exec'),{'__name__':'__main__','__file__':str(tool)})
