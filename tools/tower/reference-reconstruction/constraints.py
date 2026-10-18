"""Named anatomical estimates and bounded camera calibration of existing art.

All image positions are manually estimated panel pixels, with uncertainty.
Camera fitting uses uniform perspective projection, never aspect-ratio warps.
"""
from pathlib import Path
import json,math,hashlib
import numpy as np
from PIL import Image,ImageDraw
T=Path(__file__).resolve().parent;R=T.parents[3]/'ashhorn-reference-private';D=R/'references'
P={
 'nose':[0,-140,112],'eye_pos':[22,-118,128],'eye_neg':[-22,-118,128],
 'jaw_pos':[23,-107,102],'jaw_neg':[-23,-107,102],'ear_pos':[27,-91,138],'ear_neg':[-27,-91,138],
 'horn_root_pos':[18,-103,139],'horn_tip_pos':[33,-38,176],'horn_root_neg':[-22,-105,137],'horn_break_neg':[-28,-98,151],
 'shoulder_pos':[42,-45,111],'shoulder_neg':[-42,-45,111],'chest_bottom':[0,-42,52],'chest_pos':[34,-52,76],'chest_neg':[-34,-52,76],
 'withers':[0,-44,137],'spine_mid':[0,1,123],'spine_lumbar':[0,31,116],'hip_pos':[32,64,101],'hip_neg':[-32,64,101],
 'elbow_pos':[47,-34,65],'elbow_neg':[-47,-60,65],'wrist_pos':[53,-58,19],'wrist_neg':[-49,-86,19],
 'forepaw_pos':[54,-63,5],'forepaw_neg':[-50,-91,5],'knee_pos':[43,50,64],'knee_neg':[-41,36,65],
 'hock_pos':[43,111,31],'hock_neg':[-41,72,29],'hindpaw_pos':[44,97,5],'hindpaw_neg':[-40,45,5],
 'tail_root':[0,79,85],'tail_mid':[0,143,52],'tail_tip':[-30,190,107],
 'shoulder_plate_top_pos':[43,-49,140],'shoulder_plate_edge_pos':[64,-35,95],
 'shoulder_plate_top_neg':[-43,-49,140],'shoulder_plate_edge_neg':[-64,-35,95],
}
for i,(y,w,z) in enumerate([(-44,43,145),(-17,39,138),(10,33,128),(34,29,121),(58,27,119)]):
    P[f'armor_{i}_ridge']=[0,y,z];P[f'armor_{i}_pos']=[w,y+6,z-10];P[f'armor_{i}_neg']=[-w,y+6,z-10]
