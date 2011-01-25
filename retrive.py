#!/usr/bin/python3

import urllib.request, os, sys, shutil

cwd = os.getcwd()
url = 'havenmap.dyndns.org'

def files(path):
	return [f for f in os.listdir(path) if os.path.isfile(os.path.join(path, f))]

if os.path.exists(os.path.join(cwd,url)):
	shutil.rmtree(os.path.join(cwd,url))
os.mkdir(os.path.join(cwd,url))

r = 10
MAX_TRIES = 5
CLEANUP = False

for x in range(-r, r, 1):
	for y in range(-r, r, 1):
		furl='http://'+url+':7800/x='+str(x)+'&y='+str(y)+'&zoom=8'
		fname="tile_"+str(x)+"_"+str(y)+".png"
		tries = 0
		while True:
			print(furl,end=' - ')
			try:
				(fn,hdrs)=urllib.request.urlretrieve(furl, os.path.join(cwd,url,fname))
			except KeyboardInterrupt:
				print('interrupted')
				exit()
			except:
				print(sys.exc_info()[0])
				tries = tries + 1
				if tries > MAX_TRIES:
					break
			else:
				print('OK')
				break

if CLEANUP:
	for f in files(os.path.join(cwd,url)):
		if os.path.getsize(os.path.join(cwd,url,f)) < 150:
			os.remove(os.path.join(cwd,url,f))
