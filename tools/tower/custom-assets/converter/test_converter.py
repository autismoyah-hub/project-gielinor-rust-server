import importlib.util
from pathlib import Path
import tempfile
import unittest
import struct
import json

spec=importlib.util.spec_from_file_location('convert',Path(__file__).with_name('convert_glb.py'))
c=importlib.util.module_from_spec(spec); spec.loader.exec_module(c)

class CodecTests(unittest.TestCase):
    def test_smart_boundaries(self):
        for value in [-16384,-65,-64,-1,0,63,64,16383]:
            encoded=c.smart(value)
            actual=encoded[0]-64 if len(encoded)==1 else struct.unpack('>H',encoded)[0]-49152
            self.assertEqual(actual,value)
        for value in [-16385,16384]:
            with self.assertRaises(ValueError): c.smart(value)

    def test_reject_empty_and_section_overflow(self):
        with self.assertRaises(ValueError): c.encode([],[],[])
        with self.assertRaises(ValueError): c.encode([(0,0,0),(20000,0,0)],[(0,1,0)],[100])

    def test_opaque_palette(self):
        for rgba in [[0,0,0,0],[float('nan'),0,0,1],[1.1,0,0,1]]:
            with self.assertRaises(ValueError): c.palette(rgba)
        self.assertTrue(0<=c.palette([.4,.3,.2,1])<=65535)

    def test_malformed_glb(self):
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'bad.glb'
            path.write_bytes(struct.pack('<4sII',b'glTF',2,100))
            with self.assertRaises(ValueError): c.convert(path)

if __name__=='__main__': unittest.main()
