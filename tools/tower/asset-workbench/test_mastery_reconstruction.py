import unittest
from section_reconstruction import observe,component
from reconstruction_regions import propose
from primitives import make
from advanced_geometry import topology
from owned_textures import apply
from contact_diagnostics import inside,hull

class ReconstructionContracts(unittest.TestCase):
    def test_coarse_landmarks_do_not_reuse_source_topology(self):
        g=dict(vertices=[[-4,-20,-3],[4,-20,-3],[0,0,6],[0,-20,3]],faces=[[0,1,2],[1,3,2]],colors=[33,33]);d=observe(g,{},1,3);v,f=make(component('study',d,sides=8));self.assertFalse(d['topology_reused']);self.assertNotEqual(v,g['vertices']);self.assertEqual(topology(v,f)['boundary_edges'],0)
    def test_invisible_helper_region_is_not_made_visible(self):
        g=dict(vertices=[[0,0,0],[10,0,0],[0,-10,0]],faces=[[0,1,2]],colors=[33],alpha=[255]);self.assertEqual(propose(g),[])
    def test_authored_projection_preserves_skin_attribute_cardinality(self):
        g=dict(vertices=[[0,0,0],[10,0,0],[0,-10,0]],faces=[[0,1,2]],colors=[33],vertex_skins=[7,7,7]);apply(g,{'native_texture_id':16},{'native_uv_projection':{'axes':[0,1]}});self.assertEqual(len(g['vertices']),len(g['vertex_skins']));self.assertEqual(g['texture_coords'],[0]);self.assertEqual(g['texture_triangles'],[[3,4,5]])
    def test_single_foot_has_area_with_sole_vertices(self):
        self.assertFalse(inside([0,0],hull([[0,0]])));self.assertTrue(inside([0,0],hull([[-5,-8],[5,-8],[5,8],[-5,8]])))

if __name__=='__main__':unittest.main()
