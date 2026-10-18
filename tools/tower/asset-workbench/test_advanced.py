import unittest
from advanced_geometry import sections,shell,gear,cage,topology
from rig_contracts import validate
from owned_textures import merge,apply
from advanced_motion import segment_distance
class AdvancedContracts(unittest.TestCase):
    def test_owned_texture_map_offsets_across_components(self):
        t=dict(texture_types=[0],texture_triangles=[[0,1,2]],face_textures=[16],texture_coords=[0]);p=dict(vertices=[[0,0,0]]*3,faces=[[0,1,2]],colors=[10]);apply(p,{'native_texture_id':16},{'native_texture_triangles':[[0,1,2]],'native_texture_coords':[0]});merge(t,p,3,1);self.assertEqual(t['texture_triangles'][1],[3,4,5]);self.assertEqual(t['texture_coords'],[0,1])
    def test_untextured_owned_material_stays_unchanged(self):
        p=dict(vertices=[],faces=[],colors=[]);self.assertIs(apply(p,{'hsl':14},{}),p);self.assertNotIn('face_textures',p)
    def test_collision_proxy_crossing_and_parallel(self):
        self.assertEqual(segment_distance([0,0,0],[10,0,0],[5,-5,0],[5,5,0]),0);self.assertEqual(segment_distance([0,0,0],[10,0,0],[0,5,0],[10,5,0]),5)
    def test_section_correspondence_and_closed_topology(self):
        v,f=sections([[0,0,0,15,12],[2,-20,-1,19,10],[0,-35,0,8,7]]);self.assertEqual(topology(v,f)['boundary_edges'],0);self.assertEqual(topology(v,f)['inconsistently_wound_edges'],0)
    def test_different_ring_vertex_counts_rejected(self):
        with self.assertRaises(ValueError):sections([[0,0,0,8,8],[0,-9,0,8,8]],profiles=[[[1,1]]]*2)
    def test_fitted_shell_is_closed_at_open_edges(self):
        import math
        v,f=shell([[0,0,0,22,15],[0,-28,0,25,17]],4,8,0,math.pi);t=topology(v,f);self.assertEqual(t['boundary_edges'],0);self.assertEqual(t['nonmanifold_edges'],0);self.assertEqual(t['inconsistently_wound_edges'],0)
    def test_hollow_gear_is_closed(self):
        v,f=gear([0,0,0],40,33,8,8,10);self.assertEqual(topology(v,f)['boundary_edges'],0);self.assertEqual(topology(v,f)['inconsistently_wound_edges'],0)
    def test_cage_keeps_vertex_order(self):
        v=[[0,0,0],[10,10,10]];r=cage(v,dict(bounds=[[0,0,0],[10,10,10]],offsets=[[3,0,0]]*8));self.assertEqual(r,[[3,0,0],[13,10,10]])
    def test_inverted_cage_fails_before_native_export(self):
        with self.assertRaises(ValueError):cage([[0,0,0]],dict(bounds=[[0,0,0],[10,10,10]],offsets=[[-20 if i&1 else 0,0,0]for i in range(8)]))
    def test_attachment_owner_is_required(self):
        r=dict(rig=[dict(name='root',parent=None,origin=[0,0,0],groups=[0])],components=[dict(name='shield',group=1)])
        with self.assertRaises(ValueError):validate(r)
    def test_limits_are_reported_without_silent_clamp(self):
        r=dict(rig=[dict(name='root',parent=None,origin=[0,0,0],groups=[0],limits=[[-5,5]]*3)],components=[dict(name='body',group=0)]);self.assertEqual(len(validate(r,{'root':[6,0,0]})['limit_violations']),1)
if __name__=='__main__':unittest.main()
