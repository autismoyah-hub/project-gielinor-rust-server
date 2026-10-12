"""Compare observational startup snapshots; never equate this with gameplay parity."""
import argparse
from collections import Counter
import json
from pathlib import Path


def rows(directory, name, columns):
    path=directory/name
    result=[tuple(line.split('\t')) for line in path.read_text().splitlines() if line]
    if any(len(row)!=columns for row in result):
        raise ValueError(f'malformed snapshot: {path}')
    return result


def compare(a,b):
    summary={'scope':'offline startup state only; no live gameplay parity claim','comparisons':{}}
    for name,columns in [('floor-locations.tsv',6),('floor-npcs.tsv',12)]:
        left=Counter(rows(a,name,columns));right=Counter(rows(b,name,columns))
        missing=left-right;extra=right-left
        summary['comparisons'][name]={'equal':left==right,'kotlin_count':left.total(),'rust_count':right.total(),
            'missing_count':missing.total(),'extra_count':extra.total(),
            'missing_sample':list(missing.elements())[:40],'extra_sample':list(extra.elements())[:40]}
    def collision(directory):
        values=rows(directory,'floor-collision.tsv',4)
        table={tuple(map(int,row[:3])):int(row[3])&0xffffffff for row in values}
        expected={(x,z,l) for l in range(2) for x in range(128) for z in range(128)}
        if len(values)!=32768 or set(table)!=expected:
            raise ValueError(f'incomplete or duplicate collision snapshot: {directory}')
        return table
    left=collision(a);right=collision(b)
    differences=[{'tile':key,'kotlin':hex(left[key]),'rust':hex(right[key]),'xor':hex(left[key]^right[key])}
                 for key in sorted(left) if left[key]!=right[key]]
    summary['comparisons']['floor-collision.tsv']={'equal':not differences,'different_tiles':len(differences),'sample':differences[:80]}
    summary['equal']=all(value['equal'] for value in summary['comparisons'].values())
    return summary


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('kotlin',type=Path);parser.add_argument('rust',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args()
    result=compare(args.kotlin,args.rust)
    with args.output.open('x') as out:
        json.dump(result,out,indent=2);out.write('\n')
    print('Startup snapshots equal' if result['equal'] else 'Startup snapshot differences recorded')
    raise SystemExit(0 if result['equal'] else 1)