# Each tuple is u,v, estimated standard error in pixels. Occluded landmarks
# have low weight or are omitted; none is asserted exact.
observations={
 'front':{'nose':[271,174,7],'eye_neg':[240,140,5],'eye_pos':[300,140,5],'jaw_neg':[241,204,8],'jaw_pos':[303,204,8],
 'ear_neg':[220,123,10],'ear_pos':[324,119,10],'horn_root_pos':[284,111,13],'horn_tip_pos':[345,16,3],'horn_root_neg':[221,107,10],'horn_break_neg':[218,73,5],
 'shoulder_neg':[166,221,13],'shoulder_pos':[383,221,13],'chest_bottom':[273,354,8],
 'elbow_neg':[153,326,12],'elbow_pos':[400,326,12],'wrist_neg':[133,439,8],'wrist_pos':[418,439,8],
 'forepaw_neg':[132,481,5],'forepaw_pos':[415,481,5],'hindpaw_neg':[208,468,15],'hindpaw_pos':[334,468,15],
 'shoulder_plate_top_neg':[166,164,12],'shoulder_plate_edge_neg':[114,232,10],'shoulder_plate_top_pos':[383,164,12],'shoulder_plate_edge_pos':[433,232,10]},
 'left':{'nose':[42,202,6],'eye_pos':[111,174,5],'jaw_pos':[173,235,8],'ear_pos':[178,147,9],
 'horn_root_pos':[158,119,10],'horn_tip_pos':[314,31,3],'horn_break_neg':[87,143,9],
 'withers':[297,145,10],'spine_mid':[421,227,15],'spine_lumbar':[530,264,14],'hip_pos':[625,278,14],
 'shoulder_pos':[326,238,14],'chest_bottom':[253,344,10],'elbow_pos':[336,326,10],'elbow_neg':[235,331,17],
 'wrist_pos':[295,450,8],'wrist_neg':[167,450,14],'forepaw_pos':[274,480,6],'forepaw_neg':[150,477,10],
 'knee_pos':[640,343,13],'knee_neg':[523,367,18],'hock_pos':[704,394,10],'hock_neg':[586,411,14],'hindpaw_pos':[711,478,6],'hindpaw_neg':[530,478,10],
 'tail_root':[686,284,15],'tail_mid':[820,323,16],'tail_tip':[915,203,7],
 'shoulder_plate_top_pos':[300,119,9],'shoulder_plate_edge_pos':[364,235,10],
 'armor_0_ridge':[310,111,12],'armor_1_ridge':[385,138,14],'armor_2_ridge':[449,186,14],'armor_3_ridge':[514,222,14],'armor_4_ridge':[577,226,14]},
 'rear':{'horn_tip_pos':[201,16,6],'horn_break_neg':[351,48,9],'hip_neg':[371,249,18],'hip_pos':[171,249,18],
 'hock_pos':[194,379,15],'hock_neg':[352,379,15],'hindpaw_pos':[201,459,8],'hindpaw_neg':[351,459,8],
 'forepaw_pos':[119,430,14],'forepaw_neg':[449,430,14],'tail_root':[273,233,20],'tail_mid':[284,374,20],'tail_tip':[358,288,25],
 'armor_0_ridge':[275,74,10],'armor_1_ridge':[275,118,10],'armor_2_ridge':[275,159,10],'armor_3_ridge':[275,192,10],'armor_4_ridge':[275,222,10]},
 'quarter':{'nose':[140,221,12],'eye_pos':[192,165,8],'horn_tip_pos':[360,15,10],'horn_break_neg':[181,112,10],
 'forepaw_pos':[350,438,14],'forepaw_neg':[199,417,20],'hindpaw_pos':[782,459,15],'hindpaw_neg':[606,424,18],'tail_tip':[923,180,16]}}
sizes={'front':[584,512],'left':[952,512],'rear':[584,512],'quarter':[952,512]}
def camera(params):
    yaw,pitch,d,f,ox,oy=params;ya,pi=map(math.radians,[yaw,pitch]);target=np.array([0,0,84.]);eye=target+np.array([math.sin(ya)*d,-math.cos(ya)*d,math.tan(pi)*d]);forward=(target-eye);forward/=np.linalg.norm(forward);right=np.cross(forward,[0,0,1]);right/=np.linalg.norm(right);up=np.cross(right,forward)
    return eye,np.array([right,up,forward]),f,np.array([ox,oy])
def project(points,params,size):
    eye,basis,f,offset=camera(params);q=(np.asarray(points)-eye)@basis.T
    return np.column_stack([size[0]/2+offset[0]+q[:,0]*f/q[:,2],size[1]/2+offset[1]-q[:,1]*f/q[:,2]])
