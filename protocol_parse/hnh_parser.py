#!/usr/bin/python3.2
# -*- coding: utf-8 -*-

import pure_pcapy as pcapy
import struct, sys

# counters = {'tcp':0,'udp':0,'other':0}
msgs = {0:'SESS',1:'REL',2:'ACK',3:'BEAT',4:'MAPREQ',5:'MAPDATA',6:'OBJDATA',7:'OBJACK',8:'CLOSE'}
sesserr = ('OK','AUTH','BUSY','CONN','PVER','EXPR')
rels = ('NEWWDG','WDGMSG','DSTWDG','MAPIV','GLOBLOB','PAGINAE','RESID','PARTY','SFX','CATTR','MUSIC','TILES','BUFF')
wdg_list_types = {0:'END',1:'INT',2:'STR',3:'COORD',6:'COLOR'}
gmsg = ('TIME','ASTRO','LIGHT')
    # public static final int OD_REM = 0;
    # public static final int OD_MOVE = 1;
    # public static final int OD_RES = 2;
    # public static final int OD_LINBEG = 3;
    # public static final int OD_LINSTEP = 4;
    # public static final int OD_SPEECH = 5;
    # public static final int OD_LAYERS = 6;
    # public static final int OD_DRAWOFF = 7;
    # public static final int OD_LUMIN = 8;
    # public static final int OD_AVATAR = 9;
    # public static final int OD_FOLLOW = 10;
    # public static final int OD_HOMING = 11;
    # public static final int OD_OVERLAY = 12;
    # /* public static final int OD_AUTH = 13; -- Removed */
    # public static final int OD_HEALTH = 14;
    # public static final int OD_BUDDY = 15;
    # public static final int OD_END = 255;
objdata_types = {0:'REM',1:'MOVE',2:'RES',3:'LINBEG',4:'LINSTEP',5:'SPEECH',6:'LAYERS',7:'DRAWOFF',8:'LUMIN',
                 9:'AVATAR',10:'FOLLOW',11:'HOMING',12:'OVERLAY',14:'HEALTH',15:'BUDDY',255:'END'}

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

def cu32(data):
	ret = data[0]+(data[1]<<8)+(data[2]<<16)+(data[3]<<24)
	data[0:4] = []
	return ret

	# public static final int MIN_VALUE = -2147483648;
	# public static final int MAX_VALUE = 2147483647;
    # static int int32d(byte[] buf, int off) {
	# long u = uint32d(buf, off);
	# if(u > Integer.MAX_VALUE)
	    # return((int)((((long)Integer.MIN_VALUE) * 2) - u));
	# else
	    # return((int)u);
    # }
	
	#     0000 000E
	# v = FFFF FFF1 === -15
	# s = 8000 0000
	# 
