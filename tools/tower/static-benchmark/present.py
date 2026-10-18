from run import *
selected={'anvil':'final2','workstation':'final2','machine':'final','creature':'final2'}
for kind,ver in selected.items():
    if sys.argv[1:]and kind not in sys.argv[1:]:continue
    source=ROOT/'runs'/(kind+'-'+ver)
    for mode in ['beauty','native-match']+(['clay','topology','front','side','rear']if kind=='creature'else[]):
        run([BLENDER,'-b',source/'source.blend','--python-exit-code','1','--python',HERE/'studio.py','--',ROOT/'presentation'/kind,mode,*([source/'quarter.camera']if mode=='native-match'else[])],ROOT/'logs'/('studio-'+kind+'-'+mode+'.log'));print('STUDIO',kind,mode,flush=True)