def fit(view,prior,bounds):
    names=list(observations[view]);world=np.array([P[k]for k in names]);target=np.array([observations[view][k][:2]for k in names]);uncert=np.array([observations[view][k][2]for k in names])
    initial=np.array(prior,float);p=initial.copy();lo,hi=np.array(bounds).T;scales=np.array([20,14,300,900,150,100]);damping=.01
    def residual(x):return np.concatenate([((project(world,x,sizes[view])-target)/uncert[:,None]).ravel(),(x-initial)/scales*.35])
    for _ in range(100):
        r=residual(p);J=np.empty((len(r),len(p)))
        for j in range(len(p)):
            step=[.01,.01,.2,.2,.02,.02][j];q=p.copy();q[j]+=step;J[:,j]=(residual(q)-r)/step
        A=J.T@J;change=np.linalg.solve(A+np.diag(np.maximum(np.diag(A),1e-5))*damping,-J.T@r);trial=np.clip(p+change,lo,hi)
        if np.dot(residual(trial),residual(trial))<np.dot(r,r):p=trial;damping=max(1e-6,damping*.6)
        else:damping=min(1e6,damping*4)
        if np.linalg.norm(change/scales)<1e-6:break
    predicted=project(world,p,sizes[view]);errors=np.linalg.norm(predicted-target,axis=1)
    eye,basis,f,offset=camera(p);target_world=np.array([0.,0,84.]);native=lambda a:[float(a[0]),float(-a[2]),float(a[1])]
    return {'parameters':p.tolist(),'eye_author':eye.tolist(),'target_author':target_world.tolist(),'basis_author':basis.tolist(),'focal':float(f),'offset_pixels':offset.tolist(),'viewport':sizes[view],'mean_landmark_error_pixels':float(errors.mean()),'median_landmark_error_pixels':float(np.median(errors)),'landmark_residuals':{k:{'observed':target2.tolist(),'projected':pred.tolist(),'error_pixels':float(err),'uncertainty_pixels':observations[view][k][2]}for k,target2,pred,err in zip(names,target,predicted,errors)},'native_camera':'look '+' '.join(map(str,native(eye)+native(np.array([0,0,84]))+[float(f),*sizes[view],*offset.tolist()]))}
def triangulate(cameras,initial):
    """Fit meaningful anatomical landmarks, with pinned ground and body priors."""
    for name,q in initial.items():
        q=np.array(q,float);prior=q.copy();views=[v for v in cameras if name in observations[v]]
        if not views:continue
        def constrain(x):
            x=x.copy()
            if 'paw_' in name:x[2]=5
            if name in ['chest_bottom','withers','spine_mid','spine_lumbar','tail_root'] or name.endswith('_ridge'):x[0]=0
            if name=='horn_tip_pos':x[2]=176
            if name.endswith('_pos'):x[0]=max(2,x[0])
            if name.endswith('_neg'):x[0]=min(-2,x[0])
            return np.clip(x,prior-[25,45,24],prior+[25,45,24])
        def residual(x):
            r=[]
            for v in views:
                obs=observations[v][name];r.extend((project([x],cameras[v]['parameters'],sizes[v])[0]-obs[:2])/obs[2])
            r.extend((x-prior)/[15,25,15]*.45);return np.array(r)
        for _ in range(30):
            r=residual(q);J=np.column_stack([(residual(q+np.eye(3)[i]*.05)-r)/.05 for i in range(3)]);change=np.linalg.solve(J.T@J+np.eye(3)*.002,-J.T@r);trial=constrain(q+change)
            if np.dot(residual(trial),residual(trial))>=np.dot(r,r):break
            q=trial
            if np.linalg.norm(change)<.001:break
        P[name]=q.tolist()
