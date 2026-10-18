"""Authored rig ownership, native-angle limits and explicit attachment checks."""
import math

def validate(recipe,channels=None):
    rig=recipe.get('rig',[]);by={};owned=set();reports=[]
    for j in rig:
        name=j['name'];parent=j.get('parent');groups=set(j['groups'])
        if name in by or(parent is not None and parent not in by):raise ValueError('Rig must be unique, parent-first and acyclic')
        if not groups or any(not isinstance(g,int)or not 0<=g<=255 for g in groups):raise ValueError('Invalid native group ownership')
        if parent and not groups.issubset(by[parent]['groups']):raise ValueError('Child group escapes parent')
        if any(not math.isfinite(x)for x in j['origin']):raise ValueError('Non-finite authored pivot')
        by[name]=j;owned|=groups
        if channels is not None:
            angles=channels.get(name,[0,0,0]);limits=j.get('limits',[[-128,127]]*3)
            for axis,a in enumerate(angles):
                if not limits[axis][0]<=a<=limits[axis][1]:reports.append(dict(joint=name,axis=axis,angle=a,limits=limits[axis],finding='Joint exceeds explicit authored range'))
    if rig:
        for c in recipe['components']:
            if c.get('group',0)not in owned:raise ValueError('Component has no rig owner: '+c['name'])
    for a in recipe.get('attachments',[]):
        if a['joint']not in by:raise ValueError('Attachment has no joint owner')
        components={p['name']:p for p in recipe['components']}
        if any(n not in components for n in a['components']):raise ValueError('Attachment names missing geometry')
        if any(components[n].get('group',0)not in by[a['joint']]['groups']for n in a['components']):raise ValueError('Equipment does not follow declared attachment joint')
    return dict(joints=len(rig),groups=len(owned),limit_violations=reports)
