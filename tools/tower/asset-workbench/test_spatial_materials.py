import unittest
from spatial_materials import paint
class NativeBrushContracts(unittest.TestCase):
    def test_local_color_edit_preserves_other_native_faces(self):
        v=[[0,0,0],[6,0,0],[0,-6,0],[60,0,0],[66,0,0],[60,-6,0]];f=[[0,1,2],[3,4,5]];self.assertEqual(paint(v,f,[33,47],[dict(center=[2,-2,0],radius=8,hsl=5062)]),[5062,47])
    def test_invalid_palette_value_fails_before_blender_export(self):
        with self.assertRaises(ValueError):paint([[0,0,0]]*3,[[0,1,2]],[33],[dict(center=[0,0,0],radius=8,hsl=65536)])
if __name__=='__main__':unittest.main()
