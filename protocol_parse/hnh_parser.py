#!/usr/bin/env python
# -*- coding: utf-8 -*-

import pcapy, struct, sys

# counters = {'tcp':0,'udp':0,'other':0}
msgs = {0:'SESS',1:'REL',2:'ACK',3:'BEAT',4:'MAPREQ',5:'MAPDATA',6:'OBJDATA',7:'OBJACK',8:'CLOSE'}
sesserr = ('OK','AUTH','BUSY','CONN','PVER','EXPR')
rels = ('NEWWDG','WDGMSG','DSTWDG','MAPIV','GLOBLOB','PAGINAE','RESID','PARTY','SFX','CATTR','MUSIC','TILES','BUFF')
wdg_list_types = {0:'END',1:'INT',2:'STR',3:'COORD',6:'COLOR'}
gmsg = ('TIME','ASTRO','LIGHT')

# def cut(data,fmt):
	# res = struct.unpack(fmt,data[:struct.calcsize(fmt)])
	# data = data[struct.calcsize(fmt):]
	# return res
def cu8(data):
	ret = data[0]
	data[0:1] = []
	return ret
def cu16(data):
	ret = data[0]+(data[1]<<8)
	data[0:2] = []
	return ret
def cstr(data):
	tmp = data.index(b'\x00')
	str = data[:tmp].decode()
	data[0:tmp+1] = []
	return str
def cb(data,count=0):
	if count > 0:
		ret = data[:count]
		data[0:count] = []
	else:
		ret = data[:]
		data[:] = []
	return ret
#FIXME: not work properly
def cs32(data):
	ret = data[0]+(data[1]<<8)+(data[2]<<16)+(data[3]<<24)
	# if ret > 0x7fffffff:
		# ret = 0x80000000*2-ret;
	data[0:4] = []
	return ret

