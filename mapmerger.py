#!/usr/bin/python3

ENOUGH_FOR_MERGE = 3

# import os,sys,shutil,glob,hashlib
from os import listdir,rmdir,getcwd,rename,remove
from os.path import isdir,isfile,splitext,basename,join,exists,getctime
from hashlib import md5

def dirs(path):
	# TODO: replace with filter(os.path.isdir(os.path.join(path, f)), os.listdir(path))
	return [f for f in listdir(path) if isdir(join(path, f))]
def files(path):
	return [f for f in listdir(path) if isfile(join(path, f))]
def ispng(f):
	(dontcare,ext) = splitext(f)
	if ext == '.png':
		return True
	return False
def name2coord(name):
	if name.count('_') != 2:
		print('wrong tile filename:',name)
		exit()
	und1 = name.index('_')
	und2 = name.index('_',und1+1)
	und3 = name.index('.',und2+1)
	return (int(name[und1+1:und2]), int(name[und2+1:und3]))
def remove_empty_dir(path):
	if len(listdir(path)) == 0:
		print('remove empty dir:',basename(path))
		rmdir(path)
		return True
	return False

cwd = getcwd()
dirs_list = [d for d in dirs(cwd) if not remove_empty_dir(join(cwd,d))]

for i,dir1 in enumerate(dirs_list):
	for dir2 in dirs_list[i+1:]:
		print(dir1,'-',dir2)
		dir1path = join(cwd,dir1)
		dir2path = join(cwd,dir2)
		same_hash_cnt = 0
		diff_shift = False
		(pdx,pdy) = (0,0)
		for f1 in filter(ispng,files(dir1path)):
			for f2 in filter(ispng,files(dir2path)):
				f1hash = md5()
				f2hash = md5()
				f1hash.update(open(join(dir1path,f1), mode='rb').read())
				f2hash.update(open(join(dir2path,f2), mode='rb').read())
				if f1hash.digest() == f2hash.digest():
					(x1,y1) = name2coord(f1)
					(x2,y2) = name2coord(f2)
					(dx,dy) = (x1-x2,y1-y2)
					if same_hash_cnt > 0:
						if (dx,dy) != (pdx,pdy):
							diff_shift = True
							print('  diff shifts: ({0},{1}) ({2},{3})'.format(pdx,pdy,dx,dy))
							break
					(pdx,pdy) = (dx,dy)
					same_hash_cnt = same_hash_cnt + 1
			if diff_shift:
				break
		if not diff_shift:
			print('  matches: {0}, shift: ({1},{2})'.format(same_hash_cnt,pdx,pdy))
			if same_hash_cnt >= ENOUGH_FOR_MERGE:
				if pdx != 0 or pdy != 0:
					for f3 in filter(ispng,files(dir2path)):
						(x3,y3) = name2coord(f3)
						(x3,y3) = (x3+pdx, y3+pdy)
						rename(join(dir2path,f3),join(dir2path,'tmp_'+str(x3)+'_'+str(y3)+'.png'))
					for f3 in filter(ispng,files(dir2path)):
						(x3,y3) = name2coord(f3)
						rename(join(dir2path,f3),join(dir2path,'tile_'+str(x3)+'_'+str(y3)+'.png'))
				for f3 in filter(ispng,files(dir2path)):
					if exists(join(dir1path,f3)):
						if getctime(join(dir1path,f3)) < getctime(join(dir2path,f3)):
							remove(join(dir1path,f3))
							rename(join(dir2path,f3),join(dir1path,f3))
						else:
							remove(join(dir2path,f3))
					else:
						rename(join(dir2path,f3),join(dir1path,f3))
				# TODO: remove dir2 from fs and from dirs_list

dirs_list = dirs(cwd)

for d in dirs_list:
	p = join(cwd,d)
	if len(listdir(p)) == 0:
		rmdir(p)
		print(d,'removed')
	else:
		print(d,'has',len(listdir(p)),'entries')
