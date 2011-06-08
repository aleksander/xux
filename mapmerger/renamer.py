#!/usr/bin/python3

import os,sys
from time import time

def dirs(path):
	return [f for f in os.listdir(path) if os.path.isdir(os.path.join(path, f))]

for i,f in enumerate(dirs(os.getcwd())):
    os.rename(os.path.join(os.getcwd(),f),os.path.join(os.getcwd(),'{0}{1:03}'.format(int(time()),i)))
for i,f in enumerate(dirs(os.getcwd())):
    os.rename(os.path.join(os.getcwd(),f),os.path.join(os.getcwd(),'{0:03}'.format(i)))
