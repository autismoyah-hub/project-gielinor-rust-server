"""Synthetic checks for fixed-camera diagnostics; no game assets or fixtures."""
import unittest,tempfile,json
from pathlib import Path
import numpy as np
from PIL import Image,ImageDraw
from evaluate import compare,diagnostic,VIEWS
from landmarks import camera,project
from geometry_audit import hull2,area,topology

class Diagnostics(unittest.TestCase):
    def test_translation_is_not_aligned_away(self):
        with tempfile.TemporaryDirectory(dir=Path.cwd())as root:
            root=Path(root)
            for name,offset in [('ref',0),('same',0),('shift',8)]:
                folder=root/name;folder.mkdir()
                for view in VIEWS:
                    im=Image.new('RGB',(96,80),(16,21,26));ImageDraw.Draw(im).rectangle((20+offset,20,60+offset,60),fill=(110,145,80));im.save(folder/(view+'.png'))
            baseline=compare(root/'ref',root/'same');self.assertEqual(baseline['mean_iou'],1)
            shifted=compare(root/'ref',root/'shift',baseline);self.assertLess(shifted['mean_iou'],.8);self.assertEqual(set(shifted['regressions']),set(VIEWS))
    def test_camera_contract(self):
        cam=camera('look 0 0 -200 0 0 0 1000 1000 900');self.assertEqual(project([0,0,0],cam),[500,450]);self.assertEqual(project([10,0,0],cam),[550,450])
        cam=camera('player 0 0 0 0 0 256');self.assertEqual(cam[0].tolist(),[0,0,-320]);self.assertEqual(project([0,0,0],cam),[382,251])
        cam=camera('player 0 -70 0 2000 14500 700');np.testing.assert_allclose(cam[1]@cam[1].T,np.eye(3),atol=1e-12)
    def test_projection_preserves_scale(self):
        cam=camera('look 0 0 -200 0 0 0 1000 1000 900');a=project([10,0,0],cam);b=project([20,0,0],cam);self.assertEqual(b[0]-500,2*(a[0]-500))
    def test_hull_and_topology(self):
        self.assertEqual(area(hull2([(0,0),(1,0),(1,1),(0,1),(.5,.5)])),1)
        g={'vertices':[[0,0,0],[1,0,0],[0,1,0],[0,0,1]],'faces':[[0,2,1],[0,1,3],[0,3,2],[1,2,3]]};a=topology(g);self.assertEqual(a['boundary_edges'],0);self.assertEqual(a['degenerate_triangles'],0)
    def test_size_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=Path.cwd())as root:
            a,b=Path(root)/'a.png',Path(root)/'b.png';Image.new('RGB',(10,10)).save(a);Image.new('RGB',(20,10)).save(b)
            with self.assertRaises(AssertionError):diagnostic(a,b)
if __name__=='__main__':unittest.main()
