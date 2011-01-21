#!/usr/bin/python3

import os,sys,shutil,glob,hashlib

def dirs(path):
	# TODO: replace with filter(os.path.isdir(os.path.join(path, f)), os.listdir(path))
	return [f for f in os.listdir(path) if os.path.isdir(os.path.join(path, f))]
def files(path):
	return [f for f in os.listdir(path) if os.path.isfile(os.path.join(path, f))]
def ispng(f):
	(dontcare,ext) = os.path.splitext(f)
	if ext == '.png':
		return True
	return False

cwd = os.getcwd()
dst = os.path.join(cwd,'assembled')

# if os.path.exists(dst):
	# try:
		# shutil.rmtree(dst)
	# except:
		# print("while rmtree() raise error:", sys.exc_info()[0])
		# exit()

dirs_list = dirs(cwd)

for d in dirs_list:
	p = os.path.join(cwd,d)
	if len(os.listdir(p)) == 0:
		print('remove empty dir:',d)
		os.rmdir(p)
		# dirs_list.remove(d)

dirs_list = dirs(cwd)

# try:
	# os.mkdir(dst)
# except:
	# print("while mkdir() raise error:", sys.exc_info()[0])
	# exit()

def name2coord(name):
	if name.count('_') != 2:
		print('wrong tile filename')
		exit()
	und1 = name.index('_')
	und2 = name.index('_',und1+1)
	und3 = name.index('.',und2+1)
	return (int(name[und1+1:und2]), int(name[und2+1:und3]))

for i,dir1 in enumerate(dirs_list):
	for dir2 in dirs_list[i+1:]:
		print(dir1,'-',dir2)
		dir1path = os.path.join(cwd,dir1)
		dir2path = os.path.join(cwd,dir2)
		same_hash_cnt = 0
		diff_shift = False
		(pdx,pdy) = (0,0)
		for f1 in filter(ispng,files(dir1path)):
			for f2 in filter(ispng,files(dir2path)):
				f1hash = hashlib.md5()
				f2hash = hashlib.md5()
				f1hash.update(open(os.path.join(dir1path,f1), mode='rb').read())
				f2hash.update(open(os.path.join(dir2path,f2), mode='rb').read())
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
			if same_hash_cnt > 2:
				if pdx != 0 or pdy != 0:
					for f3 in filter(ispng,files(dir2path)):
						(x3,y3) = name2coord(f3)
						(x3,y3) = (x3+pdx, y3+pdy)
						os.rename(os.path.join(dir2path,f3),os.path.join(dir2path,'tmp_'+str(x3)+'_'+str(y3)+'.png'))
					for f3 in filter(ispng,files(dir2path)):
						(x3,y3) = name2coord(f3)
						os.rename(os.path.join(dir2path,f3),os.path.join(dir2path,'tile_'+str(x3)+'_'+str(y3)+'.png'))
				for f3 in filter(ispng,files(dir2path)):
					if os.path.exists(os.path.join(dir1path,f3)):
						if os.path.getctime(os.path.join(dir1path,f3)) < os.path.getctime(os.path.join(dir2path,f3)):
							os.remove(os.path.join(dir1path,f3))
							os.rename(os.path.join(dir2path,f3),os.path.join(dir1path,f3))
						else:
							os.remove(os.path.join(dir2path,f3))
					else:
						os.rename(os.path.join(dir2path,f3),os.path.join(dir1path,f3))

						
dirs_list = dirs(cwd)
for d in dirs_list:
	p = os.path.join(cwd,d)
	if len(os.listdir(p)) == 0:
		os.rmdir(p)
		print(d,'removed')
		# dirs_list.remove(d)
	else:
		print(d,'has',len(os.listdir(p)),'entries')
