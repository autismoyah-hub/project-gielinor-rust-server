"""Independent face-alpha expectation for full classic development audits."""
def classic_alpha(g,frame):
    alpha=list(g['alpha'])if g.get('alpha')is not None else None
    if g.get('vertex_skins')is None:return alpha
    transforms=[(label,value)for label,value in frame['transforms']if frame['types'][label]==5]
    if transforms and alpha is None:alpha=[0]*len(g['faces'])
    if g.get('face_skins')is not None and alpha is not None:
        for label,value in transforms:
            for group in frame['labels'][label]:
                for i,skin in enumerate(g['face_skins']):
                    if skin==group:alpha[i]=max(0,min(255,alpha[i]+value[0]*8))
    return alpha
