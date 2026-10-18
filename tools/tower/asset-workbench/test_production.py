"""Construction, native-control and diagnostic invariants for production tools."""
import unittest,math
from collections import Counter
from shape_families import sweep,stave,chamfer_block
from semantic_materials import material,colors
from motion_authoring import solve_limb,forward,ancestors,matrix,matmul,inverse_angles
from contact_diagnostics import hull,inside
from kinematics import frame_at
from construction import clipped
from mesh_validation import clean_owned

class ProductionContracts(unittest.TestCase):
    def test_owned_cleanup_preserves_vertex_identity_for_classic_groups(self):
        vs=[[0,0,0],[8,0,0],[0,8,0],[4,0,0]];out,faces,report=clean_owned(vs,[[0,1,2],[0,1,3]]);self.assertIs(out,vs);self.assertEqual(faces,[[0,1,2]]);self.assertEqual(report['removed_zero_area_faces'],1)
    def test_entirely_collapsed_owned_component_is_rejected(self):
        with self.assertRaises(ValueError):clean_owned([[0,0,0],[1,0,0],[2,0,0]],[[0,1,2]])
    def test_closed_separate_stave_is_a_manifold_shell(self):
        v,f=stave([[0,28,28],[-40,35,35],[-80,28,28]],.02,.6,4);edges=Counter(tuple(sorted((face[i],face[(i+1)%3])))for face in f for i in range(3));self.assertTrue(all(n==2 for n in edges.values()));self.assertEqual(len(v)-len(edges)+len(f),2)
    def test_curved_sweep_rejects_zero_tangent_and_negative_radius(self):
        with self.assertRaises(ValueError):sweep([[0,0,0,1,1],[0,0,0,1,1]])
        with self.assertRaises(ValueError):sweep([[0,0,0,-1,1],[0,10,0,1,1]])
    def test_wood_end_grain_assignment_survives_winding_reversal(self):
        vs=[[0,0,0],[10,0,0],[0,0,10]];m=material('wood');self.assertEqual(colors(vs,[[0,1,2]],m,{}),colors(vs,[[2,1,0]],m,{}));self.assertEqual(colors(vs,[[0,1,2]],m,{})[0],m['end'])
    def test_semantic_family_does_not_modify_legacy_material(self):
        self.assertEqual(colors([[0,0,0],[1,0,0],[0,1,0]],[[0,1,2]],dict(hsl=43051),{}),[43051])
    def test_clipped_board_stays_inside_its_authored_interval(self):
        p=clipped(clipped([[-10,-10],[10,-10],[6,10],[-6,10]],0,-3,True),0,3,False);self.assertTrue(all(-3-1e-8<=v[0]<=3+1e-8 for v in p));self.assertGreater(len(p),2)
    def test_analytic_limb_seed_reaches_native_quantized_target(self):
        hip=[-24,-35,0];knee=[-49,-23,7];tip=[-61,-3,13];rig=[dict(name='body',parent=None,origin=[0,-35,0],groups=[0,1,2]),dict(name='leg_hip',parent='body',origin=hip,groups=[1,2]),dict(name='leg_knee',parent='leg_hip',origin=knee,groups=[2])];limb=dict(name='leg',hip=hip,knee=knee,tip=tip);c,e=solve_limb(rig,{},limb,[-61,-3,0],[0,3,0]);self.assertLessEqual(e,2)
    def test_ankle_cancellation_matches_world_axis_rotation_order(self):
        chain=[dict(name='root'),dict(name='hip'),dict(name='knee')];c=dict(root=[4,3,2],hip=[-11,8,5],knee=[2,-7,13]);a=inverse_angles(chain,c);p=[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]]
        for j in chain:p=matmul(matrix(c[j['name']]),p)
        q=matmul(matrix(a),p);self.assertLess(max(abs(q[i][j]-(1 if i==j else 0))for i in range(3)for j in range(3)),.04)
    def test_support_polygon_requires_a_real_support_area(self):
        self.assertFalse(inside([0,0],hull([[-1,0],[1,0]])));self.assertTrue(inside([0,0],hull([[-2,-2],[2,-2],[0,2]])));self.assertFalse(inside([3,3],hull([[-2,-2],[2,-2],[0,2]])))
    def test_native_contact_phase_keeps_boundary_frame(self):
        frames=[dict(length=3),dict(length=3)];self.assertEqual(frame_at(frames,3)[0],0);self.assertEqual(frame_at(frames,4)[0],1)
if __name__=='__main__':unittest.main()
