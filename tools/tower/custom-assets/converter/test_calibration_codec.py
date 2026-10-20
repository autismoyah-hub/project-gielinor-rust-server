"""Failure boundaries for the new classic diagnostic encoder."""
import unittest,copy
from calibration_codec import encode_model
class Contracts(unittest.TestCase):
    def setUp(self):self.g=dict(vertices=[[0,0,0],[2,0,0],[0,2,0]],faces=[[0,1,2]],colors=[43051],priority=0)
    def test_quantization_is_explicit(self):
        self.g['vertices'][0][0]=.25
        with self.assertRaises(ValueError):encode_model(self.g)
    def test_skin_labels_are_bounded(self):
        self.g['vertex_skins']=[0,1,256]
        with self.assertRaises(ValueError):encode_model(self.g)
    def test_unsupported_weighted_skeleton_is_rejected(self):
        self.g['bone_weights']=[[[0,255]]]*3
        with self.assertRaises(ValueError):encode_model(self.g)
    def test_textured_payload_cannot_silently_lose_texture(self):
        self.g['face_textures']=[23]
        with self.assertRaises(ValueError):encode_model(self.g)
    def test_invalid_triangle_is_rejected(self):
        self.g['faces']=[[0,1,3]]
        with self.assertRaises(ValueError):encode_model(self.g)
    def test_native_smart_delta_cannot_overflow(self):
        self.g['vertices'][1][0]=20000
        with self.assertRaises(ValueError):encode_model(self.g)
    def test_deterministic_output(self):self.assertEqual(encode_model(self.g),encode_model(copy.deepcopy(self.g)))
if __name__=='__main__':unittest.main()