def hnh_parse(data,server):
	type = cu8(data)
	if type not in msgs:
		print(' UNKNOWN PACKET TYPE {}'.format(type))
		return
	print(' {} ({})'.format(msgs[type],type))
	try:
		######## SESS #################################
		if type == 0:
			if server:
				error = cu8(data)
				print('error={0}({1})'.format(error,sesserr[error]))
			else:
				print(data)
				cu16(data) # ???
				proto = cstr(data)
				ver = cu16(data)
				user = cstr(data)
				cookie = cb(data)
				print('proto={} ver={} user={} cookie={}'.format(proto,ver,user,cookie[:5]))
		######## REL ##################################
		elif type == 1:
			seq = cu16(data)
			while len(data) > 0:
				rel_type = cu8(data)
				if rel_type&0x80 != 0:
					rel_type &= 0x7f;
					rel_len = cu16(data);
				else:
					rel_len = len(data)
				rel = cb(data,rel_len)
				# self.rel = self.bytes(self.rel_len)
				print('  seq={0} type={1}({2}) len={3}'.format(seq,rel_type,rels[rel_type],rel_len))
				if rel_type == 0: # NEWWDG
					wdg_id = cu16(rel)
					wdg_type = cstr(rel)
					wdg_coord = [cs32(rel),cs32(rel)]
					wdg_parent = cu16(rel)
					print('   id={} type={} parent={}'.format(wdg_id,wdg_type,wdg_parent))
					while len(rel) > 0:
						wdg_lt = cu8(rel)
						print('    {}='.format(wdg_list_types[wdg_lt]),end='')
						if wdg_lt == 0: # END
							break
						elif wdg_lt == 1: # INT
							print(cs32(rel))
						elif wdg_lt == 2: # STR
							print(cstr(rel))
						elif wdg_lt == 3: # COORD
							print([cs32(rel),cs32(rel)])
						elif wdg_lt == 6: # COLOR
							print([cu8(rel),cu8(rel),cu8(rel),cu8(rel)])
				elif rel_type == 1: # WDGMSG
					wdg_id = cu16(rel)
					wdg_msg_name = cstr(rel)
					print('   id={} name={}'.format(wdg_id,wdg_msg_name))
					while len(rel) > 0:
						wdg_lt = cu8(rel)
						if wdg_lt not in wdg_list_types:
							print('    !!! wdg_lt={}'.format(wdg_lt))
							break
						print('    {}='.format(wdg_list_types[wdg_lt]),end='')
						if wdg_lt == 0: # END
							break
						elif wdg_lt == 1: # INT
							print(cs32(rel))
						elif wdg_lt == 2: # STR
							print(cstr(rel))
						elif wdg_lt == 3: # COORD
							print([cs32(rel),cs32(rel)])
						elif wdg_lt == 6: # COLOR
							print([cu8(rel),cu8(rel),cu8(rel),cu8(rel)])
				elif rel_type == 2: # DSTWDG (destroy widget)
					dw_id = cu16(rel)
					print('   id={}'.format(dw_id))
				elif rel_type == 3: # MAPIV
					mapiv_type = cu8(rel)
					if mapiv_type == 0: # ???
						print('    invalidate coord={}'.format([cs32(rel),cs32(rel)]))
					elif mapiv_type == 1: # ???
						print('    trim ul={} lr={}'.format([cs32(rel),cs32(rel)],[cs32(rel),cs32(rel)]))
					elif mapiv_type == 2: # ???
						print('    trim all')
				elif rel_type == 4: # GLOBLOB
					while len(rel) > 0:
						gmsg_type = cu8(rel)
						print('    {}='.format(gmsg[gmsg_type]),end='')
						if gmsg_type == 0: # TIME
							print(cs32(rel))
						elif gmsg_type == 1: # ASTRO
							print('dt={} mp={} yt={}'.format(cs32(rel),cs32(rel),cs32(rel)))
						elif gmsg_type == 2: # LIGHT
							print([cu8(rel),cu8(rel),cu8(rel),cu8(rel)])
				elif rel_type == 5: # PAGINAE
					while len(rel) > 0:
						print('    act={} name={} ver={}'.format(cu8(rel),cstr(rel),cu16(rel)))
				elif rel_type == 6: # RESID
					res_id = cu16(rel)
					res_name = cstr(rel)
					res_ver = cu16(rel)
					print('   id={} name={} ver={}'.format(res_id,res_name,res_ver))
				elif rel_type == 7: # PARTY
					while len(rel) > 0:
						party_type = cu8(rel)
						if party_type == 0: # LIST
							print('   LIST')
							while True:
								party_id = cs32(rel)
								if party_id > 0x7fffffff:
									break
								print('    id={}'.format(party_id))
						elif party_type == 1: # LEADER
							print('   LEADER id={}'.format(cs32(rel)))
						elif party_type == 2: # MEMBER
							print('   MEMBER id={} vis={} coord={} color={}'.format(cs32(rel),cu8(rel),[cs32(rel),cs32(rel)],[cu8(rel),cu8(rel),cu8(rel),cu8(rel)]))
				elif rel_type == 8: # SFX
					print('   res={} vol={} spd={}'.format(cu16(),cu16(),cu16()))
				elif rel_type == 9: # CATTR
					while len(rel) > 0:
						attr_name = cstr(rel)
						attr_base = cs32(rel)
						attr_comp = cs32(rel)
						print('   name={} base={} comp={}'.format(attr_name,attr_base,attr_comp))
				elif rel_type == 10: # MUSIC
					music_name = cstr(rel)
					music_ver = cu16(rel)
					if len(rel) > 0:
						music_loop = cu8(rel)
					else:
						music_loop = 0
					print('   name={} ver={} loop={}'.format(music_name,music_ver,music_loop))
				elif rel_type == 11: # TILES
					while len(rel) > 0:
						tile_id = cu8(rel)
						tile_name = cstr(rel)
						tile_ver = cu16(rel)
						print('   id={0} name={1} version={2}'.format(tile_id,tile_name,tile_ver))
				elif rel_type == 12: # BUFF
					buff_name = cstr(rel)
					if buff_name == 'clear':
						print('   clear buffers')
					elif buff_name == 'set':
						print('   set buffers id={} res={} tt={} ameter={} nmeter={} cmeter={} cticks={} major={}'.format(cs32(rel),cu16(rel),cstr(rel),cs32(rel),cs32(rel),cs32(rel),cs32(rel),cu8(rel)))
					elif buff_name == 'rm':
						print('   remove buffers id={}'.format(cs32(rel)))
				if len(rel) > 0:
					print('rel remains={}'.format(rel))
				seq += 1
		######## ACK ##################################
		elif type == 2:
			seq = cu16(data)
			print('  seq={0}'.format(seq))
		######## BEAT #################################
		elif type == 3:
			pass
		######## MAPREQ ###############################
		elif type == 4:
			pass
		######## MAPDATA ##############################
		elif type == 5:
			pass
		######## OBJDATA ##############################
		elif type == 6:
			pass
		######## OBJACK ###############################
		elif type == 7:
			pass
		######## CLOSE ################################
		elif type == 8:
			pass
		# if len(self.data) > 0:
			# self.desc += ' remains='+str(self.data)
	except:
		print("Unexpected error:", sys.exc_info()[0])
		# raise
def show_info(hdr,data):
	fmt = '!6s6sH'
	(macdst,macsrc,ethertype) = struct.unpack(fmt,data[:struct.calcsize(fmt)])
	if ethertype != 0x0800:
		print('not IP !!!')
		return
	data = data[struct.calcsize(fmt):]
	fmt = '!BBHHHBBH4s4s'
	(vhl,dscp,len,id,ffo,ttl,proto,crc,ipsrc,ipdst) = struct.unpack(fmt,data[:struct.calcsize(fmt)])
	if vhl&0xf > 5:
		print('ip options !!!')
		return
	if proto != 0x11:
		# print('not UDP !!!')
		return
	data = data[struct.calcsize(fmt):]
	fmt = '!HHHH'
	(portsrc,portdst,len,crc) = struct.unpack(fmt,data[:struct.calcsize(fmt)])
	data = data[struct.calcsize(fmt):]
	# print('{0}.{1}.{2}.{3}:{4} -> {5}.{6}.{7}.{8}:{9}'.format(ipsrc[0],ipsrc[1],ipsrc[2],ipsrc[3],portsrc,ipdst[0],ipdst[1],ipdst[2],ipdst[3],portdst))
	if ipdst == bytes([178,63,100,209]):
		print('CLIENT')
		hnh_parse(bytearray(data),False)
	else:
		print('SERVER')
		hnh_parse(bytearray(data),True)

# for i in range(100):
	# print()
rdr = pcapy.open_offline('hh.pcap')
rdr.dispatch(-1,show_info)
# print(counters)