if __name__=='__main__':
    c={}
    for view,prior,yr in [('front',[0,0,500,1200,-19,8],[-5,5]),('left',[76,8,620,1735,-50,0],[60,100]),('rear',[180,14,600,1470,-19,20],[173,187]),('quarter',[65,10,580,1650,-55,10],[35,80])]:
        c[view]=fit(view,prior,[yr,[-6,30],[400,1400],[800,3000],[-220,220],[-130,130]])
    initial={k:list(v)for k,v in P.items()}
    for _ in range(2):
        triangulate(c,initial)
        for view in c:
            yr={'front':[-5,5],'left':[60,100],'rear':[173,187],'quarter':[35,80]}[view]
            c[view]=fit(view,c[view]['parameters'],[yr,[-6,30],[400,1400],[800,3000],[-220,220],[-130,130]])
    # Facial bilateral anatomy is a stronger prior than inconsistent painted
    # corner estimates. Keep the intended horn asymmetry and planted stance.
    P['nose'][0]=0
    for part in ['eye','jaw','ear']:
        a,b=np.array(P[part+'_pos']),np.array(P[part+'_neg']);width=(abs(a[0])+abs(b[0]))/2;mid=(a+b)/2
        P[part+'_pos']=[float(width),float(mid[1]),float(mid[2])];P[part+'_neg']=[float(-width),float(mid[1]),float(mid[2])]
    for view in c:
        yr={'front':[-5,5],'left':[60,100],'rear':[173,187],'quarter':[35,80]}[view]
        c[view]=fit(view,c[view]['parameters'],[yr,[-6,30],[400,1400],[800,3000],[-220,220],[-130,130]])
    # The painted outer knee bulge is not the anatomical joint center. Preserve
    # that observed contour and place the joint forward of the hip instead of
    # forcing an implausible skeleton to match a silhouette marker.
    for side in ['pos','neg']:
        key='knee_'+side;contour='knee_contour_'+side;P[contour]=list(P[key]);P[key]=[P[key][0],min(P[key][1],P['hip_'+side][1]-12),P[key][2]]
        for view in c:
            if key in observations[view]:observations[view][contour]=observations[view].pop(key)
            if key in c[view]['landmark_residuals']:c[view]['landmark_residuals'][contour]=c[view]['landmark_residuals'].pop(key)
    data={'coordinate_contract':'Native authoring units128/tile; X viewer-right in front (anatomical left), Y rearward, Z up; ground0; origin under torso. Drawn laterality overrides superseded numeric specification.','landmarks':P,'initial_anatomical_estimates':initial,'observations_2d':observations,'uncertainty':'Hand-estimated, not automatic ground truth. Two bounded camera/landmark adjustment cycles with body/ground priors. Pixel uncertainty per observation; positions of hidden far limbs, tail and plate boundaries are especially uncertain.','camera_assumptions':'Uniform perspective cameras fitted to named framework estimates and then frozen before mesh construction; no independent image stretching. Orthographic working cameras remain available. Each reference illustration may contain incompatible pose/expression/artistic perspective.','cameras':c,'framework':{'ribcage_center':[0,-22,98],'ribcage_radii':[46,67,39],'pelvis_center':[0,61,88],'pelvis_radii':[34,31,26],'neck_axis':[[0,-54,103],[0,-102,121]],'forelimbs':[['shoulder_pos','elbow_pos','wrist_pos','forepaw_pos'],['shoulder_neg','elbow_neg','wrist_neg','forepaw_neg']],'hindlimbs':[['hip_pos','knee_pos','hock_pos','hindpaw_pos'],['hip_neg','knee_neg','hock_neg','hindpaw_neg']],'tail_centerline':['tail_root','tail_mid','tail_tip']}}
    (R/'constraints.json').write_text(json.dumps(data,indent=2)+'\n')
    raw=Image.open(R/'baseline/original-art/turnaround-v2.png').convert('RGB');prepared=json.loads((D/'preparation.json').read_text())
    for view in c:
        panel=raw.crop(prepared['panels'][view]['original_panel']);draw=ImageDraw.Draw(panel)
        for i,(name,row) in enumerate(c[view]['landmark_residuals'].items()):
            a=row['observed'];b=row['projected'];draw.line([tuple(a),tuple(b)],fill='#e2a64e',width=1);draw.ellipse([a[0]-2,a[1]-2,a[0]+2,a[1]+2],fill='#2ee0c0');draw.ellipse([b[0]-2,b[1]-2,b[0]+2,b[1]+2],fill='#ff6262');draw.text((a[0]+3,a[1]-8),str(i+1),fill='#20ffd5')
        panel.save(D/(view+'-landmarks.png'))
        print(view,'mean residual',round(c[view]['mean_landmark_error_pixels'],1),'px; camera',np.round(c[view]['parameters'],1),flush=True)
