"""Meaningful numeric, boundary and ownership tests; native tests are separate."""
import unittest,math,tempfile,json,os
from contextlib import contextmanager
from pathlib import Path
from native_codec import model,classic
from kinematics import trace,compile_joints,two_bone_yz,rotate
from primitives import socket,beam
from diagnose import principal_axes
from registry import object_package,IdentityAllocator
from analyze import surface_geometry,geometry as geometry_report
@contextmanager
def owned_temporary():
    root=Path(os.environ.get('WORKBENCH_TEST_TEMP',str(Path.cwd()/'.workbench-test-tmp'))).resolve();root.mkdir(parents=True,exist_ok=True);tmp=tempfile.TemporaryDirectory(dir=root);target=Path(tmp.name).resolve()
    if target==root or not target.is_relative_to(root):raise RuntimeError('Temporary cleanup target outside explicit test root')
    try:yield target
    finally:tmp.cleanup()
class Contracts(unittest.TestCase):
    def geometry(self):return dict(vertices=[[0,0,0],[8,0,0],[0,8,0]],faces=[[0,1,2]],colors=[43051],priority=0,vertex_skins=[1,1,1])
    def test_fractional_geometry_is_not_silently_rounded(self):
        g=self.geometry();g['vertices'][0][0]=.5
        with self.assertRaises(ValueError):model(g)
    def test_complex_texture_data_rejected(self):
        g=self.geometry();g.update(texture_types=[1],texture_triangles=[[0,1,2]])
        with self.assertRaises(ValueError):model(g)
    def test_weighted_rig_rejected(self):
        g=self.geometry();g['bone_weights']=[[[0,255]]]*3
        with self.assertRaises(ValueError):model(g)
    def test_native_group_order_is_explicit(self):
        with owned_temporary()as tmp:
            data=dict(skeleton_id=64500,frame_group=62000,frames=[dict(types=[0,2],labels=[[1],[1]],length=3,transforms=[[1,[0,0,0]],[0,[0,0,0]]])])
            with self.assertRaises(ValueError):classic(data,tmp)
    def test_socket_preserves_winding(self):
        with self.assertRaises(ValueError):socket([[1,2,3]],dict(basis=[[-1,0,0],[0,1,0],[0,0,1]]))
        self.assertEqual(socket([[1,2,3]],dict(basis=[[0,0,1],[0,1,0],[-1,0,0]])),[[-3,2,1]])
    def test_two_bone_contact_and_unreachable_flag(self):
        answer=two_bone_yz([0,0,0],[0,50,20],[35,30]);self.assertTrue(answer['reachable']);knee=rotate([0,35,0],[answer['hip_native'],0,0],[0,0,0]);tip=rotate([0,65,0],[answer['knee_native'],0,0],[0,35,0]);tip=rotate(tip,[answer['hip_native'],0,0],[0,0,0]);self.assertLess(math.dist(tip,[0,50,20]),3)
        self.assertFalse(two_bone_yz([0,0,0],[0,1000,0],[35,30])['reachable'])
    def test_joint_compiler_does_not_use_dynamic_centroid_as_joint(self):
        g=self.geometry();rig=[dict(name='joint',parent=None,origin=[20,0,0],groups=[1])];f,expected,_=compile_joints(g,rig,dict(joint=[0,0,32]));actual,records=trace(g,f);self.assertEqual(actual,expected);self.assertIn([20,0,0],[r['pivot']for r in records])
    def test_palette_is_packed_integer_not_rgb_roundtrip(self):
        g=self.geometry();blob=model(g);self.assertIn(bytes.fromhex('a82b'),blob)
    def test_original_identifier_collision_rejected(self):
        with owned_temporary()as tmp:
            with self.assertRaises(ValueError):object_package(tmp,[self.geometry()],{},[42],63300,dict(model_max=61605,object_max=62200))
    def test_plane_orientation_not_misreported_as_depth(self):
        eig=principal_axes([[0,0,0],[0,10,0],[0,0,10],[0,10,10]])
        self.assertLess(eig[0]['variance'],1e-6);self.assertGreater(abs(eig[0]['axis'][0]),.999)
    def test_hundreds_of_static_assets_do_not_exhaust_animation_space(self):
        with owned_temporary()as tmp:
            allocator=IdentityAllocator(Path(tmp)/'identities.json',dict(model_max=61605,object_max=62200,npc_max=16300,sequence_max=14428,skeleton_max=2673,frame_group_max=14521));rows=[allocator.allocate('static:'+str(i),False)for i in range(200)];actor=allocator.allocate('actor',True);self.assertEqual(actor['skeleton_base'],32000);self.assertEqual(len({v['model_id']for v in rows}),200);self.assertEqual(allocator.allocate('static:0',False),rows[0])
    def test_parent_ownership_and_joint_order_are_validated(self):
        with self.assertRaises(ValueError):compile_joints(self.geometry(),[dict(name='child',parent='missing',origin=[0]*3,groups=[1])],{})
    def test_invisible_helper_vertices_do_not_inflate_surface_descriptors(self):
        g=self.geometry();g['vertices'] += [[900,900,900],[910,900,900],[900,910,900]];g['faces'] += [[3,4,5]];g['colors'] += [43051];g['vertex_skins'] += [0,0,0];g['alpha']=[0,255]
        selected=surface_geometry(g,[0]);self.assertEqual(geometry_report(selected)['exact_decoded']['bounds'],[[0,0,0],[8,8,0]]);self.assertEqual(len(g['vertices']),6)
    def test_static_identity_does_not_silently_acquire_actor_ids(self):
        with owned_temporary()as tmp:
            allocator=IdentityAllocator(Path(tmp)/'ids.json',{});allocator.allocate('prop',False)
            with self.assertRaises(ValueError):allocator.allocate('prop',True)
    def test_reused_identity_is_rechecked_against_source_bounds(self):
        with owned_temporary()as tmp:
            path=Path(tmp)/'ids.json';allocator=IdentityAllocator(path,dict(model_max=61605,object_max=62200));allocator.allocate('prop',False);data=json.loads(path.read_text());data['assets']['prop']['model_id']=42;path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):IdentityAllocator(path,dict(model_max=61605,object_max=62200)).allocate('prop',False)
if __name__=='__main__':unittest.main()
