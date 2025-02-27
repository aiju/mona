import numpy as np
import re
import sys

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

vertices = []
normals = []
uv = []
faces = []

with open('/home/aiju/untitled.obj') as f:
    for l in f:
        l = re.sub(r'(#.*)?\n?', '', l)
        if l == "": continue
        fields = l.split(' ')
        if len(fields) == 0: continue
        if fields[0] == 'v':
            vertices += [[float(s) for s in fields[1:]]]
        elif fields[0] == 'vn':
            normals += [[float(s) for s in fields[1:]]]
        elif fields[0] == 'vt':
            uv += [[float(s) for s in fields[1:]]]
        elif fields[0] == "f":
            faces += [[[int(t) for t in s.split('/')] for s in fields[1:]]]
        else:
            eprint("unknown field {}".format(fields[0]))

def print_vertex(i, ti):
    print("        [{}, {}, {}, {}, {}],".format(*(vertices[i - 1] + uv[ti - 1])))

def print_tri(l):
    print("    [")
    for v in l:
        print_vertex(v[0], v[1])
    print("    ],")

print("[")
for face in faces:
    if len(face) == 3:
        print_tri([face[0], face[2], face[1]])
    elif len(face) == 4:
        print_tri([face[0], face[2], face[1]])
        print_tri([face[0], face[3], face[2]])
    else:
        eprint("face with {} vertices".format(len(face)))
print("]")