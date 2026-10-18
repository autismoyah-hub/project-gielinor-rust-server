"""Named geometric/material corrections from curated semantic descriptors.

This never treats overlap or a parameter proposal as artistic acceptance.
"""
import argparse,json
from pathlib import Path
from diagnose import diagnose
from refine import propose

AXES=['width (X)','height (Y)','depth (Z)']
def feedback(reference,candidate,bindings):
    import copy
    from component_review import selected
    normalized=copy.deepcopy(bindings)
    for region in normalized.values():
        for key,g in [('reference',reference),('candidate',candidate)]:
            selector=region[key]
            if any(k in selector for k in ['colors','textures','centroid_bounds']):region[key]=dict(indices=sorted({v for i in selected(g,selector)for v in g['faces'][i]}))
    comparisons=diagnose(reference,candidate,normalized);findings=[]
    for name,row in comparisons.items():
        a,b=row.get('reference'),row.get('candidate')
        if not a or not b:findings.append(dict(component=name,kind='missing_region',finding='Semantic region has no measurable geometry; author the missing construction before fitting.'));continue
        for axis in range(3):
            target,current=a['extents'][axis],b['extents'][axis]
            if target and abs(current-target)/target>.08:
                findings.append(dict(component=name,kind='proportion',axis=axis,reference=target,candidate=current,finding=f"{name}: {AXES[axis]} is {'too large'if current>target else'too small'}; revise this component's dimension, preserving its attachment."))
            ac=(a['bounds'][0][axis]+a['bounds'][1][axis])/2;bc=(b['bounds'][0][axis]+b['bounds'][1][axis])/2
            if abs(ac-bc)>max(2,target*.08):findings.append(dict(component=name,kind='position',axis=axis,delta_native=ac-bc,finding=f'{name}: move {ac-bc:.1f} native units along {AXES[axis]} after checking the named attachment.'))
        if a.get('palette')and b.get('palette'):
            aa=max(a['palette'],key=a['palette'].get);bb=max(b['palette'],key=b['palette'].get)
            if aa!=bb:findings.append(dict(component=name,kind='material',finding=f'{name}: dominant native palette differs ({aa} versus {bb}); inspect material-region placement, texture and shaded values before changing colors.'))
        if bindings[name].get('construction_requirement'):findings.append(dict(component=name,kind='construction_review',finding=bindings[name]['construction_requirement']))
    return dict(comparisons=comparisons,findings=findings,proposal=propose(comparisons,bindings,candidate),limits='Curated semantic annotation is required. Thresholds triage discrepancies, not universal art quality. No topology or physical collision inference.',artistic_acceptance=False)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('reference',type=Path);p.add_argument('candidate',type=Path);p.add_argument('bindings',type=Path);p.add_argument('out',type=Path);a=p.parse_args();load=lambda p:json.loads(p.read_text());a.out.write_text(json.dumps(feedback(load(a.reference),load(a.candidate),load(a.bindings)),indent=2));print('ACTIONABLE_COMPONENT_FEEDBACK_COMPLETE')
