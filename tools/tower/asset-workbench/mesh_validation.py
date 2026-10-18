"""Opt-in owned-mesh cleanup; reference interchange never calls this module."""
import math

def degenerate_faces(vertices,faces):
    result=[]
    for index,f in enumerate(faces):
        if len(f)!=3 or any(not isinstance(i,int)or not 0<=i<len(vertices)for i in f):raise ValueError('Invalid owned triangle indices')
        a,b,c=[vertices[i]for i in f];u=[b[i]-a[i]for i in range(3)];v=[c[i]-a[i]for i in range(3)];cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        if sum(x*x for x in cross)==0:result.append(index)
    return result

def clean_owned(vertices,faces):
    if any(not math.isfinite(c)or int(c)!=c for v in vertices for c in v):raise ValueError('Owned native geometry requires finite integer coordinates')
    removed=set(degenerate_faces(vertices,faces));kept=[f for i,f in enumerate(faces)if i not in removed]
    if not kept:raise ValueError('Owned component collapses entirely after native quantization')
    return vertices,kept,dict(input_faces=len(faces),removed_zero_area_faces=len(removed),output_faces=len(kept),vertex_order_preserved=True)