def cs32(data):
	ret = data[0]+(data[1]<<8)+(data[2]<<16)+(data[3]<<24)
	if ret>2147483647:
		ret = -((2147483648*2)-ret)
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
				print('  error={0}({1})'.format(error,sesserr[error]))
			else:
				cu16(data) # ???
				proto = cstr(data)
				ver = cu16(data)
				user = cstr(data)
				cookie = cb(data)
				print('  proto={} ver={} user={} cookie={}'.format(proto,ver,user,cookie))
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
					print('   id={} type={} coord={} parent={}'.format(wdg_id,wdg_type,wdg_coord,wdg_parent))
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
								# FIXME: replace with cs32
								party_id = cu32(rel)
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
			pktid = cs32(data)
			off = cu16(data)
			length = cu16(data)
			buf = cb(data)
			print('   pktid={} off={} len={} buf={}'.format(pktid,off,length,buf))
		######## OBJDATA ##############################
		elif type == 6:
			while len(data) > 0:
				objdata_fl = cu8(data)
				objdata_id = cs32(data)
				objdata_frame = cs32(data)
				print('  id={} frame={}'.format(objdata_id,objdata_frame))
				if objdata_fl&1 != 0:
					print('   remove id={} frame={}'.format(objdata_id,objdata_frame-1))
				while True:
					objdata_type = cu8(data)
					if objdata_type not in objdata_types:
						print('   UNKNOWN OBJDATA TYPE {}'.format(objdata_type))
						raise Exception('unknown objdata type', '...')
					print('   {}'.format(objdata_types[objdata_type]),end=' ')
					if objdata_type == 0: # REM
						pass
					elif objdata_type == 1: # MOVE
						print('coord={}'.format([cs32(data),cs32(data)]))
					elif objdata_type == 2: # RES
						res_id = cu16(data)
						if res_id&0x8000 != 0:
							res_id &= ~0x8000
							print('res_id={} sdt={}'.format(res_id,cb(data,cu8(data))))
						else:
							print('res_id={} sdt=[]'.format(res_id))
					elif objdata_type == 3: # LINBEG
						print('s={} t={} c={}'.format([cs32(data),cs32(data)],[cs32(data),cs32(data)],cs32(data)))
					elif objdata_type == 4: # LINSTEP
						print('l={}'.format(cs32(data)))
					elif objdata_type == 5: # SPEECH
						print('off={} text={}'.format([cs32(data),cs32(data)],cstr(data)))
					elif objdata_type == 6: # LAYERS
						res = cu16(data)
						layers = []
						while True:
							layer = cu16(data)
							if layer == 65535:
								break
							layers.append(layer)
						print('res={} layers={}'.format(res,layers))
					elif objdata_type == 7: # DRAWOFF
						print('off={}'.format([cs32(data),cs32(data)]))
					elif objdata_type == 8: # LUMIN
						print('off={} sz={} str={}'.format([cs32(data),cs32(data)],cu16(data),cu8(data)))
					elif objdata_type == 9: # AVATAR
						layers = []
						while True:
							layer = cu16(data)
							if layer == 65535:
								break
							layers.append(layer)
						print('layers={}'.format(layers))
					elif objdata_type == 10: # FOLLOW
						oid = cs32(data)
						# FIXME !!!!!!
						if oid != -1:
							print('oid={} off={} szo={}'.format(oid,[cs32(data),cs32(data)],cu8(data)))
						else:
							print('oid={} off=[???,???] szo=0'.format(oid))
					elif objdata_type == 11: # HOMING
						oid = cs32(data)
						print('oid={}'.format,end=' ')
						if oid == -1:
							print('homostop')
						elif oid == -2:
							print('homocoord coord={} v={}'.format([cs32(data),cs32(data)],cu16(data)))
						else:
							print('homing coord={} v={}'.format([cs32(data),cs32(data)],cu16(data)))
					elif objdata_type == 12: # OVERLAY
						olid = cs32(data)
						prs = (olid & 1) != 0
						olid >>= 1
						resid = cu16()
						if resid == 65535:
							sdt = None
						elif resid&0x8000 != 0:
							resid &= ~0x8000
							sdt = cb(data,cu8(data))
						else:
							sdt = []
						print('olid={} prs={} resid={} sdt={}'.format(olid,prs,resid,sdt))
					elif objdata_type == 14: # HEALTH
						print('hp={}'.format(cu8(data)))
					elif objdata_type == 15: # BUDDY
						print('name={} group={} btype={}'.format(cstr(data),cu8(data),cu8(data)))
					elif objdata_type == 255: # END
						print('')
						break
		######## OBJACK ###############################
		elif type == 7:
			print('   id={} frame={}'.format(cs32(data),cs32(data)))
		######## CLOSE ################################
		elif type == 8:
			pass
		if len(data) > 0:
			print('data remains={}'.format(data))
	except:
		print("Unexpected error: {} {} {}".format(sys.exc_info()[0],sys.exc_info()[1],sys.exc_info()[2]))
		if len(data) > 0:
			print('data remains={}'.format(data))
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
rdr = pcapy.open_offline('first.pcap')
rdr.dispatch(-1,show_info)
# print(counters)
