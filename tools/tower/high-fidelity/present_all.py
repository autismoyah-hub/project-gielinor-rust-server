from review import *
jobs=[('creature','creature-final',mode)for mode in ['beauty','clay','topology','front','side','rear']]+[('architecture','architecture-revision1','beauty'),('machine','machine-revision1','beauty')]
for label,source,mode in jobs:
    run([BLENDER,'-b',ROOT/'runs'/source/'source.blend','--python-exit-code','1','--python',HERE/'presentation.py','--',ROOT/'presentation'/label,mode],ROOT/'logs'/('presentation-'+label+'-'+mode+'.log'));print('PRESENTATION',label,mode,flush=True)
run([BLENDER,'-b','--factory-startup','--python-exit-code','1','--python',HERE/'scene_blender.py','--',ROOT],ROOT/'logs/scene-blender.log')
run([BLENDER,'-b',ROOT/'scene/showcase.blend','--python-exit-code','1','--python',HERE/'presentation.py','--',ROOT/'presentation/scene','beauty'],ROOT/'logs/presentation-scene.log');print('PRESENTATION scene',flush=True)
