"""Native full-duration playback, pose proof and actual motion previews."""
from review import *
import struct,math
sys.path.insert(0,str(HERE.parent/'asset-workbench'))
from kinematics import trace,frame_at
folder=ROOT/'runs/creature-final';g=json.loads((folder/'geometry-rigged.json').read_text());report=json.loads((folder/'motion-validation.json').read_text())if(folder/'motion-validation.json').exists()else{}
run([NATIVE,CACHE,folder/'motion-camera','render-own',folder/'asset.model',64,768],folder/'motion-camera/log.txt')
for name in (sys.argv[1:]or ['idle','walk','attack','hit','death']):
    target=folder/'motion'/name;run([NATIVE,CACHE,target,'play-complete',folder/'asset.model',folder/'motion-camera/camera.txt',64,768,folder/name],target/'log.txt')
    data=json.loads((folder/name/'animation.json').read_text());duration=sum(f['length']for f in data['frames']);worst=0;poses=[]
    for p in sorted(target.glob('pose-*.json')):
        tick=int(p.stem.split('-')[1]);_,frame=frame_at(data['frames'],tick%duration if data['policy'].get('loop_frames',-1)>0 else tick);predicted,_=trace(g,frame);actual=json.loads(p.read_text())['vertices'];f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
        assert len(predicted)==len(actual);worst=max(worst,max(abs(f32(a)-f32(b))for v,w in zip(predicted,actual)for a,b in zip(v,w)));poses.append((tick,actual))
    assert worst==0;assert len(poses)==duration+1
    # High resolution previews use the unchanged scene renderer and authored
    # key cadence (2 ticks); duration accounts for every complete action tick.
    scene=folder/('motion-'+name+'.tsv');scene.write_text('light\t64\t768\nowned\tasset.model\t0\t0\t0\t0\t'+name+'\t0\n');frames=[]
    for tick in range(0,duration,2):
        dest=folder/'motion-close'/name/str(tick);run([NATIVE,CACHE,dest,'scene',scene,folder/'quarter.camera',tick],dest/'log.txt');frames.append(Image.open(dest/'scene.bmp').convert('RGB'))
    frames[0].save(folder/(name+'.gif'),save_all=True,append_images=frames[1:],duration=40,loop=0 if name in ['idle','walk']else 1)
    # Representative samples show recovery/hold rather than just a good pose.
    chosen=[0,duration//4,duration//2,3*duration//4,max(0,duration-2)]
    sheet=Image.new('RGB',(500*len(chosen),400),(16,23,27))
    for i,tick in enumerate(chosen):sheet.paste(frames[min(len(frames)-1,tick//2)].resize((500,400)),(i*500,0))
    sheet.save(folder/(name+'-sheet.png'))
    report[name]=dict(duration_ticks=duration,native_pose_samples=len(poses),max_coordinate_error=worst,gif_ms=duration*20)
    print('ACTION_VERIFIED',name,len(poses),flush=True)
(folder/'motion-validation.json').write_text(json.dumps(report,indent=2))
