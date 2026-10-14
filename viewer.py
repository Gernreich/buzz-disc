#!/usr/bin/env python3
"""The two buzz discs in 3D, from their cut files.

    python3 viewer.py [--out=buzz-disc-3d.html]

Each disc is its cut file's own outline at 3mm, read by
lasermade-tools/turn3d.py, stood on edge with a cord loop threaded through its
two centre holes and running out to a hand at each side. Pulling the hands
apart unwinds the twisted cord and spins the disc about it, so "Spin it" turns
the disc about the line between the two holes' midpoint and the hands.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', 'lasermade-tools'))
import turn3d as t   # noqa: E402

DISCS = [('sawtooth disc', 'BuzzDisc1.svg'), ('plain bar', 'BuzzDisc2.svg')]
REACH = 170            # mm from the disc to each hand


def build():
    out = {}
    for name, f in DISCS:
        ps = t.parts(os.path.join(HERE, f))
        assert len(ps) == 1 and len(ps[0]['h']) == 2, (f, len(ps))
        p = ps[0]
        holes = [(sum(q[0] for q in h) / len(h), sum(q[1] for q in h) / len(h)) for h in p['h']]
        mx = sum(h[0] for h in holes) / 2
        my = sum(h[1] for h in holes) / 2
        d = t.centred(p, mx, my)                       # centred on the holes, which is where it spins
        # stood on edge facing the hands: the sheet's x becomes the vertical, its normal the cord
        M = t.chain(t.Ry(math.pi / 2), t.T(0, 0, -1.5))
        cords = []
        loop = []
        for side in (-1, 1):                           # out through one hole, round the hand, back through the other
            (ax, ay), (bx, by) = [(x - mx, y - my) for x, y in holes]
            strand = [(0, ay, -ax), (side * REACH * 0.5, ay * 0.6, -ax * 0.6), (side * REACH, 0, 0),
                      (side * REACH * 0.5, by * 0.6, -bx * 0.6), (0, by, -bx)]
            loop += strand if side < 0 else strand[1:]
        cords.append(loop)
        out[name] = {'parts': [t.placed(d, M)], 'cords': cords,
                     'axis': [0, 0, 0, 1, 0, 0], 'ex': 0, 'view': [-0.9, 0.3, 2.6]}
    return out


def main(argv):
    o = [a.split('=', 1)[1] for a in argv if a.startswith('--out=')]
    path = o[0] if o else os.path.join(HERE, 'buzz-disc-3d.html')
    t.write(path, 'Buzz disc', build(),
            'Drag to turn, scroll or pinch to zoom; pick a disc above. Each is its cut file’s own '
            'outline at 3mm, stood on edge with the cord loop through its two centre holes and out to a '
            'hand at each side. Wind it up, pull the hands apart, and the cord unwinding spins the disc '
            'about it: tick Spin it to see which way.', 'buzz-disc/viewer.py', spin=True)
    print(f'  wrote {os.path.relpath(path)}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
